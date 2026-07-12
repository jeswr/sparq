// AUTHORED-BY Claude Fable 5
//! Fixed-N single-connection request driver for the Linux syscalls-per-request harness
//! (`bench/syscalls.sh` — beyond-50k §4 P0.1).
//!
//! ## What this is
//! `strace -fc` needs a *quiesced* server and a *fixed, integer* request count per scenario so the
//! resulting syscall table divides into a deterministic syscalls-per-request metric (the perf-gate
//! rule: integer counts hard-gate, wall-clock is advisory). `oha`/`auth_load` are throughput tools —
//! they run for a *duration*, use many connections, and cannot be attached-around precisely. This
//! driver instead:
//!
//! - drives **exactly N** requests of one class over **one** keep-alive HTTP/1.1 connection
//!   (`hyper::client::conn::http1` — no pool, no implicit reconnects; an unexpected reconnect is
//!   counted and reported);
//! - separates an untraced **warm-up** phase (JWKS fetch, token-cache fill, ACL-cache fill, listing
//!   render warm) from the **measured** phase, and *pauses between them* so the wrapper script can
//!   attach `strace`/`perf` to the already-warm server;
//! - **verifies** the responses (uniform expected status per class, and a preflight proving the
//!   private fixture really answers 401 anonymously) so a misbuilt token/proof can never silently
//!   measure the 401 path;
//! - needs **no Docker and no Keycloak**: it embeds a loopback **mock OIDC issuer** (discovery +
//!   JWKS over plain HTTP) and mints its own DPoP-bound RFC 9068 access tokens, exactly like
//!   `tests/common/mod.rs` does — the server is booted with `SOLID_SERVER_TRUSTED_ISSUER` pointing
//!   at the mock issuer and `SOLID_SERVER_ALLOW_LOOPBACK=1` (the same dev/IT escape hatch the
//!   conformance run uses). The server's verify path (discovery → JWKS fetch → at+jwt verify →
//!   DPoP proof verify → replay mark → WAC) is the real production code path throughout.
//!
//! ## Wrapper protocol (stdin/stdout, line-oriented)
//! For each scenario in `--scenarios`: the driver warms up, prints `READY <scenario>` on stdout,
//! and BLOCKS until it reads one line ("GO") on stdin. The wrapper attaches strace/perf to the
//! server in that gap. The driver then runs the N measured requests and prints
//! `DONE <scenario> <one-line JSON summary>`; the wrapper detaches the tracer and moves on.
//! Everything human-readable goes to stderr; stdout carries ONLY the protocol lines.
//!
//! ## Request classes
//! - `anon-doc` — anonymous `GET /bench/public/doc` (the auth-free read hot path; expects 200)
//! - `listing` — anonymous `GET /bench/listing/` (container membership render; expects 200)
//! - `authed-doc` — DPoP-authed `GET /bench/private/doc` (token-cache-hit verify path; expects 200)
//! - `put` — DPoP-authed `PUT /bench/private/put-target` of a small fixed Turtle body, repeatedly
//!   REPLACING the same resource (uniform update path — creating N new resources would grow
//!   container membership and skew later requests; the one 201 create happens in warm-up; expects
//!   204/200 measured)
//! - `idle` — no requests at all for `--idle-secs`: the background-noise baseline (timer/epoll
//!   wakeups) so per-request numbers can be judged against it.
//!
//! Fixtures come from the server's own bench seed (`SOLID_SERVER_SEED_BENCH`, `src/seed.rs`); the
//! authed scenarios mint tokens for the WebID the seeded owner-only ACLs grant — pass `--webid`
//! matching the server's `SOLID_SERVER_SEED_BENCH_OWNER` (an https: WebID is REQUIRED by the
//! verifier, so under a plain-http base the derived owner default cannot authenticate). The WebID
//! is never dereferenced (`SOLID_SERVER_BIDIRECTIONAL=off`, the same posture as `conformance/run.sh`).
//!
//! This binary changes NO server behaviour — it is a measurement client only.

