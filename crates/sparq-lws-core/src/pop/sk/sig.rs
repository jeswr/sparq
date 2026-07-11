// AUTHORED-BY Claude Fable 5
//! RFC 9421 attestation parsing + signature-base reconstruction for DPoP-SK — strict, fail-closed.
//!
//! Two halves:
//!
//! 1. A **strict RFC 8941 structured-field parser** (the subset the profile needs: Dictionaries
//!    whose members are Inner Lists or Items, with String/Integer/Token/Byte-Sequence/Boolean bare
//!    items and parameters). Hand-written here — no structured-field crate is in the dependency
//!    tree, and the profile requires *strict* parsing ("its `Signature-Input` parses as a strict
//!    RFC 8941 Dictionary"; anything malformed ⇒ the generic challenge). Every deviation from the
//!    RFC 8941 §4.2 grammar is a parse failure, never a lenient recovery.
//!
//! 2. The **signature base** (RFC 9421 §2.5): lines are rebuilt from the ACTUAL request —
//!    `@method` / `@target-uri` derived server-side (strictly stronger than DPoP's client-asserted
//!    `htm`/`htu`), HTTP fields serialized per §2.1 — and the `"@signature-params"` line reuses the
//!    RECEIVED inner-list serialization byte-for-byte (the spec's verification step 6: "together
//!    with the received Signature-Input serialization"), so a client's own serialization choices
//!    cannot break the HMAC.
//!
//! ## Profile strictness (documented deviations-by-restriction, all fail-closed)
//! - The covered components MUST BEGIN with exactly `("@method" "@target-uri" "authorization")`
//!   in that order (the profile's REQUIRED minimum). Additional components are accepted only as
//!   plain HTTP field names (lowercase, unparameterised, present on the request) — any other
//!   derived component (`@…`) or parameterised component is rejected. Rejecting is always safe:
//!   the client falls back to full DPoP.
//! - Unknown signature parameters are rejected (the profile defines `created`/`keyid`/`nonce`/
//!   `tag`/`alg`/`expires`; anything else ⇒ deny). A server cannot honour semantics it does not
//!   understand, and silently ignoring a parameter the client considered load-bearing would be a
//!   fail-open.
//! - The `nonce` must be the canonical decimal ASCII of a `u64 ≥ 1` (no sign, no leading zeros) —
//!   "Servers MUST reject non-numeric, zero, or >64-bit values at parse time".

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;

/// A covered component of the attestation, validated against the profile's rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Component {
    /// `"@method"` — derived server-side from the actual request method.
    Method,
    /// `"@target-uri"` — derived server-side from the actual request target.
    TargetUri,
    /// A plain HTTP field, by its lowercase name (includes the REQUIRED `authorization`).
    Field(String),
}

/// A parsed, validated `dpop-sk` attestation (one per request; parse fails otherwise).
#[derive(Debug, Clone)]
pub struct Attestation {
    /// The dictionary key naming the signature (e.g. `sig`) — used to pair `Signature-Input` with
    /// the `Signature` member carrying the tag bytes.
    pub label: String,
    /// The session identifier (`keyid`).
    pub keyid: String,
    /// The anti-replay counter (`nonce`), parsed strictly (≥ 1).
    pub nonce: u64,
    /// The `created` timestamp (epoch seconds).
    pub created: i64,
    /// The optional `expires` timestamp.
    pub expires: Option<i64>,
    /// The validated covered components, in declared order (required triple first).
    pub covered: Vec<Component>,
    /// The RECEIVED inner-list + parameters serialization — reused verbatim as the
    /// `"@signature-params"` line of the signature base.
    pub raw_params: String,
    /// The decoded HMAC tag from the paired `Signature` member (32 bytes for hmac-sha256).
    pub signature: Vec<u8>,
}

/// Opaque parse/validation failure. Deliberately detail-free: every failure maps to the same
/// generic DPoP challenge (spec §"Failure signalling" — no oracle detail).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SigParseError;

