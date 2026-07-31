//! [SONNET-4.6] (sq-lhcot.2, issue #2789) The **external-key `.spqv` interoperability profile** —
//! a DRAFT, cross-implementation table mapping producer-supplied opaque **external keys** to vector
//! **slots**, plus the byte-level conformance fixtures under
//! `tests/fixtures/external-key/`. The profile document is
//! `research/spqv-external-key-profile.md`.
//!
//! # Status: DRAFT — the profile is NOT frozen
//!
//! [`EXTERNAL_KEY_PROFILE_VERSION`] is **0**, which this module defines to mean *unfrozen draft*.
//! The profile is being co-designed with Kern/PSS on GitHub #1746; the frozen profile will carry a
//! version `>= 1`. This build **rejects** any other version, so a file written against the frozen
//! profile fails loudly here rather than being mis-parsed under draft rules. Nothing produced under
//! version 0 carries a compatibility promise, and this module is deliberately **not wired into
//! [`crate::store::VectorStore`]**: the `.spqv` container (v1/v2/v3/v4) is byte-for-byte unchanged
//! by this feature. What ships here is the *format test vectors and parser* the bead sequences
//! **before** any mmap-index work — no mmap index is built (entries are parsed into an owned,
//! binary-searched `Vec`).
//!
//! # What sparq does and does not own
//!
//! sparq is a **consumer and format co-designer**, never a key producer. An external key is an
//! **opaque multihash digest supplied by the producer**; this module never derives, computes, or
//! defines one, and it defines **no** concept-hash scheme. Its whole job is to normalize (decode to
//! bytes), store in a canonical order, compare byte-for-byte, and reject anything it cannot verify.
//!
//! # Why external keys exist (the foot-gun they close)
//!
//! A `.spqv` is keyed by the build-time **dictionary id**, so it is valid only against the exact
//! persisted graph generation it was built against — a logically identical re-parse can permute the
//! ids, and the order-independent fingerprint ([`crate::fingerprint`]) passes anyway (the id-keyed
//! staleness contract, see [`crate::store`]). An external key is generation-independent: it names
//! the identity, not the id. This table is therefore deliberately **not** bound to a graph
//! fingerprint; it *is* bound to the embedding pipeline via an optional
//! [`EmbeddingProvenance`] block, because a key that survives a re-parse must still not survive a
//! change of embedding space.
//!
//! # Block layout (version 0, all fixed-width little-endian)
//!
//! ```text
//! offset 0   magic       b"SPQVXKEY"                       8 bytes
//! offset 8   version     u16 = EXTERNAL_KEY_PROFILE_VERSION 2 bytes
//! offset 10  flags       u16, MUST be 0 (reserved)          2 bytes
//! offset 12  hash_code   u32 multicodec hash code (opaque)  4 bytes
//! offset 16  key_len     u32 digest length in bytes         4 bytes
//! offset 20  count       u64 entry count                    8 bytes
//! offset 28  prov_len    u32 provenance block length        4 bytes
//! offset 32  provenance  [prov_len] bytes (EmbeddingProvenance::to_bytes; 0 ⇒ absent)
//!            entries     count × (digest[key_len] || slot u32)
//!            sig_len     u32
//!            signature   [sig_len] bytes — OPAQUE, NEVER verified here
//! ```
//!
//! The `(hash_code, key_len)` pair is factored into the header rather than repeated per entry, so
//! entries are **fixed-width** and a future mmap reader can binary-search them with no offset table.
//! The consequence is a profile rule: **one table carries one multihash code at one length**.
//! Factoring it out does not weaken the identity — [`ExternalKeyTable::lookup`] compares the code
//! *and* the length before the digest, so a record declaring a weaker hash cannot match a digest
//! stored under a stronger one.
//!
//! # Fail-closed parse invariants
//!
//! [`ExternalKeyTable::from_bytes`] is an `Err` — never a partial parse, never a panic — on: a wrong
//! magic; any version other than [`EXTERNAL_KEY_PROFILE_VERSION`]; non-zero `flags`; a `key_len` of
//! 0 or above [`MAX_EXTERNAL_KEY_LEN`]; a `count`/`prov_len`/`sig_len` that overruns the buffer
//! (checked arithmetically, before any allocation); entries not in **strictly ascending** digest
//! order (which rejects an unsorted table *and* a duplicate key in one comparison); a provenance
//! block that does not parse; or trailing bytes after the signature area.
//!
//! # The signature area is opaque and UNVERIFIED
//!
//! [`ExternalKeyTable::unverified_signature`] is named for what it is. The signature algorithm is
//! Kern's to choose and is an open question on #1746; this module round-trips the bytes and
//! **performs no verification whatsoever**. Presence of a signature is not evidence of anything.