use std::collections::BTreeMap;
use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::routing::get;
use axum::{Json, Router};
use base64::Engine as _;
use bytes::Bytes;
use http::{Method, Request};
use http_body_util::{BodyExt, Full};
use hyper::client::conn::http1::{self, SendRequest};
use hyper_util::rt::TokioIo;
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};
use rand_core::OsRng;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

// ---------------------------------------------------------------------------------------------
// Small JOSE/DPoP helpers (ES256) — mirroring tests/common/mod.rs / examples/auth_load.rs.
// ---------------------------------------------------------------------------------------------

fn b64url(bytes: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn b64url_json(v: &Value) -> String {
    b64url(serde_json::to_vec(v).expect("serializable JSON").as_slice())
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock after epoch")
        .as_secs() as i64
}

static JTI_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A per-process random nonce mixed into every jti so a re-run against a still-warm server never
/// collides with a prior run's jtis in the replay store (the auth_load lesson).
fn jti_nonce() -> &'static str {
    use std::sync::OnceLock;
    static NONCE: OnceLock<String> = OnceLock::new();
    NONCE.get_or_init(|| {
        let mut bytes = [0u8; 12];
        getrandom::getrandom(&mut bytes).expect("OS randomness for the jti nonce");
        b64url(&bytes)
    })
}

fn next_jti() -> String {
    let n = JTI_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", jti_nonce(), n)
}

/// An ES256 key pair + public JWK + RFC 7638 thumbprint (issuer signing key / DPoP proof key).
struct KeyKit {
    signing: SigningKey,
    public_jwk: Value,
    thumbprint: String,
}

impl KeyKit {
    fn generate() -> Self {
        Self::from_signing(SigningKey::random(&mut OsRng))
    }

    /// A DETERMINISTIC key derived from a seed string (SHA-256 → the P-256 scalar). Used for the
    /// mock ISSUER key so it is IDENTICAL across the harness's two driver processes (strace pass +
    /// perf pass). The server caches an issuer's JWKS for `SOLID_SERVER_JWKS_CACHE_TTL_SECS` (raised
    /// to 24h so no refetch lands mid-window); with a fresh random issuer key per process, the
    /// second process's tokens are signed by a key the server won't refetch → `InvalidSignature`.
    /// A shared seed makes both processes present the SAME JWKS, so the cached keys validate both.
    /// (The DPoP PROOF key can stay per-process random: token+proof are minted by the SAME process
    /// and the token's `cnf.jkt` binds them, so they are always internally consistent.)
    fn from_seed(seed: &str) -> Self {
        // SHA-256(seed) as the scalar; on the astronomically unlikely invalid-scalar case, re-hash
        // with a fixed suffix. from_bytes rejects zero / >= curve order.
        let mut material: [u8; 32] = Sha256::digest(seed.as_bytes()).into();
        loop {
            if let Ok(signing) = SigningKey::from_bytes((&material).into()) {
                return Self::from_signing(signing);
            }
            material = Sha256::digest([&material[..], b"retry"].concat()).into();
        }
    }