type PResult<T> = Result<T, SigParseError>;

fn fail<T>() -> PResult<T> {
    Err(SigParseError)
}

// =================================================================================================
// RFC 8941 strict parsing (the required subset)
// =================================================================================================

/// A bare item (RFC 8941 §3.3). Decimal is parsed for completeness (skipping a non-dpop-sk member
/// must not fail on a legal field) but carried as its raw text — the profile never consumes one.
#[derive(Debug, Clone, PartialEq)]
enum Bare {
    Integer(i64),
    Decimal(String),
    String(String),
    Token(String),
    Bytes(Vec<u8>),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq)]
struct Item {
    bare: Bare,
    params: Vec<(String, Bare)>,
}

/// A parsed RFC 8941 inner list: its items + the list-level parameters.
type InnerList = (Vec<Item>, Vec<(String, Bare)>);

#[derive(Debug, Clone, PartialEq)]
enum MemberValue {
    Item(Item),
    InnerList {
        items: Vec<Item>,
        params: Vec<(String, Bare)>,
    },
}

#[derive(Debug, Clone)]
struct Member {
    key: String,
    value: MemberValue,
    /// Byte span of the member's value+parameters serialization within the parsed input (empty for
    /// an implicit-`?1` member).
    raw_span: (usize, usize),
}

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> PResult<Self> {
        if !input.is_ascii() {
            return fail();
        }
        Ok(Self {
            input: input.as_bytes(),
            pos: 0,
        })
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_sp(&mut self) {
        while self.peek() == Some(b' ') {
            self.pos += 1;
        }
    }

    fn skip_ows(&mut self) {
        while matches!(self.peek(), Some(b' ') | Some(b'\t')) {
            self.pos += 1;
        }
    }

    /// RFC 8941 §4.2.2 — parse a Dictionary.
    fn dictionary(&mut self) -> PResult<Vec<Member>> {
        let mut members: Vec<Member> = Vec::new();
        self.skip_sp();
        if self.peek().is_none() {
            return Ok(members); // empty field value ⇒ empty dictionary
        }
        loop {
            let key = self.key()?;
            let (value, raw_span) = if self.peek() == Some(b'=') {
                self.pos += 1;
                let start = self.pos;
                let value = self.item_or_inner_list()?;
                (value, (start, self.pos))
            } else {
                // Implicit `?1` with optional parameters.
                let start = self.pos;
                let params = self.parameters()?;
                (
                    MemberValue::Item(Item {
                        bare: Bare::Bool(true),
                        params,
                    }),
                    (start, self.pos),
                )
            };
            // Duplicate keys: RFC 8941 keeps the LAST — but for a security field a duplicate is a
            // smuggling vector, so this profile rejects it outright (strict, fail-closed).
            if members.iter().any(|m| m.key == key) {
                return fail();
            }
            members.push(Member {
                key,
                value,
                raw_span,
            });
            self.skip_ows();
            match self.bump() {
                None => return Ok(members),
                Some(b',') => {
                    self.skip_ows();
                    if self.peek().is_none() {
                        return fail(); // trailing comma
                    }
                }
                Some(_) => return fail(),
            }
        }
    }

    /// RFC 8941 key: lcalpha/"*" then lcalpha/DIGIT/"_"/"-"/"."/"*".
    fn key(&mut self) -> PResult<String> {
        let start = self.pos;
        match self.peek() {
            Some(c) if c.is_ascii_lowercase() || c == b'*' => self.pos += 1,
            _ => return fail(),
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_lowercase()
                || c.is_ascii_digit()
                || matches!(c, b'_' | b'-' | b'.' | b'*')
            {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(std::str::from_utf8(&self.input[start..self.pos])
            .expect("ascii checked")
            .to_string())
    }

    fn item_or_inner_list(&mut self) -> PResult<MemberValue> {
        if self.peek() == Some(b'(') {
            let (items, params) = self.inner_list()?;
            Ok(MemberValue::InnerList { items, params })
        } else {
            Ok(MemberValue::Item(self.item()?))
        }
    }

    /// RFC 8941 §4.2.1.2 — inner list: `(` items `)` parameters.
    fn inner_list(&mut self) -> PResult<InnerList> {
        if self.bump() != Some(b'(') {
            return fail();
        }
        let mut items = Vec::new();
        loop {
            self.skip_sp();
            if self.peek() == Some(b')') {
                self.pos += 1;
                let params = self.parameters()?;
                return Ok((items, params));
            }
            if self.peek().is_none() {
                return fail();
            }
            items.push(self.item()?);
            match self.peek() {
                Some(b' ') | Some(b')') => {}
                _ => return fail(),
            }
        }
    }

    fn item(&mut self) -> PResult<Item> {
        let bare = self.bare_item()?;
        let params = self.parameters()?;
        Ok(Item { bare, params })
    }

    /// RFC 8941 §4.2.3.2 — parameters: `;` SP* key (`=` bare-item)?.
    fn parameters(&mut self) -> PResult<Vec<(String, Bare)>> {
        let mut params = Vec::new();
        while self.peek() == Some(b';') {
            self.pos += 1;
            self.skip_sp();
            let key = self.key()?;
            let value = if self.peek() == Some(b'=') {
                self.pos += 1;
                self.bare_item()?
            } else {
                Bare::Bool(true)
            };
            if params.iter().any(|(k, _)| *k == key) {
                return fail(); // strict: duplicate parameter keys rejected (see dictionary note)
            }
            params.push((key, value));
        }
        Ok(params)
    }

    /// RFC 8941 §4.2.3.1 — bare item, dispatched on the first character.
    fn bare_item(&mut self) -> PResult<Bare> {
        match self.peek() {
            Some(b'-') | Some(b'0'..=b'9') => self.number(),
            Some(b'"') => self.string(),
            Some(b':') => self.byte_sequence(),
            Some(b'?') => self.boolean(),
            Some(c) if c.is_ascii_alphabetic() || c == b'*' => self.token(),
            _ => fail(),
        }
    }

    fn number(&mut self) -> PResult<Bare> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        let int_start = self.pos;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.pos += 1;
        }
        let int_digits = self.pos - int_start;
        if int_digits == 0 {
            return fail();
        }
        if self.peek() == Some(b'.') {
            // Decimal: ≤12 integer digits, 1–3 fractional digits (RFC 8941 bounds).
            if int_digits > 12 {
                return fail();
            }
            self.pos += 1;
            let frac_start = self.pos;
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
            let frac = self.pos - frac_start;
            if frac == 0 || frac > 3 {
                return fail();
            }
            let text = std::str::from_utf8(&self.input[start..self.pos]).expect("ascii");
            return Ok(Bare::Decimal(text.to_string()));
        }
        if int_digits > 15 {
            return fail();
        }
        let text = std::str::from_utf8(&self.input[start..self.pos]).expect("ascii");
        text.parse::<i64>().map(Bare::Integer).or_else(|_| fail())
    }

    fn string(&mut self) -> PResult<Bare> {
        if self.bump() != Some(b'"') {
            return fail();
        }
        let mut out = String::new();
        loop {
            match self.bump() {
                Some(b'"') => return Ok(Bare::String(out)),
                Some(b'\\') => match self.bump() {
                    Some(c @ (b'"' | b'\\')) => out.push(c as char),
                    _ => return fail(),
                },
                Some(c) if (0x20..=0x7e).contains(&c) => out.push(c as char),
                _ => return fail(),
            }
        }
    }

    fn token(&mut self) -> PResult<Bare> {
        let start = self.pos;
        match self.peek() {
            Some(c) if c.is_ascii_alphabetic() || c == b'*' => self.pos += 1,
            _ => return fail(),
        }
        while let Some(c) = self.peek() {
            // tchar / ":" / "/"
            if c.is_ascii_alphanumeric()
                || matches!(
                    c,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                        | b':'
                        | b'/'
                )
            {
                self.pos += 1;
            } else {
                break;
            }
        }
        Ok(Bare::Token(
            std::str::from_utf8(&self.input[start..self.pos])
                .expect("ascii")
                .to_string(),
        ))
    }

    fn byte_sequence(&mut self) -> PResult<Bare> {
        if self.bump() != Some(b':') {
            return fail();
        }
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == b':' {
                let b64 = &self.input[start..self.pos];
                self.pos += 1;
                let decoded = STANDARD.decode(b64).map_err(|_| SigParseError)?;
                return Ok(Bare::Bytes(decoded));
            }
            if c.is_ascii_alphanumeric() || matches!(c, b'+' | b'/' | b'=') {
                self.pos += 1;
            } else {
                return fail();
            }
        }
        fail()
    }

    fn boolean(&mut self) -> PResult<Bare> {
        if self.bump() != Some(b'?') {
            return fail();
        }
        match self.bump() {
            Some(b'1') => Ok(Bare::Bool(true)),
            Some(b'0') => Ok(Bare::Bool(false)),
            _ => fail(),
        }
    }
}

