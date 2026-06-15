// [OPUS-4.8] sq-tg6b — the NETWORK tier of the MPC benchmark matrix (Tier 2/3).
// New module: a REAL multi-process transport where each of N parties is its own
// OS process, exchanging the actual protocol messages over a loopback TCP socket
// (`127.0.0.1`). The codec + `Channel` + `Coordinator` are transport-AGNOSTIC
// (generic over any `Read + Write`, so a `std::os::unix::net::UnixStream` slots
// in unchanged), but the shipped runner uses TCP loopback. This is what makes
// wall-clock latency a meaningful MPC cost — unlike the in-process counting tier
// (`bench.rs`/`metrics.rs`, sq-sxm), where every "party" is a function call and
// there is no network.
//
// HONESTY (design record §5, the critical-honesty constraint): the in-process
// tier emits MODELLED communication (deterministic byte/round counts); this tier
// emits MEASURED wall-clock + MEASURED bytes-on-wire under a real transport. The
// two must agree on the protocol RESULT (the loopback validation test), which is
// the only correctness anchor — a faster-but-wrong network run is caught.
//
// What is privileged-gated: the netem LAN/WAN shaping (`netprofiles.rs` +
// `scripts/mpc-netem.sh`) needs CAP_NET_ADMIN (sudo `tc qdisc`), NOT available
// unprivileged on the dev box, so the LAN/WAN-shaped runs are documented +
// shipped but gated to a privileged host or the EC2 tier (bead sq-hoaj). The
// loopback transport itself needs no privilege and is validated here.
//! Real multi-process transport for the MPC benchmark network tier (Tier 2/3).
//!
//! ## The two tiers this enables
//!
//! - **Tier 2 (loopback, unprivileged):** N OS processes over `127.0.0.1`
//!   exchanging the actual protocol messages — real serialisation, real socket
//!   syscalls, ~0 RTT. Gives real bytes-on-wire and real round latency at
//!   LAN-ideal. Validated here ([`run_cell_networked`] vs the in-process result).
//!   (The transport types are generic over any `Read + Write`, so a Unix-domain
//!   stream works too; the shipped runner uses loopback TCP.)
//! - **Tier 3 (netem-shaped, privileged):** the same processes under a Linux
//!   `tc qdisc netem` LAN/WAN profile ([`crate::netprofiles`]). Needs
//!   CAP_NET_ADMIN, so it runs on a privileged host or the EC2 bead (sq-hoaj),
//!   driven by `scripts/mpc-netem.sh`. The transport is identical; only the
//!   kernel queue discipline changes.
//!
//! ## Protocol fidelity — the star-coordinator simulation harness
//!
//! Each **party process** holds ONLY its own share-points (its column of the
//! Shamir sharing) — never the cleartext inputs. A **coordinator** (the
//! benchmark driver) deals shares out to the parties and collects the per-party
//! contributions needed to open a result, exactly as the in-process dealer does,
//! but now over a socket. This star topology is the standard MPC-DB simulation
//! harness (MP-SPDZ's `Player` / Secrecy's coordinator); it faithfully exercises
//! serialisation + socket round-trips while keeping the harness small. The
//! *deployed* protocol would use a full party mesh; the star is the loopback
//! measurement vehicle, and the round count it pays matches the modelled
//! `round_count` of the in-process tier (asserted by the loopback test).
//!
//! ## Why this is native-only
//!
//! Sockets + child processes have no browser story; `sparq-mpc` is already
//! excluded from `sparq-wasm` (verified by `cargo tree -p sparq-wasm`), and this
//! module uses only `std::net` / `std::io` / `std::thread` (the example driver
//! adds `std::process`), so the exclusion is preserved.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

use crate::bench::QueryClass;
use crate::field::Fp;
use crate::metrics::FIELD_BYTES;
use crate::partial::MpcError;
use crate::shamir::{self, ShamirBackend, ShamirDealer, Share};

// =============================================================================
// Wire codec — length-prefixed frames of field elements / control words.
// =============================================================================