use crate::spqv_provenance::EmbeddingProvenance;

/// First eight bytes of an external-key table block.
pub const EXTERNAL_KEY_MAGIC: [u8; 8] = *b"SPQVXKEY";

/// The profile version this build writes and reads. **0 means DRAFT / not frozen** — see the module
/// docs. The frozen cross-implementation profile (#1746) will carry a version `>= 1`, which this
/// build rejects rather than mis-parsing under draft rules.
pub const EXTERNAL_KEY_PROFILE_VERSION: u16 = 0;

/// Upper bound on an external key's digest length, in bytes. 64 admits the longest digests in
/// practical use (SHA-512 is 64 bytes) while bounding the per-entry width a corrupt header can
/// claim.
pub const MAX_EXTERNAL_KEY_LEN: usize = 64;

/// Upper bound on the opaque signature area, as a defence against a corrupt header claiming an
/// absurd length before any allocation happens.
pub const MAX_EXTERNAL_KEY_SIGNATURE_LEN: usize = 64 * 1024;

/// Fixed size of the version-0 header prefix, up to and including `prov_len`.
const HEADER_PREFIX_LEN: usize = 32;

/// [SONNET-4.6] (sq-lhcot.2) A DRAFT external-key table: producer-supplied opaque multihash digests
/// mapped to `.spqv` vector slots, in canonical ascending digest order.
///
/// All entries share one multihash `hash_code` at one `key_len` (see the module docs). Construct
/// with [`ExternalKeyTable::new`], fill with [`insert`](Self::insert), serialize with
/// [`to_bytes`](Self::to_bytes); parse a received table with [`from_bytes`](Self::from_bytes) and
/// resolve with [`lookup`](Self::lookup).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalKeyTable {
    hash_code: u32,
    key_len: usize,
    /// Ascending by digest bytes, with no duplicates — the type invariant.
    entries: Vec<(Vec<u8>, u32)>,
    provenance: Option<EmbeddingProvenance>,
    signature: Vec<u8>,
}

impl ExternalKeyTable {
    /// An empty table for keys of multihash `hash_code` at `key_len` bytes. `Err` if `key_len` is 0
    /// or above [`MAX_EXTERNAL_KEY_LEN`] — the same bound the parser enforces, so a table this
    /// build can construct is always a table it can re-read.
    pub fn new(hash_code: u32, key_len: usize) -> Result<ExternalKeyTable, String> {
        check_key_len(key_len)?;
        Ok(ExternalKeyTable {
            hash_code,
            key_len,
            entries: Vec::new(),
            provenance: None,
            signature: Vec::new(),
        })
    }

    /// Binds the embedding provenance the vectors behind these slots were produced with. An
    /// external key is generation-independent but **not** embedding-space-independent, so a
    /// consumer can still reject a table produced by an incompatible pipeline.
    #[must_use]
    pub fn with_provenance(mut self, provenance: EmbeddingProvenance) -> ExternalKeyTable {
        self.provenance = Some(provenance);
        self
    }