// =================================================================================================
// Attestation extraction (profile validation over the parsed dictionaries)
// =================================================================================================

/// The exact string the `tag` parameter must carry.
pub const TAG: &str = "dpop-sk";
/// The attestation algorithm this profile version defines.
pub const ALG: &str = "hmac-sha256";
/// The HMAC-SHA256 tag length in bytes.
const TAG_LEN: usize = 32;

/// Whether the (joined) `Signature-Input` field value could carry a `dpop-sk` tag at all — the
/// cheap routing pre-check. Requests without the substring are NOT under this profile (their
/// Signature headers, if any, belong to some other application and are ignored); requests WITH it
/// go through [`parse_attestation`], whose failure is a deny (never a fallback).
pub fn mentions_dpop_sk(signature_input: &str) -> bool {
    signature_input.contains(TAG)
}

/// Parse + validate the request's attestation from the joined `Signature-Input` and `Signature`
/// field values. Returns the single `dpop-sk`-tagged attestation, `Ok(None)` when NO member
/// carries the tag (not under this profile), or an error on ANY malformation (fail-closed).
pub fn parse_attestation(signature_input: &str, signature: &str) -> PResult<Option<Attestation>> {
    let members = Parser::new(signature_input)?.dictionary()?;

    // Locate the dpop-sk-tagged member(s): tagged ⇒ must be an inner list with our tag param.
    let mut tagged: Vec<&Member> = Vec::new();
    for m in &members {
        let params = match &m.value {
            MemberValue::InnerList { params, .. } => params,
            MemberValue::Item(item) => &item.params,
        };
        if params
            .iter()
            .any(|(k, v)| k == "tag" && *v == Bare::String(TAG.to_string()))
        {
            tagged.push(m);
        }
    }
    let member = match tagged.len() {
        0 => return Ok(None),
        1 => tagged[0],
        _ => return fail(), // "exactly one dpop-sk-tagged signature"
    };

    let (items, params) = match &member.value {
        MemberValue::InnerList { items, params } => (items, params),
        MemberValue::Item(_) => return fail(), // a signature input must be an inner list
    };

    // --- Covered components ----------------------------------------------------------------
    let mut covered: Vec<Component> = Vec::with_capacity(items.len());
    for item in items {
        if !item.params.is_empty() {
            return fail(); // parameterised components (;req, ;key, …) are not in this profile
        }
        let name = match &item.bare {
            Bare::String(s) => s.as_str(),
            _ => return fail(), // component identifiers are sf-strings
        };
        let comp = match name {
            "@method" => Component::Method,
            "@target-uri" => Component::TargetUri,
            other if other.starts_with('@') => return fail(), // unsupported derived component
            other => {
                // Plain field name: RFC 9421 §2.1 lowercases component names; enforce it.
                if other.is_empty()
                    || !other
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                {
                    return fail();
                }
                Component::Field(other.to_string())
            }
        };
        if covered.contains(&comp) {
            return fail(); // duplicate covered component (RFC 9421 §2.5 error)
        }
        covered.push(comp);
    }
    // The REQUIRED minimum, in order, first (profile rule).
    let required = [
        Component::Method,
        Component::TargetUri,
        Component::Field("authorization".to_string()),
    ];
    if covered.len() < required.len() || covered[..required.len()] != required {
        return fail();
    }

    // --- Signature parameters ---------------------------------------------------------------
    let mut keyid: Option<String> = None;
    let mut nonce: Option<u64> = None;
    let mut created: Option<i64> = None;
    let mut expires: Option<i64> = None;
    for (k, v) in params {
        match (k.as_str(), v) {
            ("created", Bare::Integer(i)) => created = Some(*i),
            ("expires", Bare::Integer(i)) => expires = Some(*i),
            ("keyid", Bare::String(s)) => keyid = Some(s.clone()),
            ("nonce", Bare::String(s)) => nonce = Some(parse_nonce(s)?),
            // `alg`, when present, MUST equal the session's algorithm — v1 defines exactly one, so
            // any other value (or a non-string) is rejected here at parse time (downgrade rule 5).
            ("alg", Bare::String(s)) if s == ALG => {}
            ("tag", Bare::String(s)) if s == TAG => {}
            _ => return fail(), // unknown/mistyped parameter ⇒ deny (see module docs)
        }
    }
    let (Some(keyid), Some(nonce), Some(created)) = (keyid, nonce, created) else {
        return fail(); // REQUIRED parameters missing
    };

    // --- The paired Signature member ---------------------------------------------------------
    let sig_members = Parser::new(signature)?.dictionary()?;
    let sig_member = sig_members
        .iter()
        .find(|m| m.key == member.key)
        .ok_or(SigParseError)?;
    let sig_bytes = match &sig_member.value {
        MemberValue::Item(Item {
            bare: Bare::Bytes(b),
            params,
        }) if params.is_empty() => b.clone(),
        _ => return fail(),
    };
    if sig_bytes.len() != TAG_LEN {
        return fail();
    }

    // The received serialization of the inner list + its parameters — verbatim, for the base.
    // (`Parser::new` established the whole field is ASCII, so the byte span is a char boundary.)
    let raw_params = signature_input[member.raw_span.0..member.raw_span.1].to_string();

    Ok(Some(Attestation {
        label: member.key.clone(),
        keyid,
        nonce,
        created,
        expires,
        covered,
        raw_params,
        signature: sig_bytes,
    }))
}

