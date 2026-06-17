//! SERVICE federation egress allowlist — server configuration. [OPUS-4.8] (sq-4w18)
//!
//! SPARQL 1.1 federated query (`SERVICE <iri> { … }`) turns attacker-controlled
//! query text into an OUTBOUND HTTP request from the server host — a textbook SSRF
//! primitive (the worst case being the cloud-metadata endpoint
//! `169.254.169.254`). `sparq-server` is the network-exposed surface, so it locks
//! federation down by default and makes the operator opt specific hosts back in.
//!
//! ## Default policy: DENY ALL SERVICE
//!
//! With the `service` cargo feature on but NO allowlist configured, **every**
//! `SERVICE` clause is refused before any network call — federation is effectively
//! off. Rationale: the server accepts queries from the network, and a SERVICE clause
//! is a request to dial an arbitrary endpoint *from inside the deployment's network
//! boundary*; defaulting to "reach nothing" is the only safe posture for an
//! unauthenticated, network-facing process. An operator enables federation by
//! listing the hosts it is allowed to reach; nothing else becomes reachable. This
//! mirrors the server's other fail-closed defaults (loopback-only bind unless
//! `--allow-remote`; `LOAD file://` gated behind a base).
//!
//! This is STRICTER than the engine's standalone default (which allows public IPs
//! and only blocks private/internal ones): on the server, a host must be on the
//! allowlist to be reached *at all*, even a public one. The strictness is wired via
//! `sparq_engine::with_service_egress_policy` with `strict = true`.
//!
//! ## Allowlist syntax
//!
//! Each entry is one of:
//!   * an **exact host** — `sparql.example.org`, `192.0.2.10`, `localhost`
//!     (matched case-insensitively against the SERVICE IRI authority);
//!   * a **suffix wildcard** — `*.example.org`, matching the apex `example.org` and
//!     any subdomain (`a.example.org`, `a.b.example.org`) but not `notexample.org`.
//!
//! Entries come from (union of all three, deduplicated):
//!   * the repeatable `--service-allow <entry>` CLI flag;
//!   * `--service-allow-file <path>` (one entry per line; `#` comments + blanks
//!     ignored);
//!   * the `SPARQ_SERVICE_ALLOW` env var (comma- and/or whitespace-separated).
//!
//! Precedence: env establishes a baseline, CLI flags ADD to it (the union — an
//! allowlist is additive, so there is no "override" that could silently *remove* a
//! host the env granted; CLI only ever widens). The file is loaded at startup and
//! merged in too.

use std::collections::BTreeSet;

/// The configured SERVICE egress allowlist. Empty = deny ALL SERVICE (the default).
///
/// Stored as a normalised, deduplicated, sorted set so two configs built from the
/// same inputs in any order compare equal (handy for tests) and the startup log is
/// deterministic. Each stored entry is already lower-cased and validated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServiceAllowlist {
    /// Exact host entries (`sparql.example.org`).
    exact: BTreeSet<String>,
    /// Suffix-wildcard entries stored in the engine's leading-dot form
    /// (`*.example.org` -> `.example.org`).
    suffix: BTreeSet<String>,
}

impl ServiceAllowlist {
    /// True when no host is allowlisted — i.e. SERVICE federation is fully denied.
    pub fn is_empty(&self) -> bool {
        self.exact.is_empty() && self.suffix.is_empty()
    }

    /// Number of distinct entries (exact + suffix).
    pub fn len(&self) -> usize {
        self.exact.len() + self.suffix.len()
    }

