//! `sparq-e2ee-ng-brokerd` — the reference **opaque broker** daemon.
//!
//! A thin transport around [`sparq_e2ee_ng_broker::Broker`]: a plain `std` TCP
//! listener, one thread per connection, and length-prefixed deterministic-CBOR
//! frames (a 4-byte big-endian length, then the frame).
//!
//! ```text
//! sparq-e2ee-ng-brokerd [--listen ADDR] [--max-block-bytes N] [--max-message-bytes N]
//!                       [--unpinned-ttl-secs N] [--max-topic-bytes N]
//!                       [--allow-clear-headers] [--log|--quiet]
//! ```
//!
//! **Honesty boundary.** Research-grade and externally unaudited (`sq-qhy4`).
//! This daemon implements **no transport authentication and no TLS**: run it
//! behind an authenticated, encrypted transport. It is an opaque store-and-route
//! service — it never holds a key, never decrypts a block, and never sees a
//! SPARQL query — but per design §5 it still observes topic membership,
//! subscription/publication patterns, timing, sizes, and storage volume, and it
//! is not trusted for integrity or availability.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use sparq_e2ee_ng_broker::broker::{Broker, BrokerConfig, SessionId};
use sparq_e2ee_ng_broker::log::{LogRecord, MetadataLog, StderrLog};
use sparq_e2ee_ng::broker_protocol::HeaderMode;

/// The operator-selected log sink. An enum rather than a trait object so the
/// closed set of sinks stays visible at the call site.
enum OperatorLog {
    Null,
    Stderr(StderrLog),
}

impl MetadataLog for OperatorLog {
    fn record(&mut self, r: &LogRecord) {
        match self {
            OperatorLog::Null => {}
            OperatorLog::Stderr(s) => s.record(r),
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

struct Args {
    listen: String,
    cfg: BrokerConfig,
    log: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut listen = "127.0.0.1:9425".to_string();
    let mut cfg = BrokerConfig::default();
    let mut log = true;
    let mut argv = std::env::args().skip(1);
    while let Some(a) = argv.next() {
        let mut value = || argv.next().ok_or_else(|| format!("{} needs a value", a));
        match a.as_str() {
            "--listen" => listen = value()?,
            "--max-block-bytes" => {
                cfg.limits.max_block_bytes = value()?.parse().map_err(|_| "bad number")?
            }
            "--max-message-bytes" => {
                cfg.limits.max_message_bytes = value()?.parse().map_err(|_| "bad number")?
            }
            "--max-ids-per-request" => {
                cfg.limits.max_ids_per_request = value()?.parse().map_err(|_| "bad number")?
            }
            "--unpinned-ttl-secs" => {
                cfg.retention.unpinned_ttl_secs = value()?.parse().map_err(|_| "bad number")?
            }
            "--max-topic-bytes" => {
                cfg.retention.max_topic_bytes = value()?.parse().map_err(|_| "bad number")?
            }
            // Opting in to clear routing headers means opting in to the broker
            // learning the commit DAG shape (design §5). It is never the default.
            "--allow-clear-headers" => cfg.header_modes.push(HeaderMode::Clear),
            "--log" => log = true,
            "--quiet" => log = false,
            "--help" | "-h" => return Err("help".to_string()),
            other => return Err(format!("unknown argument {}", other)),
        }
    }
    Ok(Args { listen, cfg, log })
}

const USAGE: &str = "\
sparq-e2ee-ng-brokerd — opaque broker for the sparq E2EE-NG profile

  --listen ADDR              bind address (default 127.0.0.1:9425; use :0 for an ephemeral port)
  --max-block-bytes N        largest accepted block ciphertext
  --max-message-bytes N      largest accepted frame
  --max-ids-per-request N    largest identifier list per request
  --unpinned-ttl-secs N      age after which an unpinned block may be collected
  --max-topic-bytes N        per-topic storage budget
  --allow-clear-headers      also accept clear-header mode (reveals the commit DAG to the broker)
  --log | --quiet            metadata-safe logging on stderr (default on)

RESEARCH-GRADE, externally UNAUDITED (sq-qhy4). No transport authentication, no TLS.
";

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            if e == "help" {
                print!("{}", USAGE);
                return;
            }
            eprintln!("sparq-e2ee-ng-brokerd: {}", e);
            eprintln!("{}", USAGE);
            std::process::exit(2);
        }
    };

    let listener = match TcpListener::bind(&args.listen) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("sparq-e2ee-ng-brokerd: cannot bind {}: {}", args.listen, e);
            std::process::exit(1);
        }
    };
    // Printed so a caller that bound port 0 can discover the ephemeral port.
    match listener.local_addr() {
        Ok(addr) => println!("listening {}", addr),
        Err(e) => {
            eprintln!("sparq-e2ee-ng-brokerd: cannot read local address: {}", e);
            std::process::exit(1);
        }
    }
    let _ = std::io::stdout().flush();

    let max_message_bytes = args.cfg.limits.max_message_bytes;
    let sink = if args.log {
        OperatorLog::Stderr(StderrLog)
    } else {
        OperatorLog::Null
    };
    let broker = Arc::new(Mutex::new(Broker::new(args.cfg, sink)));

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        // Opportunistic retention pass on each new connection: the broker is
        // clock-free, so the daemon is the only thing that decides "now".
        if let Ok(mut b) = broker.lock() {
            b.collect_garbage(now_secs());
        }
        let broker = Arc::clone(&broker);
        std::thread::spawn(move || {
            let session = match broker.lock() {
                Ok(mut b) => b.open_session(),
                Err(_) => return,
            };
            serve(&broker, session, stream, max_message_bytes);
            if let Ok(mut b) = broker.lock() {
                b.close_session(session);
            }
        });
    }
}

fn serve(
    broker: &Arc<Mutex<Broker<OperatorLog>>>,
    session: SessionId,
    mut stream: TcpStream,
    max_message_bytes: u64,
) {
    loop {
        let mut len = [0u8; 4];
        if stream.read_exact(&mut len).is_err() {
            return;
        }
        let n = u32::from_be_bytes(len) as u64;
        if n > max_message_bytes {
            // Refuse to allocate for an over-large declared frame; the peer has
            // already violated the advertised ceiling, so drop the connection.
            return;
        }
        let mut frame = vec![0u8; n as usize];
        if stream.read_exact(&mut frame).is_err() {
            return;
        }
        let (reply, pushes) = {
            let Ok(mut b) = broker.lock() else { return };
            let reply = b.handle_frame(session, now_secs(), &frame);
            let pushes = b.take_pushes(session);
            (reply, pushes)
        };
        if write_frame(&mut stream, &reply).is_err() {
            return;
        }
        for p in pushes {
            // Unsolicited fan-out echoes request_id 0.
            if write_frame(&mut stream, &p.encode(0)).is_err() {
                return;
            }
        }
    }
}

fn write_frame(stream: &mut TcpStream, bytes: &[u8]) -> std::io::Result<()> {
    let len = u32::try_from(bytes.len())
        .map_err(|_| std::io::Error::other("frame exceeds u32 length prefix"))?;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(bytes)?;
    stream.flush()
}