/// Strict counter parse: canonical decimal ASCII of a `u64`, ≥ 1 (no sign, no leading zeros —
/// "Servers MUST reject non-numeric, zero, or >64-bit values at parse time").
fn parse_nonce(s: &str) -> PResult<u64> {
    if s.is_empty() || s.len() > 20 || !s.bytes().all(|b| b.is_ascii_digit()) {
        return fail();
    }
    if s.starts_with('0') {
        return fail(); // rejects both "0" (zero) and non-canonical "01"
    }
    s.parse::<u64>().or_else(|_| fail())
}

// =================================================================================================
// Signature-base reconstruction (RFC 9421 §2.5)
// =================================================================================================

/// Build the signature base from the ACTUAL request (`method`, `target_uri`, `headers`) plus the
/// received parameter serialization. Fails (⇒ deny) if any covered field is absent or malformed.
pub fn signature_base(
    attestation: &Attestation,
    method: &str,
    target_uri: &str,
    headers: &http::HeaderMap,
) -> PResult<String> {
    let mut lines: Vec<String> = Vec::with_capacity(attestation.covered.len() + 1);
    for comp in &attestation.covered {
        match comp {
            Component::Method => {
                // @method is the request method — uppercase by HTTP grammar (RFC 9421 §2.2.1).
                lines.push(format!("\"@method\": {}", method.to_ascii_uppercase()));
            }
            Component::TargetUri => {
                lines.push(format!("\"@target-uri\": {target_uri}"));
            }
            Component::Field(name) => {
                lines.push(format!("\"{name}\": {}", field_value(headers, name)?));
            }
        }
    }
    lines.push(format!("\"@signature-params\": {}", attestation.raw_params));
    Ok(lines.join("\n"))
}