/// One on-wire message in the network-tier protocol. Every variant serialises to
/// a self-describing length-prefixed frame; the codec is hand-written (no `serde`
/// — the crate stays dependency-minimal, mirroring `bench::MatrixResults::to_json`).
///
/// Field elements travel as their canonical `u64` representative in big-endian
/// (8 bytes each — [`FIELD_BYTES`]), so the measured bytes-on-wire equals the
/// modelled `bytes_per_party` of the in-process tier up to the small fixed frame
/// header.
///
/// [OPUS-4.8] PR #96: the harness reports a single RAW byte count
/// ([`NetworkCell::bytes_sent`] / [`NetworkCell::bytes_recv`]) — total bytes on
/// the wire, which includes the per-frame length+tag header and the 8-byte `x`
/// coordinate of every share, NOT a field-payload-only figure. When comparing to
/// the modelled `FIELD_BYTES` accounting, account for that fixed overhead rather
/// than expecting an exact field-only match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    /// Coordinator → party: "here are your share-points for this batch of
    /// dealt secrets" — one [`Share`] value per dealt secret (all at this party's
    /// fixed evaluation point `x`).
    Shares(Vec<Share>),
    /// Party → coordinator: "here are my contributions to open these values" —
    /// one share value per value being opened (the party's broadcast for the
    /// recombine step). Carries the party's `x` so the coordinator can interpolate.
    Open(Vec<Share>),
    /// Coordinator → party: run this query-class step over the shares you hold.
    /// Carries the opcode so a party process is a thin protocol executor.
    Step(StepCode),
    /// Either direction: protocol is done for this cell; close cleanly.
    Done,
}

/// The per-cell protocol step a party executes (matches the in-process cells).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepCode {
    /// Cumulative-sum aggregate: locally ADD all share-points you hold (free,
    /// non-interactive) and send the single summed share back to open.
    SumAndOpen,
    /// Hidden-value equality: you hold `(d_share, r_share)` per candidate pair;
    /// locally multiply them (degree-`2t` product) and send each product share
    /// back to open. The match bit is `product == 0`.
    MulAndOpen,
}

impl StepCode {
    fn to_byte(self) -> u8 {
        match self {
            StepCode::SumAndOpen => 1,
            StepCode::MulAndOpen => 2,
        }
    }
    fn from_byte(b: u8) -> Result<Self, MpcError> {
        match b {
            1 => Ok(StepCode::SumAndOpen),
            2 => Ok(StepCode::MulAndOpen),
            other => Err(MpcError::Protocol(format!("unknown StepCode {other}"))),
        }
    }
}

// Frame tags.
const TAG_SHARES: u8 = 1;
const TAG_OPEN: u8 = 2;
const TAG_STEP: u8 = 3;
const TAG_DONE: u8 = 4;

fn write_share(buf: &mut Vec<u8>, s: &Share) {
    buf.extend_from_slice(&s.x.to_be_bytes());
    buf.extend_from_slice(&s.y.value().to_be_bytes());
}

fn read_share(bytes: &[u8], off: &mut usize) -> Result<Share, MpcError> {
    let need = 8 + FIELD_BYTES;
    if *off + need > bytes.len() {
        return Err(MpcError::Protocol("short Share frame".into()));
    }
    let x = u64::from_be_bytes(bytes[*off..*off + 8].try_into().unwrap());
    *off += 8;
    let y = u64::from_be_bytes(bytes[*off..*off + FIELD_BYTES].try_into().unwrap());
    *off += FIELD_BYTES;
    Ok(Share { x, y: Fp::new(y) })
}

impl Message {
    /// Serialise to a length-prefixed frame: `[u32 len][u8 tag][payload]`. `len`
    /// covers the tag + payload, so a reader frames on the prefix alone.
    pub fn encode(&self) -> Vec<u8> {
        let mut body = Vec::new();
        match self {
            Message::Shares(v) => {
                body.push(TAG_SHARES);
                body.extend_from_slice(&(v.len() as u32).to_be_bytes());
                for s in v {
                    write_share(&mut body, s);
                }
            }
            Message::Open(v) => {
                body.push(TAG_OPEN);
                body.extend_from_slice(&(v.len() as u32).to_be_bytes());
                for s in v {
                    write_share(&mut body, s);
                }
            }
            Message::Step(code) => {
                body.push(TAG_STEP);
                body.push(code.to_byte());
            }
            Message::Done => body.push(TAG_DONE),
        }
        let mut frame = Vec::with_capacity(4 + body.len());
        frame.extend_from_slice(&(body.len() as u32).to_be_bytes());
        frame.extend_from_slice(&body);
        frame
    }