    /// Adds one raw entry (CLI flag value, file line, or env token). Returns an
    /// error string for a malformed entry so the caller can fail fast at startup
    /// rather than silently dropping a host the operator meant to allow.
    ///
    /// Accepted forms: a bare host, or `*.suffix` (suffix wildcard). The entry is
    /// lower-cased; surrounding whitespace is trimmed. A `*` anywhere other than a
    /// leading `*.` is rejected (we deliberately support only the simple
    /// apex-and-subdomain suffix wildcard, not arbitrary globs).
    ///
    /// [OPUS-4.8] sq-4w18: exact-host entries are NORMALISED to the bare host the
    /// engine actually compares against (see `Self::normalize_host`), so a copy-
    /// pasted endpoint like `example.org:443` or a bracketed IPv6 `[::1]` matches
    /// instead of silently never matching (which would make the allowlist look
    /// configured but be inert — a fail-open footgun).
    pub fn add(&mut self, raw: &str) -> Result<(), String> {
        let e = raw.trim().to_ascii_lowercase();
        if e.is_empty() {
            return Ok(()); // blank line / empty token — nothing to add
        }
        if let Some(suffix) = e.strip_prefix("*.") {
            if suffix.is_empty() || suffix.contains('*') {
                return Err(format!("invalid SERVICE allow pattern {raw:?}: expected '*.<suffix>'"));
            }
            // Engine form: leading dot. Matches the apex + any subdomain.
            self.suffix.insert(format!(".{suffix}"));
            return Ok(());
        }
        if e.contains('*') {
            return Err(format!(
                "invalid SERVICE allow entry {raw:?}: '*' is only supported as a leading '*.' suffix wildcard"
            ));
        }
        self.exact.insert(Self::normalize_host(&e));
        Ok(())
    }

