// Reproduces the PR #3451 shape: the banned term as PUBLIC Rust API.
// The old gate scanned only *.md, so this was unreachable.
pub struct TrustEnvelope {
    pub query: String,
    pub nonce: String,
}

/// Parse an envelope into a [`TrustEnvelope`].
pub fn parse_envelope(query: &str) -> TrustEnvelope {
    let trust_envelope = TrustEnvelope { query: query.into(), nonce: String::new() };
    trust_envelope
}