    /// Attaches the opaque signature area. The bytes are **never verified** by this crate (the
    /// algorithm is an open question on #1746); they round-trip and nothing more. `Err` if the
    /// bytes exceed [`MAX_EXTERNAL_KEY_SIGNATURE_LEN`].
    pub fn with_unverified_signature(
        mut self,
        signature: impl Into<Vec<u8>>,
    ) -> Result<ExternalKeyTable, String> {
        let signature = signature.into();
        if signature.len() > MAX_EXTERNAL_KEY_SIGNATURE_LEN {
            return Err(format!(
                "external-key signature area is {} byte(s), above the {}-byte cap",
                signature.len(),
                MAX_EXTERNAL_KEY_SIGNATURE_LEN
            ));
        }
        self.signature = signature;
        Ok(self)
    }

    /// Records `digest → slot`. `Err` on a digest whose length is not this table's `key_len`, or on
    /// a **duplicate** digest — the profile forbids one key resolving to two slots, and accepting
    /// the second silently would make resolution order-dependent.
    pub fn insert(&mut self, digest: &[u8], slot: u32) -> Result<(), String> {
        if digest.len() != self.key_len {
            return Err(format!(
                "external key is {} byte(s); this table stores {}-byte digests",
                digest.len(),
                self.key_len
            ));
        }
        match self.entries.binary_search_by(|(k, _)| k.as_slice().cmp(digest)) {
            Ok(_) => Err(format!(
                "duplicate external key {} — one key must resolve to exactly one slot",
                hex(digest)
            )),
            Err(at) => {
                self.entries.insert(at, (digest.to_vec(), slot));
                Ok(())
            }
        }
    }

    /// The multihash code every key in this table is declared under.
    pub fn hash_code(&self) -> u32 {
        self.hash_code
    }

    /// The digest length, in bytes, every key in this table carries.
    pub fn key_len(&self) -> usize {
        self.key_len
    }

    /// The number of `key → slot` entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table carries no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The bound embedding provenance, if the producer declared one.
    pub fn provenance(&self) -> Option<&EmbeddingProvenance> {
        self.provenance.as_ref()
    }

    /// The opaque signature area — **unverified**, and named so no caller can read its presence as
    /// an integrity claim. Empty when the producer attached none.
    pub fn unverified_signature(&self) -> &[u8] {
        &self.signature
    }

    /// The `(digest, slot)` entries in canonical ascending digest order.
    pub fn entries(&self) -> impl Iterator<Item = (&[u8], u32)> {
        self.entries.iter().map(|(k, s)| (k.as_slice(), *s))
    }

    /// Resolves an external key to its slot. The declared `hash_code` **and** the digest length are
    /// checked before the digest is compared: a lookup under a different multihash code, or a
    /// truncated digest, is an `Err` (fail-closed), never a miss that a caller could read as "this
    /// key is simply absent". `Ok(None)` means the key is genuinely not in the table.
    pub fn lookup(&self, hash_code: u32, digest: &[u8]) -> Result<Option<u32>, String> {
        if hash_code != self.hash_code {
            return Err(format!(
                "external-key multihash code mismatch: query declares 0x{:x}, table stores 0x{:x}",
                hash_code, self.hash_code
            ));
        }
        if digest.len() != self.key_len {
            return Err(format!(
                "external-key length mismatch: query digest is {} byte(s), table stores {}",
                digest.len(),
                self.key_len
            ));
        }
        Ok(self
            .entries
            .binary_search_by(|(k, _)| k.as_slice().cmp(digest))
            .ok()
            .map(|at| self.entries[at].1))
    }

    /// [`lookup`](Self::lookup) over a whole binary multihash (`<code varint><length varint><digest>`),
    /// decoded by [`parse_multihash`]. A malformed multihash is an `Err`, so a caller never silently
    /// searches for a mis-decoded key.
    pub fn lookup_multihash(&self, multihash: &[u8]) -> Result<Option<u32>, String> {
        let (code, digest) = parse_multihash(multihash)?;
        self.lookup(code, digest)
    }