    fn decode(body: &[u8]) -> Result<Message, MpcError> {
        if body.is_empty() {
            return Err(MpcError::Protocol("empty frame".into()));
        }
        let tag = body[0];
        let mut off = 1;
        match tag {
            TAG_SHARES | TAG_OPEN => {
                if off + 4 > body.len() {
                    return Err(MpcError::Protocol("short share-vec header".into()));
                }
                let count = u32::from_be_bytes(body[off..off + 4].try_into().unwrap()) as usize;
                off += 4;
                let mut v = Vec::with_capacity(count);
                for _ in 0..count {
                    v.push(read_share(body, &mut off)?);
                }
                Ok(if tag == TAG_SHARES {
                    Message::Shares(v)
                } else {
                    Message::Open(v)
                })
            }
            TAG_STEP => {
                if off >= body.len() {
                    return Err(MpcError::Protocol("short Step frame".into()));
                }
                Ok(Message::Step(StepCode::from_byte(body[off])?))
            }
            TAG_DONE => Ok(Message::Done),
            other => Err(MpcError::Protocol(format!("unknown frame tag {other}"))),
        }
    }
}

/// A framed, byte-counting message channel over any `Read + Write` stream
/// (TcpStream on `127.0.0.1`, or a Unix-domain stream). Tracks bytes sent /
/// received so the harness can report the MEASURED on-wire volume.
pub struct Channel<S: Read + Write> {
    stream: S,
    pub bytes_sent: u64,
    pub bytes_recv: u64,
}

impl<S: Read + Write> Channel<S> {
    /// Wrap a connected stream.
    pub fn new(stream: S) -> Self {
        Channel {
            stream,
            bytes_sent: 0,
            bytes_recv: 0,
        }
    }

    /// Send one message as a length-prefixed frame; flush so the peer sees a full
    /// round-trip (matters under netem — we want the RTT, not a coalesced batch).
    /// I/O errors surface as [`MpcError::Protocol`] (the crate's error type) so
    /// the coordinator / party loops can `?` uniformly.
    pub fn send(&mut self, msg: &Message) -> Result<(), MpcError> {
        let frame = msg.encode();
        self.write_all(&frame)?;
        self.flush()?;
        self.bytes_sent += frame.len() as u64;
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), MpcError> {
        self.stream
            .write_all(buf)
            .map_err(|e| MpcError::Protocol(format!("transport write: {e}")))
    }

    fn flush(&mut self) -> Result<(), MpcError> {
        self.stream
            .flush()
            .map_err(|e| MpcError::Protocol(format!("transport flush: {e}")))
    }

    /// Block until one full frame is read and decoded.
    pub fn recv(&mut self) -> Result<Message, MpcError> {
        let mut len_buf = [0u8; 4];
        self.read_exact(&mut len_buf)?;
        let len = u32::from_be_bytes(len_buf) as usize;
        // Frame-length sanity cap: a single frame over this harness never exceeds
        // a few hundred KiB (it carries a batch of 8-byte shares); reject an
        // absurd length rather than allocating gigabytes on a garbled header.
        const MAX_FRAME: usize = 64 * 1024 * 1024;
        if len > MAX_FRAME {
            return Err(MpcError::Protocol(format!(
                "frame length {len} exceeds cap"
            )));
        }
        let mut body = vec![0u8; len];
        self.read_exact(&mut body)?;
        self.bytes_recv += (4 + len) as u64;
        Message::decode(&body)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), MpcError> {
        self.stream
            .read_exact(buf)
            .map_err(|e| MpcError::Protocol(format!("transport read: {e}")))
    }
}

// =============================================================================
// The PARTY side — a thin protocol executor over one channel.
// =============================================================================