    /// [OPUS-4.8] sq-4w18: normalise an exact-host entry to the EXACT string the
    /// engine's egress resolver compares the SERVICE IRI authority against.
    ///
    /// The engine ([`sparq_engine`]'s `EgressFilterResolver`) is handed `netloc` as
    /// `host:port` (IPv6 bracketed: `[::1]:80`) and derives the bare host by
    /// (1) dropping a trailing `:port` via `rsplit_once(':')` and (2) stripping a
    /// surrounding `[ ]`. An allowlist entry that still carries a port or brackets
    /// would therefore NEVER equal what the engine looks up. We apply the same two
    /// transforms here so an operator can paste an authority verbatim:
    ///
    ///   * `example.org:443`  -> `example.org`        (port stripped)
    ///   * `[::1]`            -> `::1`                 (brackets stripped)
    ///   * `[::1]:443`        -> `::1`                 (port + brackets stripped)
    ///   * `192.0.2.10:8080`  -> `192.0.2.10`         (port stripped)
    ///   * `example.org`      -> `example.org`        (unchanged)
    ///   * `::1` (bare IPv6)  -> `::1`                 (unchanged — NOT mangled)
    ///
    /// A trailing `:port` is only stripped when the entry is unambiguously
    /// `host:port`: either bracketed (`[...]:port`) or a single-colon `host:digits`
    /// form. A bare IPv6 literal (multiple colons, unbracketed) is left intact so we
    /// never amputate its last hextet.
    fn normalize_host(e: &str) -> String {
        // Bracketed IPv6: `[addr]` or `[addr]:port`. Strip the optional `:port`
        // that follows the closing bracket, then strip the brackets themselves.
        if let Some(rest) = e.strip_prefix('[') {
            if let Some((inner, _after)) = rest.split_once(']') {
                // `_after` is either empty or `:port`; the engine drops it either way.
                return inner.to_string();
            }
            // Malformed (no closing bracket) — leave verbatim; it just won't match.
            return e.to_string();
        }
        // Unbracketed. Only a single-colon `host:port` is a port-bearing authority;
        // anything with more than one colon is a bare IPv6 literal we must NOT touch.
        if let Some((host, port)) = e.split_once(':') {
            if !host.contains(':') && !port.contains(':') && !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) {
                return host.to_string();
            }
        }
        e.to_string()
    }

    /// Builds an allowlist from the CLI flag values, an optional file, and the
    /// `SPARQ_SERVICE_ALLOW` env var. The union of all three (additive — CLI never
    /// removes an env/file entry). Returns an error if any entry is malformed or the
    /// file cannot be read.
    ///
    /// `cli` are the `--service-allow` values in order; `file` is the
    /// `--service-allow-file` path (if any). The env var is read here so a single
    /// call assembles the full effective allowlist.
    pub fn from_sources(cli: &[String], file: Option<&str>) -> Result<Self, String> {
        Self::from_sources_with_env(cli, file, std::env::var("SPARQ_SERVICE_ALLOW").ok())
    }

    /// [OPUS-4.8] sq-x4jy: the env-baseline-injected core of [`Self::from_sources`].
    /// Production passes the real `SPARQ_SERVICE_ALLOW` value; tests pass an explicit
    /// `Some`/`None` instead of mutating the process-global environment.
    ///
    /// WHY: the prior tests set/removed `SPARQ_SERVICE_ALLOW` via `std::env::set_var`/
    /// `remove_var`. `cargo test` runs a binary's tests on parallel THREADS, so that
    /// mutation raced every concurrent `std::env::var` reader in the same process — a
    /// libc `setenv`/`getenv` data race (UB; it reallocates `environ`) that can abort the
    /// whole test binary, surfacing as CI `build + test` exit 101 with no FAILED line (the
    /// recurring flake in sq-x4jy). Injecting the value removes the global mutation, so the
    /// tests are hermetic and cannot race.
    fn from_sources_with_env(
        cli: &[String],
        file: Option<&str>,
        env: Option<String>,
    ) -> Result<Self, String> {
        let mut out = Self::default();
        // 1. Env baseline.
        if let Some(v) = env {
            out.add_many(&v)?;
        }
        // 2. File (one entry per line; '#' comments and blanks ignored).
        if let Some(path) = file {
            let text = std::fs::read_to_string(path)
                .map_err(|e| format!("--service-allow-file {path}: {e}"))?;
            for line in text.lines() {
                let line = line.split('#').next().unwrap_or("").trim();
                out.add(line)?;
            }
        }
        // 3. CLI flags (additive — widen the union).
        for entry in cli {
            out.add(entry)?;
        }
        Ok(out)
    }

    /// Adds every comma- and/or whitespace-separated token in `s` (the
    /// `SPARQ_SERVICE_ALLOW` env-var grammar).
    pub fn add_many(&mut self, s: &str) -> Result<(), String> {
        for tok in s.split([',', ' ', '\t', '\n', '\r']) {
            self.add(tok)?;
        }
        Ok(())
    }

    /// The entries in the engine's allowlist representation: exact hosts verbatim,
    /// suffix wildcards as leading-dot strings (`.example.org`). Fed to
    /// `sparq_engine::with_service_egress_policy`.
    pub fn engine_entries(&self) -> Vec<String> {
        self.exact.iter().chain(self.suffix.iter()).cloned().collect()
    }

    /// A stable, human-readable rendering for the startup log (suffix entries shown
    /// in the user-facing `*.` form).
    pub fn display(&self) -> String {
        if self.is_empty() {
            return "deny-all (no SERVICE host allowlisted)".to_string();
        }
        let mut parts: Vec<String> = self.exact.iter().cloned().collect();
        parts.extend(self.suffix.iter().map(|s| format!("*{s}")));
        parts.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [OPUS-4.8] sq-x4jy: tests that exercise the env baseline now call
    // `from_sources_with_env` with an explicit value rather than mutating the
    // process-global `SPARQ_SERVICE_ALLOW`. That removes the `setenv`/`getenv` data race
    // (parallel test threads + concurrent readers) that could abort the test binary
    // (CI `build + test` exit 101), so no RAII restore guard is needed any more.

    #[test]
    fn empty_is_deny_all() {
        let a = ServiceAllowlist::default();
        assert!(a.is_empty());
        assert_eq!(a.len(), 0);
        assert_eq!(a.engine_entries(), Vec::<String>::new());
        assert_eq!(a.display(), "deny-all (no SERVICE host allowlisted)");
    }

    #[test]
    fn exact_host_is_lowercased_and_deduped() {
        let mut a = ServiceAllowlist::default();
        a.add("Sparql.Example.ORG").unwrap();
        a.add("sparql.example.org").unwrap(); // dup
        a.add(" 192.0.2.10 ").unwrap(); // trimmed
        assert_eq!(a.len(), 2);
        let mut e = a.engine_entries();
        e.sort();
        assert_eq!(e, vec!["192.0.2.10".to_string(), "sparql.example.org".to_string()]);
    }

    #[test]
    fn exact_host_normalised_to_engine_bare_host() {
        // [OPUS-4.8] sq-4w18: the engine compares the SERVICE IRI authority with the
        // port dropped and IPv6 brackets stripped, so `add` must normalise to the same
        // form or the entry would silently never match (fail-open).
        for (raw, want) in [
            ("example.org:443", "example.org"),    // host:port -> bare host
            ("192.0.2.10:8080", "192.0.2.10"),     // ipv4:port -> bare ip
            ("[::1]", "::1"),                       // bracketed ipv6 -> bare ipv6
            ("[::1]:443", "::1"),                   // bracketed ipv6 + port -> bare ipv6
            ("[2001:db8::1]:80", "2001:db8::1"),    // full bracketed ipv6 + port
            ("example.org", "example.org"),         // already bare -> unchanged
            ("::1", "::1"),                          // bare (unbracketed) ipv6 -> NOT mangled
            ("2001:db8::1", "2001:db8::1"),          // bare full ipv6 -> NOT mangled
        ] {
            let mut a = ServiceAllowlist::default();
            a.add(raw).unwrap();
            assert_eq!(
                a.engine_entries(),
                vec![want.to_string()],
                "{raw:?} should normalise to the bare host {want:?} the engine compares against",
            );
        }
    }

    #[test]
    fn suffix_wildcard_normalised_to_leading_dot() {
        let mut a = ServiceAllowlist::default();
        a.add("*.example.org").unwrap();
        assert_eq!(a.engine_entries(), vec![".example.org".to_string()]);
        assert_eq!(a.display(), "*.example.org");
    }

    #[test]
    fn blank_entries_ignored() {
        let mut a = ServiceAllowlist::default();
        a.add("   ").unwrap();
        a.add("").unwrap();
        assert!(a.is_empty());
    }

    #[test]
    fn malformed_wildcards_rejected() {
        let mut a = ServiceAllowlist::default();
        assert!(a.add("ex*mple.org").is_err()); // '*' not leading
        assert!(a.add("*.").is_err()); // empty suffix
        assert!(a.add("*.a.*.b").is_err()); // extra '*'
    }

    #[test]
    fn add_many_splits_on_comma_and_whitespace() {
        let mut a = ServiceAllowlist::default();
        a.add_many("a.example.org, b.example.org  c.example.org\nd.example.org").unwrap();
        assert_eq!(a.len(), 4);
    }

    #[test]
    fn from_sources_unions_cli_file_env() {
        // [OPUS-4.8] sq-x4jy: inject the env baseline (hermetic — no process-global mutation).
        let a = ServiceAllowlist::from_sources_with_env(
            &["cli.example.org".to_string(), "*.cli.example.org".to_string()],
            None,
            Some("env.example.org, *.env.example.org".to_string()),
        )
        .unwrap();
        assert!(a.engine_entries().contains(&"env.example.org".to_string()));
        assert!(a.engine_entries().contains(&".env.example.org".to_string()));
        assert!(a.engine_entries().contains(&"cli.example.org".to_string()));
        assert!(a.engine_entries().contains(&".cli.example.org".to_string()));
        assert_eq!(a.len(), 4);
    }

    #[test]
    fn from_sources_reads_file_with_comments_and_blanks() {
        // [OPUS-4.8] sq-x4jy: hermetic — no env baseline, unique temp file keyed by PID.
        let dir = std::env::temp_dir();
        let path = dir.join(format!("sparq-svc-allow-{}.txt", std::process::id()));
        std::fs::write(
            &path,
            "# a comment\nfile.example.org\n\n  *.file.example.org   # trailing comment\n",
        )
        .unwrap();
        let a = ServiceAllowlist::from_sources_with_env(&[], Some(path.to_str().unwrap()), None).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(a.len(), 2);
        assert!(a.engine_entries().contains(&"file.example.org".to_string()));
        assert!(a.engine_entries().contains(&".file.example.org".to_string()));
    }

    #[test]
    fn from_sources_missing_file_errors() {
        // [OPUS-4.8] sq-x4jy: hermetic — no env baseline.
        let err = ServiceAllowlist::from_sources_with_env(&[], Some("/no/such/sparq-allow-file"), None)
            .unwrap_err();
        assert!(err.contains("service-allow-file"));
    }
}