    /// Serializes the table to the version-0 block described in the module docs. Entries are
    /// written in ascending digest order, so two producers holding the same set of `(key, slot)`
    /// pairs, provenance and signature emit **byte-identical** blocks — the property the
    /// cross-repository fixtures pin.
    pub fn to_bytes(&self) -> Vec<u8> {
        let prov = self.provenance.as_ref().map(|p| p.to_bytes()).unwrap_or_default();
        let mut out = Vec::with_capacity(
            HEADER_PREFIX_LEN
                + prov.len()
                + self.entries.len() * (self.key_len + 4)
                + 4
                + self.signature.len(),
        );
        out.extend_from_slice(&EXTERNAL_KEY_MAGIC);
        out.extend_from_slice(&EXTERNAL_KEY_PROFILE_VERSION.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // flags — reserved, always 0
        out.extend_from_slice(&self.hash_code.to_le_bytes());
        out.extend_from_slice(&(self.key_len as u32).to_le_bytes());
        out.extend_from_slice(&(self.entries.len() as u64).to_le_bytes());
        out.extend_from_slice(&(prov.len() as u32).to_le_bytes());
        out.extend_from_slice(&prov);
        for (digest, slot) in &self.entries {
            out.extend_from_slice(digest);
            out.extend_from_slice(&slot.to_le_bytes());
        }
        out.extend_from_slice(&(self.signature.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.signature);
        out
    }

    /// Parses a version-0 block. Fail-closed on every malformation listed in the module docs; the
    /// returned table always satisfies the strictly-ascending, duplicate-free invariant, so
    /// [`lookup`](Self::lookup)'s binary search is sound on any table this function returns.
    pub fn from_bytes(bytes: &[u8]) -> Result<ExternalKeyTable, String> {
        if bytes.len() < HEADER_PREFIX_LEN {
            return Err(format!(
                "truncated external-key block: {} byte(s), need at least {} for the header",
                bytes.len(),
                HEADER_PREFIX_LEN
            ));
        }
        if bytes[0..8] != EXTERNAL_KEY_MAGIC {
            return Err("bad external-key magic (expected \"SPQVXKEY\")".to_string());
        }
        let version = u16::from_le_bytes([bytes[8], bytes[9]]);
        if version != EXTERNAL_KEY_PROFILE_VERSION {
            return Err(format!(
                "unsupported external-key profile version {} (this build reads/writes the DRAFT \
                 version {}; the frozen profile is #1746 and is not implemented here)",
                version, EXTERNAL_KEY_PROFILE_VERSION
            ));
        }
        let flags = u16::from_le_bytes([bytes[10], bytes[11]]);
        if flags != 0 {
            return Err(format!(
                "external-key flags field is 0x{:x}; every bit is reserved and MUST be 0",
                flags
            ));
        }
        let hash_code = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]);
        let key_len = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) as usize;
        check_key_len(key_len)?;
        let count = u64::from_le_bytes([
            bytes[20], bytes[21], bytes[22], bytes[23], bytes[24], bytes[25], bytes[26], bytes[27],
        ]);
        let prov_len = u32::from_le_bytes([bytes[28], bytes[29], bytes[30], bytes[31]]) as usize;

        let mut pos = HEADER_PREFIX_LEN;
        let provenance = if prov_len == 0 {
            None
        } else {
            let block = take(bytes, &mut pos, prov_len, "provenance block")?;
            Some(EmbeddingProvenance::from_bytes(block).map_err(|e| {
                format!("external-key provenance block does not parse: {}", e)
            })?)
        };

