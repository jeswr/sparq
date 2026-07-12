//! Deterministic codec for the opt-in per-vector metadata block in `.spqv` v4.
//!
//! Metadata is an opaque UTF-8 string keyed by vector id. Entries are encoded in ascending id
//! order, so hash-map insertion order cannot affect the store bytes.

use rustc_hash::FxHashMap;
use sparq_core::dict::Id;

pub(crate) const MAGIC: [u8; 4] = *b"SPQM";
const VERSION: u32 = 1;
const HEADER_LEN: usize = 16;

pub(crate) fn encode(metadata: &FxHashMap<Id, String>) -> Vec<u8> {
    let mut entries: Vec<_> = metadata.iter().collect();
    entries.sort_unstable_by_key(|&(id, _)| *id);

    let body_len = entries
        .iter()
        .map(|(_, value)| 8 + value.len())
        .sum::<usize>();
    let mut bytes = Vec::with_capacity(HEADER_LEN + body_len);
    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&(entries.len() as u64).to_le_bytes());
    for (id, value) in entries {
        bytes.extend_from_slice(&id.to_le_bytes());
        bytes.extend_from_slice(&(value.len() as u32).to_le_bytes());
        bytes.extend_from_slice(value.as_bytes());
    }
    bytes
}

pub(crate) fn decode(
    bytes: &[u8],
    vector_count: usize,
    origin: &str,
) -> Result<FxHashMap<Id, String>, String> {
    if bytes.len() < HEADER_LEN {
        return Err(format!("{origin}: truncated metadata block"));
    }
    if bytes[0..4] != MAGIC {
        return Err(format!("{origin}: invalid metadata block magic"));
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if version != VERSION {
        return Err(format!(
            "{origin}: unsupported metadata block version {version}"
        ));
    }
    let entry_count64 = u64::from_le_bytes(bytes[8..16].try_into().unwrap());
    let entry_count: usize = entry_count64
        .try_into()
        .map_err(|_| format!("{origin}: metadata entry count exceeds the address space"))?;
    if entry_count > vector_count {
        return Err(format!(
            "{origin}: metadata has {entry_count} entries for only {vector_count} vectors"
        ));
    }

    let mut metadata = FxHashMap::default();
    metadata.reserve(entry_count);
    let mut offset = HEADER_LEN;
    let mut previous_id = None;
    for entry in 0..entry_count {
        let header_end = offset
            .checked_add(8)
            .ok_or_else(|| format!("{origin}: metadata entry {entry} header offset overflows"))?;
        if header_end > bytes.len() {
            return Err(format!("{origin}: truncated metadata entry {entry} header"));
        }
        let id = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let value_len =
            u32::from_le_bytes(bytes[offset + 4..header_end].try_into().unwrap()) as usize;
        if previous_id.is_some_and(|previous| previous >= id) {
            return Err(format!(
                "{origin}: metadata entry {entry} (id {id}) is not strictly ascending"
            ));
        }
        previous_id = Some(id);
        offset = header_end;
        let value_end = offset
            .checked_add(value_len)
            .ok_or_else(|| format!("{origin}: metadata value length {value_len} overflows"))?;
        if value_end > bytes.len() {
            return Err(format!("{origin}: truncated metadata value for id {id}"));
        }
        let value = std::str::from_utf8(&bytes[offset..value_end])
            .map_err(|e| format!("{origin}: metadata for id {id} is not UTF-8: {e}"))?;
        metadata.insert(id, value.to_owned());
        offset = value_end;
    }
    if offset != bytes.len() {
        return Err(format!(
            "{origin}: metadata block has {} trailing bytes",
            bytes.len() - offset
        ));
    }
    Ok(metadata)
}
