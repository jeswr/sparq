// The APPROVED wording — must NOT trip the gate.
pub struct ContractRequest {
    pub query: String,
    pub nonce: String,
    pub requirements: TrustRequirements,
}

/// Parse a request into a [`ContractRequest`].
pub fn parse_request(query: &str) -> ContractRequest { todo!() }

// Bare "envelope" is legitimate and unrelated — the ban is on the COMPOUND term.
pub struct Envelope { pub payload: Vec<u8> }
/// The leakage envelope of the routing seam; an SD-JWT envelope over a JWT.
pub fn noise_envelope() {}