    fn from_signing(signing: SigningKey) -> Self {
        let verifying: VerifyingKey = *signing.verifying_key();
        let point = verifying.to_encoded_point(false);
        let x = b64url(point.x().expect("uncompressed point has x"));
        let y = b64url(point.y().expect("uncompressed point has y"));
        let public_jwk = json!({ "kty": "EC", "crv": "P-256", "x": x, "y": y });
        let canonical = format!(r#"{{"crv":"P-256","kty":"EC","x":"{x}","y":"{y}"}}"#);
        let thumbprint = b64url(&Sha256::digest(canonical.as_bytes()));
        Self {
            signing,
            public_jwk,
            thumbprint,
        }
    }

    fn sign(&self, header: &Value, claims: &Value) -> String {
        let signing_input = format!("{}.{}", b64url_json(header), b64url_json(claims));
        let sig: Signature = self.signing.sign(signing_input.as_bytes());
        format!("{signing_input}.{}", b64url(&sig.to_bytes()))
    }
}

/// Mint an RFC 9068 `at+jwt` access token: `iss`=the mock issuer, `aud`=the server base URL,
/// `webid`=the bench fixture owner, `cnf.jkt`-bound to the DPoP proof key. `exp_secs` is generous
/// (the strace-slowed measured window must not outlive the token — an expiry mid-run would flip
/// the measured path to 401 and the driver would abort on the status check).
fn mint_access_token(
    issuer_key: &KeyKit,
    cnf_jkt: &str,
    issuer: &str,
    audience: &str,
    webid: &str,
    exp_secs: i64,
) -> String {
    let header = json!({ "alg": "ES256", "typ": "at+jwt" });
    let iat = unix_now();
    let claims = json!({
        "iss": issuer,
        "sub": webid,
        "jti": format!("at-{}", next_jti()),
        "client_id": "syscall-bench",
        "aud": audience,
        "webid": webid,
        "cnf": { "jkt": cnf_jkt },
        "iat": iat,
        "exp": iat + exp_secs,
    });
    issuer_key.sign(&header, &claims)
}

fn ath(token: &str) -> String {
    b64url(&Sha256::digest(token.as_bytes()))
}

/// A fresh DPoP proof (unique jti) for `method`+`url`, `ath`-bound to `access_token`.
fn mint_dpop_proof(client_key: &KeyKit, method: &str, url: &str, access_token: &str) -> String {
    let header = json!({ "alg": "ES256", "typ": "dpop+jwt", "jwk": client_key.public_jwk });
    let claims = json!({
        "htm": method,
        "htu": url,
        "jti": next_jti(),
        "iat": unix_now(),
        "ath": ath(access_token),
    });
    client_key.sign(&header, &claims)
}

// ---------------------------------------------------------------------------------------------
// The embedded mock OIDC issuer (loopback plain-HTTP discovery + JWKS).
// ---------------------------------------------------------------------------------------------

/// Serve `<issuer>/.well-known/openid-configuration` + `<issuer>/jwks` for the given issuer key.
/// The server's `NetworkJwksProvider` requires the discovery doc's `issuer` to EQUAL the configured
/// trusted issuer (RFC 8414 mix-up defence), so the issuer string must match
/// `SOLID_SERVER_TRUSTED_ISSUER` exactly — `http://127.0.0.1:<port>`.
async fn start_mock_issuer(port: u16, issuer_key: &KeyKit) -> Result<String, Box<dyn Error>> {
    let issuer = format!("http://127.0.0.1:{port}");
    let discovery = json!({
        "issuer": issuer,
        "jwks_uri": format!("{issuer}/jwks"),
    });
    let jwks = json!({ "keys": [issuer_key.public_jwk] });
    let app = Router::new()
        .route(
            "/.well-known/openid-configuration",
            get(move || {
                let doc = discovery.clone();
                async move { Json(doc) }
            }),
        )
        .route(
            "/jwks",
            get(move || {
                let doc = jwks.clone();
                async move { Json(doc) }
            }),
        );
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port)).await?;
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("mock issuer serve error: {e}");
        }
    });
    Ok(issuer)
}

// ---------------------------------------------------------------------------------------------
// The single keep-alive HTTP/1.1 connection.
// ---------------------------------------------------------------------------------------------

struct Conn {
    sender: SendRequest<Full<Bytes>>,
    addr: String,
    host: String,
    reconnects: u64,
    /// The last response's `WWW-Authenticate` challenge, kept for preflight diagnostics (WHY a
    /// 401 happened — token vs proof vs WAC — without re-deriving it).
    last_challenge: Option<String>,
}