        // Bound the entry section arithmetically BEFORE reserving anything: a corrupt `count` of
        // u64::MAX must be an error, not an allocation attempt.
        let entry_width = key_len + 4;
        let entry_bytes = (count as usize)
            .checked_mul(entry_width)
            .filter(|_| count <= u64::from(u32::MAX))
            .ok_or_else(|| {
                format!("external-key entry count {} overruns the addressable block", count)
            })?;
        let section = take(bytes, &mut pos, entry_bytes, "entry section")?;
        let mut entries: Vec<(Vec<u8>, u32)> = Vec::with_capacity(count as usize);
        for chunk in section.chunks_exact(entry_width) {
            let (digest, slot) = chunk.split_at(key_len);
            if let Some((prev, _)) = entries.last() {
                if prev.as_slice() >= digest {
                    return Err(format!(
                        "external-key entries must be strictly ascending by digest: {} does not \
                         follow {} (an unsorted table or a duplicate key)",
                        hex(digest),
                        hex(prev)
                    ));
                }
            }
            let slot = u32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]);
            entries.push((digest.to_vec(), slot));
        }

        let sig_len_bytes = take(bytes, &mut pos, 4, "signature length")?;
        let sig_len = u32::from_le_bytes([
            sig_len_bytes[0],
            sig_len_bytes[1],
            sig_len_bytes[2],
            sig_len_bytes[3],
        ]) as usize;
        if sig_len > MAX_EXTERNAL_KEY_SIGNATURE_LEN {
            return Err(format!(
                "external-key signature area declares {} byte(s), above the {}-byte cap",
                sig_len, MAX_EXTERNAL_KEY_SIGNATURE_LEN
            ));
        }
        let signature = take(bytes, &mut pos, sig_len, "signature area")?.to_vec();

        if pos != bytes.len() {
            return Err(format!(
                "external-key block has {} trailing byte(s) after the signature area",
                bytes.len() - pos
            ));
        }
        Ok(ExternalKeyTable {
            hash_code,
            key_len,
            entries,
            provenance,
            signature,
        })
    }
}

/// Decodes a binary multihash — `<code unsigned-varint><length unsigned-varint><digest>` — into its
/// `(code, digest)` parts.
///
/// This is the **normalization** step of the profile: a key received in any multibase text form is
/// decoded to these bytes before it is ever compared, so two spellings of the same digest can never
/// read as two different keys. Fail-closed: a truncated varint, a **non-minimally encoded** varint
/// (canonical encoding is required so one digest has exactly one binary form), a code that does not
/// fit in `u32`, a declared length that disagrees with the bytes present, or trailing bytes after
/// the digest are all an `Err`.
pub fn parse_multihash(bytes: &[u8]) -> Result<(u32, &[u8]), String> {
    let (code, rest) = read_varint(bytes, "multihash code")?;
    let code = u32::try_from(code)
        .map_err(|_| format!("multihash code {} does not fit in u32", code))?;
    let (len, rest) = read_varint(rest, "multihash digest length")?;
    let len = usize::try_from(len)
        .map_err(|_| format!("multihash digest length {} is not addressable", len))?;
    if rest.len() != len {
        return Err(format!(
            "multihash declares a {}-byte digest but carries {} byte(s)",
            len,
            rest.len()
        ));
    }
    Ok((code, rest))
}

/// Reads one canonical unsigned-varint (LEB128, little-endian groups of 7 bits) and returns it with
/// the remaining bytes. Rejects a non-minimal encoding and an over-long (`> 64`-bit) value.
fn read_varint<'a>(bytes: &'a [u8], what: &str) -> Result<(u64, &'a [u8]), String> {
    let mut value: u64 = 0;
    for (i, &byte) in bytes.iter().enumerate() {
        if i >= 10 {
            return Err(format!("{} varint is longer than 10 bytes", what));
        }
        let payload = u64::from(byte & 0x7f);
        value |= payload
            .checked_shl(7 * i as u32)
            .ok_or_else(|| format!("{} varint overflows u64", what))?;
        if byte & 0x80 == 0 {
            // Canonical encoding: a continuation byte whose payload is 0 adds nothing, so the
            // shorter encoding was the only legal one.
            if i > 0 && payload == 0 {
                return Err(format!("{} varint is not minimally encoded", what));
            }
            return Ok((value, &bytes[i + 1..]));
        }
    }
    Err(format!("truncated {} varint", what))
}

fn check_key_len(key_len: usize) -> Result<(), String> {
    if key_len == 0 {
        return Err("external-key digest length must not be 0".to_string());
    }
    if key_len > MAX_EXTERNAL_KEY_LEN {
        return Err(format!(
            "external-key digest length {} exceeds the {}-byte cap",
            key_len, MAX_EXTERNAL_KEY_LEN
        ));
    }
    Ok(())
}