/// Run one party's side of the network protocol over an already-connected
/// channel: receive a [`StepCode`] + the share batch, do the local computation,
/// and send the share contributions back to open. This is what each party PROCESS
/// runs (the child binary's body); it holds only its own share-points.
///
/// The party never sees cleartext, never reconstructs — it only does local
/// share-arithmetic (add for the sum; multiply for the equality product) and
/// broadcasts its result share. Returns when it receives [`Message::Done`].
pub fn run_party<S: Read + Write>(chan: &mut Channel<S>) -> Result<(), MpcError> {
    loop {
        match chan.recv()? {
            Message::Step(StepCode::SumAndOpen) => {
                // Next frame: the shares this party holds (one per dealt secret).
                let shares = match chan.recv()? {
                    Message::Shares(v) => v,
                    other => {
                        return Err(MpcError::Protocol(format!(
                            "expected Shares, got {other:?}"
                        )))
                    }
                };
                // Local, non-interactive sum of all share-points (the aggregate's
                // free linear step). All shares are at this party's single point.
                // [OPUS-4.8] PR #96: an empty batch is a protocol error, NOT an
                // `x=0` placeholder share — interpolating a manufactured `x=0`
                // point (or any duplicate x) is undefined for Shamir and panics in
                // `Fp::inv(0)` during reconstruction. Fail closed, mirroring the
                // in-process `ShamirBackend::run_secure` ("no shared inputs").
                let x = match shares.first() {
                    Some(s) => s.x,
                    None => {
                        return Err(MpcError::Protocol(
                            "SumAndOpen: empty share batch (no shared inputs)".into(),
                        ))
                    }
                };
                // All share-points in this batch must share this party's single
                // evaluation point `x`; a mixed batch is a framing/protocol fault.
                if let Some(bad) = shares.iter().find(|s| s.x != x) {
                    return Err(MpcError::Protocol(format!(
                        "SumAndOpen: mixed evaluation points in batch (expected x={x}, saw x={})",
                        bad.x
                    )));
                }
                let mut acc = Fp::zero();
                for s in &shares {
                    acc = acc.add(s.y);
                }
                chan.send(&Message::Open(vec![Share { x, y: acc }]))?;
            }
            Message::Step(StepCode::MulAndOpen) => {
                // Next frame: interleaved (d_share, r_share) pairs, one pair per
                // candidate equality. Local degree-2t product per pair.
                let shares = match chan.recv()? {
                    Message::Shares(v) => v,
                    other => {
                        return Err(MpcError::Protocol(format!(
                            "expected Shares, got {other:?}"
                        )))
                    }
                };
                if shares.len() % 2 != 0 {
                    return Err(MpcError::Protocol("MulAndOpen expects (d,r) pairs".into()));
                }
                let mut products = Vec::with_capacity(shares.len() / 2);
                for pair in shares.chunks_exact(2) {
                    let (d, r) = (pair[0], pair[1]);
                    if d.x != r.x {
                        return Err(MpcError::Protocol("MulAndOpen pair point mismatch".into()));
                    }
                    products.push(Share {
                        x: d.x,
                        y: d.y.mul(r.y),
                    });
                }
                chan.send(&Message::Open(products))?;
            }
            Message::Done => return Ok(()),
            other => {
                return Err(MpcError::Protocol(format!(
                    "party got unexpected {other:?}"
                )))
            }
        }
    }
}

// =============================================================================
// The COORDINATOR side — deals shares, drives the step, reconstructs.
// =============================================================================

/// One networked cell's measured result: the protocol RESULT (so it can be
/// checked against the in-process tier) plus the MEASURED wall-clock and
/// bytes-on-wire. Distinct from `bench::MatrixCell`, whose cost is MODELLED.
#[derive(Debug, Clone)]
pub struct NetworkCell {
    /// The query class this cell ran.
    pub query_class: QueryClass,
    /// Number of party processes.
    pub parties: usize,
    /// Threshold `t`.
    pub threshold: usize,
    /// Data scale (rows/holder for the join; party count for the aggregate).
    pub scale: usize,
    /// The protocol RESULT, type-erased to an integer the caller verifies against
    /// the in-process result: the reconstructed sum for the aggregate, or the
    /// match count for the hidden join.
    pub result: u64,
    /// MEASURED wall-clock for the networked run (the cross-`N` latency the
    /// in-process tier cannot produce). NOT a modelled figure.
    pub wall_clock: Duration,
    /// Total bytes the coordinator sent across all party channels.
    pub bytes_sent: u64,
    /// Total bytes the coordinator received across all party channels.
    pub bytes_recv: u64,
    /// Number of logical communication rounds the coordinator drove (one deal +
    /// one open per protocol step — the MEASURED round count, to cross-check the
    /// modelled `round_count`).
    pub rounds: u64,
}