/// RFC 9421 §2.1 HTTP-field serialization: each instance's value trimmed of leading/trailing
/// SP/HTAB, instances joined with `", "`. Absent or non-ASCII fields fail.
fn field_value(headers: &http::HeaderMap, name: &str) -> PResult<String> {
    let mut values: Vec<String> = Vec::new();
    for v in headers.get_all(name) {
        let s = v.to_str().map_err(|_| SigParseError)?;
        values.push(s.trim_matches([' ', '\t']).to_string());
    }
    if values.is_empty() {
        return fail();
    }
    Ok(values.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The spec's Appendix-A Signature-Input value (single line on the wire).
    const APPENDIX_A_INPUT: &str = "sig=(\"@method\" \"@target-uri\" \"authorization\");created=1720000000;keyid=\"AAECAwQFBgcICQoLDA0ODw\";alg=\"hmac-sha256\";nonce=\"1\";tag=\"dpop-sk\"";
    const APPENDIX_A_SIG: &str = "sig=:hPkuPd32xbb4hUjR/hjbj0Cp445ZfWoOgrcq8+f3I3g=:";

    fn parse_ok(input: &str, sig: &str) -> Attestation {
        parse_attestation(input, sig)
            .expect("parse must not error")
            .expect("must find the dpop-sk attestation")
    }

    #[test]
    fn appendix_a_signature_input_parses_exactly() {
        let a = parse_ok(APPENDIX_A_INPUT, APPENDIX_A_SIG);
        assert_eq!(a.label, "sig");
        assert_eq!(a.keyid, "AAECAwQFBgcICQoLDA0ODw");
        assert_eq!(a.nonce, 1);
        assert_eq!(a.created, 1720000000);
        assert_eq!(a.expires, None);
        assert_eq!(
            a.covered,
            vec![
                Component::Method,
                Component::TargetUri,
                Component::Field("authorization".into())
            ]
        );
        // The raw params must be the byte-exact received serialization (used in the base).
        assert_eq!(
            a.raw_params,
            "(\"@method\" \"@target-uri\" \"authorization\");created=1720000000;keyid=\"AAECAwQFBgcICQoLDA0ODw\";alg=\"hmac-sha256\";nonce=\"1\";tag=\"dpop-sk\""
        );
        assert_eq!(a.signature.len(), 32);
    }

    #[test]
    fn appendix_a_signature_base_is_byte_exact() {
        let a = parse_ok(APPENDIX_A_INPUT, APPENDIX_A_SIG);
        let token = crate::pop::sk::test_vectors::EXAMPLE_TOKEN;
        let mut headers = http::HeaderMap::new();
        headers.insert(
            http::header::AUTHORIZATION,
            format!("DPoP {token}").parse().unwrap(),
        );
        let base =
            signature_base(&a, "GET", "https://pod.example/alice/private/doc", &headers).unwrap();
        let expected = format!(
            "\"@method\": GET\n\"@target-uri\": https://pod.example/alice/private/doc\n\"authorization\": DPoP {token}\n\"@signature-params\": (\"@method\" \"@target-uri\" \"authorization\");created=1720000000;keyid=\"AAECAwQFBgcICQoLDA0ODw\";alg=\"hmac-sha256\";nonce=\"1\";tag=\"dpop-sk\""
        );
        assert_eq!(base, expected);
    }

    #[test]
    fn no_dpop_sk_tag_is_not_ours() {
        // A foreign RFC 9421 signature (another application's tag) is Ok(None) — not processed,
        // not denied by this profile (the DPoP baseline still gates the request).
        let input = "sig=(\"@method\");created=1;keyid=\"x\";tag=\"other-app\"";
        let sig = "sig=:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=:";
        assert!(parse_attestation(input, sig).unwrap().is_none());
    }

    #[test]
    fn two_dpop_sk_tagged_members_are_rejected() {
        let input = format!("{APPENDIX_A_INPUT}, sig2=(\"@method\" \"@target-uri\" \"authorization\");created=1720000000;keyid=\"k\";nonce=\"2\";tag=\"dpop-sk\"");
        assert!(parse_attestation(&input, APPENDIX_A_SIG).is_err());
    }

    #[test]
    fn missing_required_params_rejected() {
        for omitted in ["created", "keyid", "nonce"] {
            let input = APPENDIX_A_INPUT
                .split(';')
                .filter(|p| !p.starts_with(omitted))
                .collect::<Vec<_>>()
                .join(";");
            assert!(
                parse_attestation(&input, APPENDIX_A_SIG).is_err(),
                "must reject when {omitted} is missing"
            );
        }
    }

    #[test]
    fn wrong_alg_param_rejected() {
        // Downgrade rule 5: a mismatching `alg` parameter is rejected.
        let input = APPENDIX_A_INPUT.replace("hmac-sha256", "rsa-pss-sha512");
        assert!(parse_attestation(&input, APPENDIX_A_SIG).is_err());
    }

    #[test]
    fn unknown_param_rejected() {
        let input = format!("{APPENDIX_A_INPUT};mystery=\"x\"");
        assert!(parse_attestation(&input, APPENDIX_A_SIG).is_err());
    }

    #[test]
    fn covered_set_must_start_with_required_triple_in_order() {
        // Missing authorization.
        let input =
            "sig=(\"@method\" \"@target-uri\");created=1;keyid=\"k\";nonce=\"1\";tag=\"dpop-sk\"";
        assert!(parse_attestation(input, APPENDIX_A_SIG).is_err());
        // Reordered.
        let input = "sig=(\"@target-uri\" \"@method\" \"authorization\");created=1;keyid=\"k\";nonce=\"1\";tag=\"dpop-sk\"";
        assert!(parse_attestation(input, APPENDIX_A_SIG).is_err());
        // Unsupported derived component after the triple.
        let input = "sig=(\"@method\" \"@target-uri\" \"authorization\" \"@query\");created=1;keyid=\"k\";nonce=\"1\";tag=\"dpop-sk\"";
        assert!(parse_attestation(input, APPENDIX_A_SIG).is_err());
        // Duplicate component.
        let input = "sig=(\"@method\" \"@target-uri\" \"authorization\" \"authorization\");created=1;keyid=\"k\";nonce=\"1\";tag=\"dpop-sk\"";
        assert!(parse_attestation(input, APPENDIX_A_SIG).is_err());
        // Extra plain field is ACCEPTED at parse time (verified against the request later).
        let input = "sig=(\"@method\" \"@target-uri\" \"authorization\" \"content-type\");created=1;keyid=\"k\";nonce=\"1\";tag=\"dpop-sk\"";
        let a = parse_ok(input, APPENDIX_A_SIG);
        assert_eq!(a.covered.len(), 4);
        assert_eq!(a.covered[3], Component::Field("content-type".into()));
        // Uppercase field name rejected (RFC 9421 lowercases component names).
        let input = "sig=(\"@method\" \"@target-uri\" \"authorization\" \"Content-Type\");created=1;keyid=\"k\";nonce=\"1\";tag=\"dpop-sk\"";
        assert!(parse_attestation(input, APPENDIX_A_SIG).is_err());
    }

    #[test]
    fn nonce_strictness() {
        let mk = |nonce: &str| {
            format!("sig=(\"@method\" \"@target-uri\" \"authorization\");created=1;keyid=\"k\";nonce=\"{nonce}\";tag=\"dpop-sk\"")
        };
        // Valid.
        assert!(parse_attestation(&mk("1"), APPENDIX_A_SIG)
            .unwrap()
            .is_some());
        assert!(
            parse_attestation(&mk("18446744073709551615"), APPENDIX_A_SIG)
                .unwrap()
                .is_some()
        ); // u64::MAX
           // Rejected: zero, leading zero, sign, non-digit, > u64, empty, integer-typed.
        for bad in ["0", "01", "+1", "-1", "1a", "18446744073709551616", ""] {
            assert!(
                parse_attestation(&mk(bad), APPENDIX_A_SIG).is_err(),
                "nonce {bad:?} must be rejected"
            );
        }
        let int_nonce = "sig=(\"@method\" \"@target-uri\" \"authorization\");created=1;keyid=\"k\";nonce=1;tag=\"dpop-sk\"";
        assert!(parse_attestation(int_nonce, APPENDIX_A_SIG).is_err());
    }

    #[test]
    fn signature_member_strictness() {
        // Missing paired Signature member.
        assert!(parse_attestation(APPENDIX_A_INPUT, "other=:AAAA:").is_err());
        // Wrong length (not 32 bytes).
        assert!(parse_attestation(APPENDIX_A_INPUT, "sig=:AAAA:").is_err());
        // Not a byte sequence.
        assert!(parse_attestation(APPENDIX_A_INPUT, "sig=\"nope\"").is_err());
        // Invalid base64.
        assert!(parse_attestation(APPENDIX_A_INPUT, "sig=:!!!!:").is_err());
    }

    #[test]
    fn tag_must_be_a_string_not_a_token() {
        // tag=dpop-sk as a Token (unquoted) is NOT the sf-string the profile requires; with no
        // string-typed dpop-sk tag present the member is simply not ours… but the unknown-param
        // rule then rejects the member? No — with no dpop-sk-tagged member at all the field is
        // Ok(None) (not under this profile; DPoP still gates the request).
        let input = "sig=(\"@method\" \"@target-uri\" \"authorization\");created=1;keyid=\"k\";nonce=\"1\";tag=dpop-sk";
        assert!(parse_attestation(input, APPENDIX_A_SIG).unwrap().is_none());
    }

    #[test]
    fn malformed_dictionaries_rejected() {
        for bad in [
            "sig=(\"@method\"",                                 // unterminated inner list
            "SIG=(\"@method\");tag=\"dpop-sk\"",                // uppercase key
            "sig=(\"@method\");tag=\"dpop-sk\",",               // trailing comma
            "sig=(\"@method\"\"@target-uri\");tag=\"dpop-sk\"", // missing item separator
            "sig=('x');tag=\"dpop-sk\"",                        // not an RFC 8941 bare item
        ] {
            assert!(
                parse_attestation(bad, APPENDIX_A_SIG).is_err(),
                "{bad:?} must be rejected"
            );
        }
        // Duplicate dictionary key.
        let dup = "sig=(\"@method\");tag=\"dpop-sk\", sig=(\"@method\");tag=\"dpop-sk\"";
        assert!(parse_attestation(dup, APPENDIX_A_SIG).is_err());
    }

    #[test]
    fn multi_instance_field_values_join_per_rfc9421() {
        let mut headers = http::HeaderMap::new();
        headers.append("x-multi", "  a ".parse().unwrap());
        headers.append("x-multi", "b\t".parse().unwrap());
        assert_eq!(field_value(&headers, "x-multi").unwrap(), "a, b");
        assert!(field_value(&headers, "absent").is_err());
    }

    #[test]
    fn mentions_pre_check() {
        assert!(mentions_dpop_sk(APPENDIX_A_INPUT));
        assert!(!mentions_dpop_sk("sig=(\"@method\");tag=\"other\""));
    }
}