/// Bounds-checked forward slice. Every read validates before slicing, so a malformed block is a
/// descriptive `Err`, never an out-of-bounds panic.
fn take<'a>(bytes: &'a [u8], pos: &mut usize, n: usize, what: &str) -> Result<&'a [u8], String> {
    let end = pos
        .checked_add(n)
        .ok_or_else(|| format!("external-key {} length overflow", what))?;
    if end > bytes.len() {
        return Err(format!(
            "truncated external-key {}: need {} byte(s) at offset {}, have {}",
            what,
            n,
            *pos,
            bytes.len() - *pos
        ));
    }
    let slice = &bytes[*pos..end];
    *pos = end;
    Ok(slice)
}

/// Lower-case hex, for error messages only.
fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spqv_provenance::{Metric, Normalization};

    fn digest(seed: u8) -> Vec<u8> {
        (0..32u8).map(|i| i.wrapping_mul(7).wrapping_add(seed)).collect()
    }

    /// sha2-256 in the multicodec table. Used only as a realistic opaque value — this crate
    /// privileges no hash function and never computes one.
    const SHA2_256: u32 = 0x12;

    fn table() -> ExternalKeyTable {
        let mut t = ExternalKeyTable::new(SHA2_256, 32).unwrap();
        t.insert(&digest(3), 2).unwrap();
        t.insert(&digest(1), 0).unwrap();
        t.insert(&digest(2), 1).unwrap();
        t
    }

    #[test]
    fn round_trips_through_bytes() {
        let t = table();
        assert_eq!(ExternalKeyTable::from_bytes(&t.to_bytes()).unwrap(), t);
    }

    #[test]
    fn entries_are_written_in_ascending_digest_order() {
        // The canonical order is the cross-implementation property: insertion order must not leak
        // into the bytes, or two producers of the same table would disagree.
        let mut reversed = ExternalKeyTable::new(SHA2_256, 32).unwrap();
        reversed.insert(&digest(2), 1).unwrap();
        reversed.insert(&digest(3), 2).unwrap();
        reversed.insert(&digest(1), 0).unwrap();
        assert_eq!(reversed.to_bytes(), table().to_bytes());

        let parsed = ExternalKeyTable::from_bytes(&table().to_bytes()).unwrap();
        let keys: Vec<Vec<u8>> = parsed.entries().map(|(k, _)| k.to_vec()).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
        assert_eq!(parsed.len(), 3);
        assert!(!parsed.is_empty());
        assert_eq!(parsed.key_len(), 32);
        assert_eq!(parsed.hash_code(), SHA2_256);
    }

    #[test]
    fn lookup_resolves_present_keys_and_misses_absent_ones() {
        let t = table();
        assert_eq!(t.lookup(SHA2_256, &digest(1)).unwrap(), Some(0));
        assert_eq!(t.lookup(SHA2_256, &digest(2)).unwrap(), Some(1));
        assert_eq!(t.lookup(SHA2_256, &digest(3)).unwrap(), Some(2));
        assert_eq!(t.lookup(SHA2_256, &digest(9)).unwrap(), None);
    }

    #[test]
    fn lookup_rejects_a_different_multihash_code() {
        // The substitution guard: the SAME digest bytes declared under a weaker/other hash code
        // must NOT resolve. An Err, not a miss — "absent" and "unverifiable" are different answers.
        let err = table().lookup(0x13, &digest(1)).unwrap_err();
        assert!(err.contains("code mismatch"), "got: {}", err);
    }

    #[test]
    fn lookup_rejects_a_wrong_length_digest() {
        let err = table().lookup(SHA2_256, &digest(1)[..16]).unwrap_err();
        assert!(err.contains("length mismatch"), "got: {}", err);
    }

    #[test]
    fn insert_rejects_a_duplicate_key() {
        let mut t = table();
        let err = t.insert(&digest(1), 7).unwrap_err();
        assert!(err.contains("duplicate"), "got: {}", err);
        assert_eq!(t.lookup(SHA2_256, &digest(1)).unwrap(), Some(0), "slot unchanged");
    }

    #[test]
    fn insert_rejects_a_wrong_length_key() {
        let mut t = ExternalKeyTable::new(SHA2_256, 32).unwrap();
        assert!(t.insert(&[0u8; 31], 0).is_err());
    }

    #[test]
    fn new_rejects_an_out_of_range_key_length() {
        assert!(ExternalKeyTable::new(SHA2_256, 0).is_err());
        assert!(ExternalKeyTable::new(SHA2_256, MAX_EXTERNAL_KEY_LEN + 1).is_err());
        assert!(ExternalKeyTable::new(SHA2_256, MAX_EXTERNAL_KEY_LEN).is_ok());
    }

    #[test]
    fn provenance_and_signature_round_trip() {
        let prov = EmbeddingProvenance::new("m", Metric::Cosine, Normalization::L2);
        let t = table()
            .with_provenance(prov.clone())
            .with_unverified_signature(vec![0xAB, 0xCD])
            .unwrap();
        let back = ExternalKeyTable::from_bytes(&t.to_bytes()).unwrap();
        assert_eq!(back.provenance(), Some(&prov));
        assert_eq!(back.unverified_signature(), &[0xAB, 0xCD]);
        // Absent by default — and an absent signature is an empty slice, not a claim.
        assert_eq!(table().provenance(), None);
        assert!(table().unverified_signature().is_empty());
    }

    #[test]
    fn signature_area_is_capped() {
        let over = vec![0u8; MAX_EXTERNAL_KEY_SIGNATURE_LEN + 1];
        assert!(table().with_unverified_signature(over).is_err());
    }

    #[test]
    fn empty_table_round_trips() {
        let t = ExternalKeyTable::new(SHA2_256, 32).unwrap();
        assert!(t.is_empty());
        assert_eq!(ExternalKeyTable::from_bytes(&t.to_bytes()).unwrap(), t);
    }

    #[test]
    fn from_bytes_rejects_bad_magic() {
        let mut b = table().to_bytes();
        b[0] = b'X';
        assert!(ExternalKeyTable::from_bytes(&b).unwrap_err().contains("magic"));
    }

    #[test]
    fn from_bytes_rejects_the_frozen_profile_version() {
        // A file written against the FROZEN profile (version >= 1) must fail LOUDLY here rather
        // than be re-interpreted under draft rules.
        let mut b = table().to_bytes();
        b[8..10].copy_from_slice(&1u16.to_le_bytes());
        let err = ExternalKeyTable::from_bytes(&b).unwrap_err();
        assert!(err.contains("version"), "got: {}", err);
    }

    #[test]
    fn from_bytes_rejects_nonzero_flags() {
        let mut b = table().to_bytes();
        b[10..12].copy_from_slice(&1u16.to_le_bytes());
        assert!(ExternalKeyTable::from_bytes(&b).unwrap_err().contains("reserved"));
    }

    #[test]
    fn from_bytes_rejects_unsorted_and_duplicate_entries() {
        let bytes = table().to_bytes();
        let first = HEADER_PREFIX_LEN;
        let width = 36;
        // Swap entries 0 and 1 → descending at that step.
        let mut unsorted = bytes.clone();
        unsorted[first..first + width].copy_from_slice(&bytes[first + width..first + 2 * width]);
        unsorted[first + width..first + 2 * width].copy_from_slice(&bytes[first..first + width]);
        assert!(ExternalKeyTable::from_bytes(&unsorted)
            .unwrap_err()
            .contains("strictly ascending"));

        // Copy entry 0's digest over entry 1 → an exact duplicate.
        let mut dup = bytes.clone();
        dup[first + width..first + width + 32].copy_from_slice(&bytes[first..first + 32]);
        assert!(ExternalKeyTable::from_bytes(&dup)
            .unwrap_err()
            .contains("strictly ascending"));
    }

    #[test]
    fn from_bytes_rejects_truncation_and_trailing_bytes() {
        let bytes = table().to_bytes();
        let err = ExternalKeyTable::from_bytes(&bytes[..bytes.len() - 5]).unwrap_err();
        assert!(err.contains("truncated"), "got: {}", err);
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(ExternalKeyTable::from_bytes(&extra).unwrap_err().contains("trailing"));
        assert!(ExternalKeyTable::from_bytes(&[]).unwrap_err().contains("truncated"));
    }

    #[test]
    fn from_bytes_rejects_an_absurd_entry_count_without_allocating() {
        let mut b = table().to_bytes();
        b[20..28].copy_from_slice(&u64::MAX.to_le_bytes());
        let err = ExternalKeyTable::from_bytes(&b).unwrap_err();
        assert!(err.contains("overruns") || err.contains("truncated"), "got: {}", err);
    }

    #[test]
    fn from_bytes_rejects_an_out_of_range_key_length() {
        let mut zero = table().to_bytes();
        zero[16..20].copy_from_slice(&0u32.to_le_bytes());
        assert!(ExternalKeyTable::from_bytes(&zero).unwrap_err().contains("must not be 0"));
        let mut big = table().to_bytes();
        big[16..20].copy_from_slice(&(MAX_EXTERNAL_KEY_LEN as u32 + 1).to_le_bytes());
        assert!(ExternalKeyTable::from_bytes(&big).unwrap_err().contains("cap"));
    }

    #[test]
    fn from_bytes_rejects_a_corrupt_provenance_block() {
        let t = table().with_provenance(EmbeddingProvenance::new(
            "m",
            Metric::Cosine,
            Normalization::L2,
        ));
        let mut b = t.to_bytes();
        // The metric tag sits two bytes into the provenance block, which starts at the header end.
        b[HEADER_PREFIX_LEN + 2] = 200;
        let err = ExternalKeyTable::from_bytes(&b).unwrap_err();
        assert!(err.contains("provenance block does not parse"), "got: {}", err);
    }

    #[test]
    fn from_bytes_rejects_an_oversized_signature_length() {
        let bytes = table().to_bytes();
        let sig_at = bytes.len() - 4;
        let mut b = bytes.clone();
        b[sig_at..sig_at + 4].copy_from_slice(&(MAX_EXTERNAL_KEY_SIGNATURE_LEN as u32 + 1).to_le_bytes());
        assert!(ExternalKeyTable::from_bytes(&b).unwrap_err().contains("cap"));
    }

    #[test]
    fn parse_multihash_decodes_code_and_digest() {
        // sha2-256 (0x12) at 32 bytes — both varints are single-byte here.
        let mut mh = vec![0x12, 32];
        mh.extend_from_slice(&digest(1));
        let (code, d) = parse_multihash(&mh).unwrap();
        assert_eq!(code, SHA2_256);
        assert_eq!(d, digest(1).as_slice());

        // A two-byte varint code (blake3 = 0x1e is single-byte; use 0x0100 to force continuation).
        let mut wide = vec![0x80, 0x02, 1, 0xAA];
        assert_eq!(parse_multihash(&wide).unwrap(), (0x100, &[0xAAu8][..]));
        wide.push(0xBB);
        assert!(parse_multihash(&wide).is_err(), "trailing byte must be rejected");
    }

    #[test]
    fn parse_multihash_is_fail_closed() {
        assert!(parse_multihash(&[]).unwrap_err().contains("truncated"));
        assert!(parse_multihash(&[0x12]).unwrap_err().contains("truncated"));
        // Declared 32 bytes, 2 present.
        assert!(parse_multihash(&[0x12, 32, 0, 0]).unwrap_err().contains("declares"));
        // Non-minimal encoding of 0x12: 0x92 0x00 — the continuation byte carries nothing.
        assert!(parse_multihash(&[0x92, 0x00, 1, 0xAA])
            .unwrap_err()
            .contains("minimally encoded"));
    }

    #[test]
    fn lookup_multihash_matches_the_decomposed_lookup() {
        let t = table();
        let mut mh = vec![0x12, 32];
        mh.extend_from_slice(&digest(2));
        assert_eq!(t.lookup_multihash(&mh).unwrap(), Some(1));
        assert!(t.lookup_multihash(&mh[..4]).is_err(), "a malformed multihash must Err");
    }
}