impl Conn {
    async fn open(addr: &str, host: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            sender: Self::dial(addr).await?,
            addr: addr.to_string(),
            host: host.to_string(),
            reconnects: 0,
            last_challenge: None,
        })
    }

    async fn dial(addr: &str) -> Result<SendRequest<Full<Bytes>>, Box<dyn Error>> {
        let stream = tokio::net::TcpStream::connect(addr).await?;
        // NODELAY on the client so each request arrives as one segment — keeps the server-side
        // read pattern uniform (no Nagle-delayed split requests).
        stream.set_nodelay(true)?;
        let (sender, conn) = http1::handshake(TokioIo::new(stream)).await?;
        tokio::spawn(async move {
            // Drive the connection; an error here surfaces as a send_request failure.
            let _ = conn.await;
        });
        Ok(sender)
    }

    /// One request → (status, response body bytes). Reconnects at most once on a dead connection
    /// (counted — a nonzero count is reported and taints the single-connection premise).
    async fn request(
        &mut self,
        method: &Method,
        path: &str,
        auth: Option<(&str, &str)>,          // (access_token, dpop_proof)
        body: Option<(&'static str, Bytes)>, // (content-type, body)
    ) -> Result<(u16, u64), Box<dyn Error>> {
        for attempt in 0..2 {
            let mut builder = Request::builder()
                .method(method.clone())
                .uri(path)
                .header(http::header::HOST, &self.host)
                .header(http::header::ACCEPT, "text/turtle");
            if let Some((token, proof)) = auth {
                builder = builder
                    .header(http::header::AUTHORIZATION, format!("DPoP {token}"))
                    .header("dpop", proof);
            }
            if let Some((ctype, _)) = &body {
                builder = builder.header(http::header::CONTENT_TYPE, *ctype);
            }
            let payload = body.as_ref().map(|(_, b)| b.clone()).unwrap_or_default();
            let req = builder.body(Full::new(payload))?;

            match self.sender.send_request(req).await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    self.last_challenge = resp
                        .headers()
                        .get(http::header::WWW_AUTHENTICATE)
                        .and_then(|v| v.to_str().ok())
                        .map(str::to_string);
                    let mut incoming = resp.into_body();
                    let mut nbytes: u64 = 0;
                    while let Some(frame) = incoming.frame().await {
                        let frame = frame?;
                        if let Some(data) = frame.data_ref() {
                            nbytes += data.len() as u64;
                        }
                    }
                    return Ok((status, nbytes));
                }
                Err(e) if attempt == 0 => {
                    eprintln!("connection error ({e}); reconnecting once");
                    self.reconnects += 1;
                    self.sender = Self::dial(&self.addr).await?;
                }
                Err(e) => return Err(Box::new(e)),
            }
        }
        unreachable!("two attempts always return");
    }
}

// ---------------------------------------------------------------------------------------------
// Scenarios.
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Class {
    AnonDoc,
    Listing,
    AuthedDoc,
    Put,
    Idle,
}

impl Class {
    fn parse(s: &str) -> Result<Self, String> {
        match s {
            "anon-doc" => Ok(Self::AnonDoc),
            "listing" => Ok(Self::Listing),
            "authed-doc" => Ok(Self::AuthedDoc),
            "put" => Ok(Self::Put),
            "idle" => Ok(Self::Idle),
            other => Err(format!("unknown scenario '{other}'")),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::AnonDoc => "anon-doc",
            Self::Listing => "listing",
            Self::AuthedDoc => "authed-doc",
            Self::Put => "put",
            Self::Idle => "idle",
        }
    }

    fn path(self) -> &'static str {
        match self {
            Self::AnonDoc => "/bench/public/doc",
            Self::Listing => "/bench/listing/",
            Self::AuthedDoc => "/bench/private/doc",
            Self::Put => "/bench/private/put-target",
            Self::Idle => "",
        }
    }

    fn method(self) -> Method {
        match self {
            Self::Put => Method::PUT,
            _ => Method::GET,
        }
    }

    fn authed(self) -> bool {
        matches!(self, Self::AuthedDoc | Self::Put)
    }

    /// Statuses acceptable in the MEASURED phase (must also be uniform across the phase).
    fn expected(self) -> &'static [u16] {
        match self {
            Self::AnonDoc | Self::Listing | Self::AuthedDoc => &[200],
            // First-ever PUT creates (201) — absorbed by warm-up; measured PUTs replace.
            Self::Put => &[200, 204, 201],
            Self::Idle => &[],
        }
    }
}

/// The small fixed Turtle body every measured PUT carries (constant bytes ⇒ uniform request size).
const PUT_BODY: &str =
    "<> <http://www.w3.org/2000/01/rdf-schema#label> \"syscall bench put target\" .\n";