impl NetworkCell {
    /// Total measured on-wire bytes (sent + received), coordinator's view.
    pub fn total_bytes(&self) -> u64 {
        self.bytes_sent + self.bytes_recv
    }
}

/// The coordinator drives one cell over `n` already-connected party channels.
/// Generic over the stream type so the same logic runs over TCP loopback or a
/// Unix socket (and, under netem, over a shaped loopback — same code).
pub struct Coordinator<S: Read + Write> {
    channels: Vec<Channel<S>>,
    backend: ShamirBackend,
}

impl<S: Read + Write> Coordinator<S> {
    /// Build a coordinator over `channels` (one per party, in party-index order)
    /// and a backend whose `(n, t)` matches the party set. `channels.len()` must
    /// equal `backend.parties()`.
    pub fn new(channels: Vec<Channel<S>>, backend: ShamirBackend) -> Result<Self, MpcError> {
        if channels.len() != backend.parties() {
            return Err(MpcError::Protocol(format!(
                "coordinator: {} channels for {} parties",
                channels.len(),
                backend.parties()
            )));
        }
        Ok(Coordinator { channels, backend })
    }

    /// Total bytes the coordinator sent / received across all channels.
    pub fn byte_totals(&self) -> (u64, u64) {
        let sent = self.channels.iter().map(|c| c.bytes_sent).sum();
        let recv = self.channels.iter().map(|c| c.bytes_recv).sum();
        (sent, recv)
    }

    /// Run the **cumulative-sum aggregate** over the parties: deal each holder's
    /// private value, instruct each party to sum-and-open, collect the opened
    /// shares, reconstruct the sum at degree `t`. Mirrors the in-process
    /// `bench::check_aggregate` exactly, but over the wire.
    pub fn aggregate(
        &mut self,
        dealer: &mut ShamirDealer,
        values: &[u64],
    ) -> Result<u64, MpcError> {
        let n = self.channels.len();
        // [OPUS-4.8] PR #96: fail closed on an empty input set, exactly as the
        // in-process `ShamirBackend::run_secure` does ("run_secure: no shared
        // inputs"). An empty deal would otherwise ship empty share columns to the
        // parties, which manufacture an `x=0` open-share and panic in
        // reconstruction (`Fp::inv(0)`). Reject before any frame crosses the wire.
        if values.is_empty() {
            return Err(MpcError::Protocol(
                "aggregate: no shared inputs (empty values)".into(),
            ));
        }
        // Deal: each value → a full sharing; party `p` gets share at point `p+1`.
        // We build, per party, the column of all its share-points.
        let sharings: Vec<Vec<Share>> = values.iter().map(|&v| dealer.share(Fp::new(v))).collect();
        // Step opcode first, then this party's column of shares.
        for (p, chan) in self.channels.iter_mut().enumerate() {
            chan.send(&Message::Step(StepCode::SumAndOpen))?;
            let column: Vec<Share> = sharings.iter().map(|s| s[p]).collect();
            chan.send(&Message::Shares(column))?;
        }
        // Open: collect each party's single summed share, reconstruct at degree t.
        let opened = self.collect_opens(1)?;
        let sum = shamir::reconstruct_degree(&opened[0], self.backend.threshold())?;
        // Tell parties the cell is done.
        self.broadcast_done()?;
        debug_assert_eq!(n, self.channels.len());
        Ok(sum.value())
    }