struct Args {
    base: String,
    issuer_port: u16,
    scenarios: Vec<Class>,
    n: u64,
    warmup: u64,
    idle_secs: u64,
    token_exp_secs: i64,
    /// The WebID minted into the tokens. Must equal the WebID the seeded owner-only ACLs grant —
    /// i.e. `SOLID_SERVER_SEED_BENCH_OWNER` when the server serves plain HTTP (the verifier
    /// requires an https: webid claim, so the derived http: owner can never authenticate).
    webid: Option<String>,
    /// A seed for the DETERMINISTIC mock-issuer key. When two harness passes each spawn a driver
    /// process against a JWKS-caching server, they MUST share the issuer key — pass the same seed.
    /// Unset ⇒ a random per-process issuer key (fine for a single-pass run).
    issuer_seed: Option<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut base = "http://127.0.0.1:3400".to_string();
    let mut issuer_port: u16 = 3401;
    let mut scenarios = vec![
        Class::Idle,
        Class::AnonDoc,
        Class::Listing,
        Class::AuthedDoc,
        Class::Put,
    ];
    let mut n: u64 = 5000;
    let mut warmup: u64 = 200;
    let mut idle_secs: u64 = 5;
    let mut token_exp_secs: i64 = 3600;
    let mut webid: Option<String> = None;
    let mut issuer_seed: Option<String> = None;

    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < argv.len() {
        let flag = argv[i].as_str();
        let val = argv
            .get(i + 1)
            .ok_or_else(|| format!("{flag} needs a value"))?;
        match flag {
            "--base" => base = val.trim_end_matches('/').to_string(),
            "--issuer-port" => issuer_port = val.parse().map_err(|e| format!("{flag}: {e}"))?,
            "--scenarios" => {
                scenarios = val
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(Class::parse)
                    .collect::<Result<_, _>>()?;
            }
            "--n" => n = val.parse().map_err(|e| format!("{flag}: {e}"))?,
            "--warmup" => warmup = val.parse().map_err(|e| format!("{flag}: {e}"))?,
            "--idle-secs" => idle_secs = val.parse().map_err(|e| format!("{flag}: {e}"))?,
            "--token-exp-secs" => {
                token_exp_secs = val.parse().map_err(|e| format!("{flag}: {e}"))?
            }
            "--webid" => webid = Some(val.clone()),
            "--issuer-seed" => issuer_seed = Some(val.clone()),
            other => return Err(format!("unknown flag '{other}'")),
        }
        i += 2;
    }
    if n == 0 {
        return Err("--n must be >= 1".into());
    }
    Ok(Args {
        base,
        issuer_port,
        scenarios,
        n,
        warmup,
        idle_secs,
        token_exp_secs,
        webid,
        issuer_seed,
    })
}

/// host:port from an http:// base URL (plain HTTP only — the syscall harness runs without TLS so
/// the counted syscalls are the server's own socket pattern, not rustls record framing; the TLS
/// delta is a separate follow-up measurement).
fn host_port(base: &str) -> Result<String, String> {
    base.strip_prefix("http://")
        .map(|s| s.trim_end_matches('/').to_string())
        .ok_or_else(|| format!("--base must be a plain http:// URL, got {base}"))
}

/// Print `READY <scenario>` and block until one line ("GO") arrives on stdin. The blocking read
/// runs on the blocking pool (tokio's `io-std` feature is not enabled for this crate), so the
/// current-thread runtime — which also serves the mock issuer — stays responsive.
async fn wait_go(scenario: &str) -> Result<(), Box<dyn Error>> {
    println!("READY {scenario}");
    let got = tokio::task::spawn_blocking(|| {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).map(|n| n > 0)
    })
    .await??;
    if got {
        Ok(())
    } else {
        Err(format!("stdin closed while waiting for GO before '{scenario}'").into())
    }
}

struct ScenarioOutcome {
    statuses: BTreeMap<u16, u64>,
    bytes_total: u64,
    reconnects: u64,
}