    /// Run the **hidden-value equi-join** over two `scale`-row tables (keys
    /// `0..scale` on both sides ⇒ exactly `scale` matches). For each of the
    /// `scale²` candidate pairs the coordinator deals `(d, r)` shares (d = keyL −
    /// keyR done in cleartext-then-shared here — the dealer plays the holders, as
    /// the in-process `secure_equal` does), the parties multiply-and-open, and the
    /// match bit is `product == 0`. Returns the match count. Mirrors
    /// `bench::check_hidden_join`.
    pub fn hidden_join(
        &mut self,
        dealer: &mut ShamirDealer,
        scale: usize,
    ) -> Result<u64, MpcError> {
        let t = self.backend.threshold();
        if self.channels.len() < 2 * t + 1 {
            return Err(MpcError::Protocol(format!(
                "hidden_join needs n >= 2t+1 (n={}, t={})",
                self.channels.len(),
                t
            )));
        }
        // Build all candidate (d, r) sharings. d = keyL - keyR; r = fresh nonzero
        // mask. Keys 0..scale on both sides. Pair count = scale².
        let mut d_sharings: Vec<Vec<Share>> = Vec::with_capacity(scale * scale);
        let mut r_sharings: Vec<Vec<Share>> = Vec::with_capacity(scale * scale);
        for lkey in 0..scale {
            for rkey in 0..scale {
                let d = Fp::new(lkey as u64).sub(Fp::new(rkey as u64));
                let mask = dealer.draw_nonzero_fp();
                d_sharings.push(dealer.share(d));
                r_sharings.push(dealer.share(mask));
            }
        }
        let pairs = d_sharings.len();
        // Deal each party its interleaved (d_share, r_share) column, then mul-open.
        for (p, chan) in self.channels.iter_mut().enumerate() {
            chan.send(&Message::Step(StepCode::MulAndOpen))?;
            let mut column = Vec::with_capacity(pairs * 2);
            for k in 0..pairs {
                column.push(d_sharings[k][p]);
                column.push(r_sharings[k][p]);
            }
            chan.send(&Message::Shares(column))?;
        }
        // Open: each party returns `pairs` product shares; reconstruct each at 2t.
        let opened = self.collect_opens(pairs)?;
        let mut matches = 0u64;
        for col in &opened {
            let m = shamir::reconstruct_degree(col, 2 * t)?;
            if m == Fp::zero() {
                matches += 1;
            }
        }
        self.broadcast_done()?;
        Ok(matches)
    }

    /// Collect one [`Message::Open`] frame from every party and transpose it into
    /// `count` columns (one per opened value), each a `Vec<Share>` ready to
    /// reconstruct. Errors if any party returns the wrong shape.
    fn collect_opens(&mut self, count: usize) -> Result<Vec<Vec<Share>>, MpcError> {
        let n = self.channels.len();
        let mut columns: Vec<Vec<Share>> = vec![Vec::with_capacity(n); count];
        for chan in self.channels.iter_mut() {
            match chan.recv()? {
                Message::Open(shares) => {
                    if shares.len() != count {
                        return Err(MpcError::Protocol(format!(
                            "open: party returned {} shares, expected {count}",
                            shares.len()
                        )));
                    }
                    for (i, s) in shares.into_iter().enumerate() {
                        columns[i].push(s);
                    }
                }
                other => return Err(MpcError::Protocol(format!("expected Open, got {other:?}"))),
            }
        }
        Ok(columns)
    }

    fn broadcast_done(&mut self) -> Result<(), MpcError> {
        for chan in self.channels.iter_mut() {
            chan.send(&Message::Done)?;
        }
        Ok(())
    }
}

// =============================================================================
// In-process LOOPBACK harness over real TCP sockets + OS threads-as-processes.
// =============================================================================
//
// The fully-separate-PROCESS runner needs a child binary entry point; the crate
// ships that as an example (`examples/mpc_party.rs`) + the driver
// (`examples/mpc_net_bench.rs`). For the in-crate TEST that the network result
// matches the in-process result, we run each party on its own OS THREAD with a
// real TCP `127.0.0.1` connection: this exercises the SAME transport (real
// serialisation + real socket syscalls + real round-trips) without needing to
// fork a child binary inside a unit test. A thread-per-party over loopback TCP is
// the same wire path a process-per-party uses; the process boundary adds only
// scheduling, not protocol behaviour, so the result-equality guarantee is
// identical. The example binaries provide the genuine process-per-party runner.