async fn drive_fixed_n(
    conn: &mut Conn,
    class: Class,
    base: &str,
    auth_material: Option<(&str, &KeyKit)>, // (access_token, proof key)
    count: u64,
) -> Result<ScenarioOutcome, Box<dyn Error>> {
    let url = format!("{base}{}", class.path());
    let method = class.method();
    let mut statuses: BTreeMap<u16, u64> = BTreeMap::new();
    let mut bytes_total: u64 = 0;
    let reconnects_before = conn.reconnects;
    for _ in 0..count {
        let proof = auth_material
            .as_ref()
            .map(|(token, key)| (*token, mint_dpop_proof(key, method.as_str(), &url, token)));
        let auth = proof.as_ref().map(|(t, p)| (*t, p.as_str()));
        let body = match class {
            Class::Put => Some(("text/turtle", Bytes::from_static(PUT_BODY.as_bytes()))),
            _ => None,
        };
        let (status, nbytes) = conn.request(&method, class.path(), auth, body).await?;
        *statuses.entry(status).or_insert(0) += 1;
        bytes_total += nbytes;
    }
    Ok(ScenarioOutcome {
        statuses,
        bytes_total,
        reconnects: conn.reconnects - reconnects_before,
    })
}

/// Fail unless every observed status is in `expected` AND the measured phase is uniform (a single
/// status value) — mixed statuses mean the run measured more than one code path.
fn assert_uniform_expected(
    scenario: &str,
    phase: &str,
    outcome: &ScenarioOutcome,
    expected: &[u16],
    require_uniform: bool,
) -> Result<(), Box<dyn Error>> {
    for status in outcome.statuses.keys() {
        if !expected.contains(status) {
            return Err(format!(
                "{scenario} {phase}: unexpected status {status} (statuses: {:?})",
                outcome.statuses
            )
            .into());
        }
    }
    if require_uniform && outcome.statuses.len() > 1 {
        return Err(format!(
            "{scenario} {phase}: non-uniform statuses {:?} — refusing to average across two code paths",
            outcome.statuses
        )
        .into());
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args().map_err(|e| -> Box<dyn Error> { e.into() })?;
    let addr = host_port(&args.base).map_err(|e| -> Box<dyn Error> { e.into() })?;
    let host = addr.clone();

    // Keys + the mock issuer. One issuer key + one DPoP proof key for the whole process; the access
    // token is re-minted PER SCENARIO (fresh exp) so a long traced window never straddles expiry.
    let issuer_key = match args.issuer_seed.as_deref() {
        Some(seed) => KeyKit::from_seed(seed),
        None => KeyKit::generate(),
    };
    let proof_key = KeyKit::generate();
    let needs_auth = args.scenarios.iter().any(|c| c.authed());
    let issuer = if needs_auth {
        Some(start_mock_issuer(args.issuer_port, &issuer_key).await?)
    } else {
        None
    };
    let webid = args
        .webid
        .clone()
        .unwrap_or_else(|| format!("{}/bench/profile/card#me", args.base));
    let mint = |exp: i64| -> Option<String> {
        issuer.as_ref().map(|iss| {
            mint_access_token(
                &issuer_key,
                &proof_key.thumbprint,
                iss,
                &args.base,
                &webid,
                exp,
            )
        })
    };

    // ---- Preflight: prove each fixture answers the intended code path BEFORE measuring. --------
    // - anon GET public doc  → 200 (fixture exists, world-readable)
    // - anon GET private doc → 401 (genuinely private — a broken ACL would silently flip the
    //   authed scenario to measuring an already-authorized-public path)
    // - authed GET private doc → 200 (token + proof + WAC actually work)
    // - authed PUT put-target  → 201/204 (write path works; absorbs the one-off create)
    {
        let mut conn = Conn::open(&addr, &host).await?;
        let (st, _) = conn
            .request(&Method::GET, Class::AnonDoc.path(), None, None)
            .await?;
        if st != 200 {
            return Err(format!(
                "preflight: anon GET {} → {st}, want 200 (is the server bench-seeded?)",
                Class::AnonDoc.path()
            )
            .into());
        }
        let (st, _) = conn
            .request(&Method::GET, Class::AuthedDoc.path(), None, None)
            .await?;
        if st != 401 {
            return Err(format!(
                "preflight: anon GET {} → {st}, want 401 (private fixture is not private!)",
                Class::AuthedDoc.path()
            )
            .into());
        }
        if needs_auth {
            let token = mint(300).expect("issuer running");
            let url = format!("{}{}", args.base, Class::AuthedDoc.path());
            let proof = mint_dpop_proof(&proof_key, "GET", &url, &token);
            let (st, _) = conn
                .request(
                    &Method::GET,
                    Class::AuthedDoc.path(),
                    Some((&token, &proof)),
                    None,
                )
                .await?;
            if st != 200 {
                return Err(format!(
                    "preflight: authed GET {} → {st}, want 200 (token/proof/WAC broken — refusing to measure the 401 path); WWW-Authenticate: {}",
                    Class::AuthedDoc.path(),
                    conn.last_challenge.as_deref().unwrap_or("<none>")
                )
                .into());
            }
            let put_url = format!("{}{}", args.base, Class::Put.path());
            let proof = mint_dpop_proof(&proof_key, "PUT", &put_url, &token);
            let (st, _) = conn
                .request(
                    &Method::PUT,
                    Class::Put.path(),
                    Some((&token, &proof)),
                    Some(("text/turtle", Bytes::from_static(PUT_BODY.as_bytes()))),
                )
                .await?;
            if !Class::Put.expected().contains(&st) {
                return Err(format!(
                    "preflight: authed PUT {} → {st}, want 201/204/200",
                    Class::Put.path()
                )
                .into());
            }
        }
        eprintln!(
            "preflight OK (public=200, private-anon=401{})",
            if needs_auth {
                ", private-authed=200, put=ok"
            } else {
                ""
            }
        );
    }

    // ---- Scenario loop (READY / GO / DONE protocol with the wrapper). --------------------------
    for (idx, class) in args.scenarios.iter().copied().enumerate() {
        let scenario = class.name();

        if class == Class::Idle {
            wait_go(scenario).await?;
            tokio::time::sleep(std::time::Duration::from_secs(args.idle_secs)).await;
            let summary = json!({
                "scenario": scenario, "index": idx, "idle_secs": args.idle_secs,
            });
            println!("DONE {scenario} {summary}");
            continue;
        }

        // Fresh connection + (for authed classes) a fresh token per scenario. The connection is
        // opened and warmed BEFORE the tracer attaches, so accept4/TLS-free handshake + JWKS +
        // token-cache fill all land in the untraced warm-up.
        let mut conn = Conn::open(&addr, &host).await?;
        let token = if class.authed() {
            mint(args.token_exp_secs)
        } else {
            None
        };
        let auth_material = token.as_deref().map(|t| (t, &proof_key));

        let warm = drive_fixed_n(&mut conn, class, &args.base, auth_material, args.warmup).await?;
        assert_uniform_expected(scenario, "warm-up", &warm, class.expected(), false)?;

        wait_go(scenario).await?;
        let measured = drive_fixed_n(&mut conn, class, &args.base, auth_material, args.n).await?;
        assert_uniform_expected(scenario, "measured", &measured, class.expected(), true)?;

        let statuses_json: BTreeMap<String, u64> = measured
            .statuses
            .iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect();
        let summary = json!({
            "scenario": scenario,
            "index": idx,
            "n": args.n,
            "warmup": args.warmup,
            "statuses": statuses_json,
            "bytes_total": measured.bytes_total,
            "bytes_per_request": measured.bytes_total as f64 / args.n as f64,
            "reconnects": measured.reconnects,
        });
        println!("DONE {scenario} {summary}");
        if measured.reconnects > 0 {
            eprintln!("WARNING: {scenario} needed {} reconnect(s) — single-connection premise tainted; treat this scenario's accept/connect syscalls as noise", measured.reconnects);
        }
    }

    eprintln!("all scenarios complete");
    Ok(())
}