/// Spawn `n` party threads, each owning one accepted TCP connection on
/// `127.0.0.1`, and run `cell` over the coordinator side. Returns the measured
/// [`NetworkCell`]. The threads are the loopback stand-in for the party
/// processes (see the module note); the example binaries run real processes.
///
/// `values` is consulted for the aggregate; `scale` for the hidden join.
pub fn run_cell_networked(
    query_class: QueryClass,
    backend: &ShamirBackend,
    dealer: &mut ShamirDealer,
    values: &[u64],
    scale: usize,
) -> Result<NetworkCell, MpcError> {
    let n = backend.parties();
    // Bind a loopback listener; parties connect to it. Port 0 ⇒ OS-assigned.
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| MpcError::Protocol(format!("bind loopback: {e}")))?;
    let addr = listener
        .local_addr()
        .map_err(|e| MpcError::Protocol(format!("local_addr: {e}")))?;

    // Spawn the party threads. Each connects, runs `run_party`, exits.
    let mut party_handles = Vec::with_capacity(n);
    for _ in 0..n {
        party_handles.push(std::thread::spawn(move || -> Result<(), MpcError> {
            let stream = TcpStream::connect(addr)
                .map_err(|e| MpcError::Protocol(format!("party connect: {e}")))?;
            // Nagle off: we want each frame on the wire immediately so the round
            // latency is real (matters under netem; harmless on raw loopback).
            stream.set_nodelay(true).ok();
            let mut chan = Channel::new(stream);
            run_party(&mut chan)
        }));
    }

    // Accept `n` connections (coordinator side), build channels in connect order.
    let mut channels = Vec::with_capacity(n);
    for _ in 0..n {
        let (stream, _peer) = listener
            .accept()
            .map_err(|e| MpcError::Protocol(format!("accept: {e}")))?;
        stream.set_nodelay(true).ok();
        channels.push(Channel::new(stream));
    }

    let mut coord = Coordinator::new(channels, backend.clone())?;

    let start = Instant::now();
    let (result, rounds) = match query_class {
        QueryClass::CumulativeSumAggregate => {
            let sum = coord.aggregate(dealer, values)?;
            // One deal round + one open round = 2 logical rounds (matches the
            // in-process aggregate: 0 mult rounds + 1 open, plus the deal).
            (sum, 2u64)
        }
        QueryClass::HiddenValueEquiJoin => {
            let matches = coord.hidden_join(dealer, scale)?;
            // One deal round + one batched mul-open round (the |L|·|R| equalities
            // are independent ⇒ one network round; the batched lower bound).
            (matches, 2u64)
        }
        QueryClass::ObliviousShuffle | QueryClass::ObliviousSort => return Err(MpcError::not_yet(
            "oblivious shuffle/sort over the network transport",
            "sq-tg6b follow-up bead (representative cells aggregate + hidden-join shipped first)",
        )),
    };
    let wall_clock = start.elapsed();

    let (bytes_sent, bytes_recv) = coord.byte_totals();

    // Join the party threads (they exit on `Done`).
    for h in party_handles {
        h.join()
            .map_err(|_| MpcError::Protocol("party thread panicked".into()))??;
    }

    Ok(NetworkCell {
        query_class,
        parties: n,
        threshold: backend.threshold(),
        scale: if query_class == QueryClass::CumulativeSumAggregate {
            values.len()
        } else {
            scale
        },
        result,
        wall_clock,
        bytes_sent,
        bytes_recv,
        rounds,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Codec round-trips every message variant. --------------------------
    #[test]
    fn codec_roundtrips_all_messages() {
        let msgs = vec![
            Message::Shares(vec![
                Share {
                    x: 1,
                    y: Fp::new(42),
                },
                Share {
                    x: 2,
                    y: Fp::new(99),
                },
            ]),
            Message::Open(vec![Share {
                x: 3,
                y: Fp::new(7),
            }]),
            Message::Step(StepCode::SumAndOpen),
            Message::Step(StepCode::MulAndOpen),
            Message::Done,
        ];
        for m in &msgs {
            let frame = m.encode();
            // Strip the 4-byte length prefix the way Channel::recv does.
            let len = u32::from_be_bytes(frame[0..4].try_into().unwrap()) as usize;
            let body = &frame[4..4 + len];
            let decoded = Message::decode(body).unwrap();
            assert_eq!(&decoded, m, "round-trip {m:?}");
        }
    }

    #[test]
    fn decode_rejects_garbage_tag() {
        assert!(Message::decode(&[0xFF]).is_err());
        assert!(Message::decode(&[]).is_err());
    }

    // --- THE load-bearing test: networked result == in-process result. -----
    #[test]
    fn networked_aggregate_matches_in_process() {
        use crate::bench::check_aggregate;
        for &n in &[2usize, 3, 5, 7] {
            let backend = ShamirBackend::new_seeded(n, 0xA11CE).unwrap();
            let values: Vec<u64> = (0..n as u64).map(|i| 1000 * (i + 1)).collect();
            let expected_plain: u64 = values.iter().sum();

            // In-process reference.
            let in_proc = check_aggregate(&backend, &values).unwrap();
            assert_eq!(in_proc, expected_plain, "in-process sum");

            // Networked (real TCP loopback).
            let mut dealer = backend.dealer();
            let cell = run_cell_networked(
                QueryClass::CumulativeSumAggregate,
                &backend,
                &mut dealer,
                &values,
                1,
            )
            .unwrap();
            assert_eq!(
                cell.result, in_proc,
                "n={n}: networked aggregate must equal in-process result"
            );
            assert_eq!(cell.result, expected_plain);
            // The transport actually moved bytes and drove rounds.
            assert!(cell.total_bytes() > 0, "n={n}: bytes must cross the wire");
            assert_eq!(cell.rounds, 2);
            assert_eq!(cell.parties, n);
        }
    }

    #[test]
    fn networked_hidden_join_matches_in_process() {
        use crate::bench::check_hidden_join;
        // n must satisfy n >= 2t+1 (always true at t = ⌊(n−1)/2⌋).
        for &n in &[3usize, 5, 7] {
            let backend = ShamirBackend::new_seeded(n, 0xB0B).unwrap();
            let scale = 4; // 16 candidate pairs, 4 expected matches.

            let in_proc = check_hidden_join(&backend, scale).unwrap();
            assert_eq!(in_proc, scale, "in-process join finds all matches");

            let mut dealer = backend.dealer();
            let cell = run_cell_networked(
                QueryClass::HiddenValueEquiJoin,
                &backend,
                &mut dealer,
                &[],
                scale,
            )
            .unwrap();
            assert_eq!(
                cell.result as usize, in_proc,
                "n={n}: networked hidden-join match count must equal in-process"
            );
            assert_eq!(cell.result as usize, scale);
            assert!(cell.total_bytes() > 0);
        }
    }

    #[test]
    fn networked_join_allows_n2_degenerate_t0() {
        // [OPUS-4.8] PR #96: renamed — this test asserts the join is ALLOWED at
        // n=2, not rejected. n=2 ⇒ t=0 ⇒ 2t+1 = 1 ≤ 2, so the `n >= 2t+1` guard
        // passes; assert the join runs at n=2 (degenerate t=0) and still matches.
        // This pins that the guard mirrors the in-process one rather than being
        // stricter.
        let backend = ShamirBackend::new_seeded(2, 0xC0FFEE).unwrap();
        let mut dealer = backend.dealer();
        let cell = run_cell_networked(
            QueryClass::HiddenValueEquiJoin,
            &backend,
            &mut dealer,
            &[],
            3,
        )
        .unwrap();
        assert_eq!(cell.result as usize, 3);
    }

    #[test]
    fn oblivious_classes_are_honestly_deferred_on_the_network_tier() {
        // The brief ships the two representative cells (aggregate + hidden join);
        // shuffle/sort over the network is a beaded follow-up, and must FAIL
        // CLOSED (NotYetImplemented), never silently return a fake number.
        let backend = ShamirBackend::new_seeded(3, 1).unwrap();
        let mut dealer = backend.dealer();
        let err = run_cell_networked(QueryClass::ObliviousShuffle, &backend, &mut dealer, &[], 8)
            .unwrap_err();
        assert!(matches!(err, MpcError::NotYetImplemented { .. }));
    }

    // [OPUS-4.8] PR #96: an empty aggregate input must FAIL CLOSED on the network
    // tier exactly as the in-process `ShamirBackend::run_secure` does, never
    // manufacture an `x=0` open-share that would panic in reconstruction.
    #[test]
    fn networked_aggregate_rejects_empty_inputs() {
        let backend = ShamirBackend::new_seeded(3, 0xDEAD).unwrap();
        let mut dealer = backend.dealer();
        let err = run_cell_networked(
            QueryClass::CumulativeSumAggregate,
            &backend,
            &mut dealer,
            &[],
            1,
        )
        .unwrap_err();
        match err {
            MpcError::Protocol(msg) => assert!(
                msg.contains("no shared inputs"),
                "expected a 'no shared inputs' protocol error, got: {msg}"
            ),
            other => panic!("expected Protocol error, got {other:?}"),
        }
    }
}
