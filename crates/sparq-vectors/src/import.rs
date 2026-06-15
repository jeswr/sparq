//! [OPUS-4.8] (sq-xsq9) Bulk import of **externally-computed** embeddings into a `.spqv`
//! [`VectorStore`].
//!
//! The in-process embed path ([`crate::embed_labels`] / [`crate::embed_entities`]) runs an
//! [`Embedder`](crate::Embedder) over the graph. This module is the *bring-your-own-embeddings*
//! path: load an embedding matrix that was computed elsewhere (a Python/PyTorch/sentence-transformers
//! pipeline, a vendor batch job, …) and write it into a `.spqv` keyed by dictionary term id, without
//! ever running a model in this process. It **reuses the existing store writer**
//! ([`VectorStore::create`] + [`put`](VectorStore::put) + [`finalize`](VectorStore::finalize)) — the
//! file format, alignment, validation and fingerprint header are exactly what the embed path produces.
//!
//! # Row → dict-id contract
//!
//! An external matrix is just rows of numbers; it carries no term identity. The caller therefore
//! supplies a **parallel slice of dictionary ids** ([`ImportSpec::ids`]): row `i` of the matrix is
//! stored for term id `ids[i]`. The two MUST have the same length (`shape[0] == ids.len()`), and the
//! ids are subject to the same [`put`](VectorStore::put) rules as any other build — **no duplicate
//! ids**, and the per-row vector must be finite and non-zero. Resolve a `Term` to its id with
//! `graph.id_of(&term)`; the natural source of `ids` is the same scan that produced the texts you
//! embedded out-of-process (emit `(id, text)` pairs, embed the texts, then import the matrix with the
//! ids in the same order).
//!
//! # Formats
//!
//! * **`.npy`** ([`import_npy`](VectorStore::import_npy)) — a NumPy v1.0/v2.0/v3.0 array file. The
//!   header (`\x93NUMPY`, version, the `{ 'descr': …, 'fortran_order': …, 'shape': … }` dict) is
//!   parsed and validated; only a **2-D, C-order (`fortran_order: False`), little-endian `f4`/`f8`**
//!   array is accepted (`<f4`/`<f8`/`|f4`/`=f4` etc.). `f8` rows are narrowed to `f32` on import (the
//!   store is `f32`). This is exactly what `numpy.save(path, arr.astype(np.float32))` writes for a
//!   row-per-entity embedding matrix.
//! * **plain numeric dump** ([`import_numeric_dump`](VectorStore::import_numeric_dump)) — a *header-less*
//!   flat little-endian binary file of `rows × dim` contiguous `f32` (row-major / C-order): exactly
//!   `rows·dim·4` bytes, nothing else. This is what `arr.astype('<f4').tofile(path)` writes, or
//!   `Vec<f32>` → bytes from any language. `rows` is taken from `ids.len()` and `dim` from the store,
//!   so the file length is fully determined and checked.
//!
//! # Fail-closed validation
//!
//! Every length/shape/dtype field is validated **before** the proportional *decode* allocation (the
//! flat `Vec<f32>` rows), mirroring the bounded-read hardening in [`crate::store`] / sparq-hdt
//! (sq-tzwa): a declared `.npy` shape is checked against the actual file length (`data_offset +
//! rows·dim·elem_size`) so a hostile header cannot trigger a multi-gigabyte decode pre-allocation, and
//! a dtype/shape/order/dim/row-count mismatch is an `Err`, never a silent reinterpretation. The header
//! length itself is bounded to [`MAX_NPY_HEADER_LEN`] (64 KiB) so a malformed/oversized `HEADER_LEN` is
//! rejected up front. Note that the raw file bytes themselves are read fully into memory up front (via
//! `std::fs::read`); the validation above precedes the *decode* `Vec<f32>` allocation, not the file
//! read — see the peak-memory note below. [OPUS-4.8]
//!
//! # Peak memory
//!
//! These import paths are **not** streaming. `import_npy` / `import_numeric_dump` read the entire input
//! file into a `Vec<u8>` (`std::fs::read`) and then decode it into a flat `Vec<f32>`; `write_store`
//! ([`VectorStore::create`] + [`put`](VectorStore::put)) in turn buffers the whole dense payload in RAM
//! before [`finalize`](VectorStore::finalize). So peak resident memory is roughly *input file size +
//! decoded matrix* — on the order of **~2× the embedding matrix** (a little more for `f8` input, which
//! is read at 8 B/elem and narrowed to 4 B/elem). This is fine for typical embedding sets, but large
//! imports are memory-bound; a streaming import path that avoids holding the whole file + matrix at
//! once is tracked as a follow-up (sq-xsq9). [OPUS-4.8]
//!
//! # Graph binding
//!
//! Set [`ImportSpec::binding`] to [`ImportBinding::Graph`] to bind the resulting store to a graph (so
//! [`VectorStore::check_graph`] can later reject a stale store — see [`crate::fingerprint`]). Use the
//! SAME graph whose term ids `ids` come from. With [`ImportBinding::Unbound`] (when no graph is on
//! hand) the store is left unbound (an all-zero fingerprint block, reported as "unverifiable" rather
//! than a spurious mismatch).

use crate::store::VectorStore;
use sparq_core::dict::Id;
use sparq_core::Graph;

/// The largest `.npy` header length this importer will accept, in bytes: **64 KiB** (`64 * 1024`). The
/// NumPy header is a tiny ASCII dict (`{'descr': '<f4', 'fortran_order': False, 'shape': (N, D), }` plus
/// padding to a 64-byte boundary); 64 KiB is already far larger than any real header. Bounding it keeps
/// a malformed or hostile declared `HEADER_LEN` from forcing a large pre-allocation before we have even
/// seen the body (the sq-tzwa fail-closed discipline). [OPUS-4.8]
pub const MAX_NPY_HEADER_LEN: usize = 64 * 1024;

/// [OPUS-4.8] (sq-xsq9) How to bind a bulk-imported store to its source graph.
///
/// (No `Debug`: [`sparq_core::Graph`] is not `Debug`, and a graph is not meaningfully printable here.)
#[derive(Clone, Copy, Default)]
pub enum ImportBinding<'g> {
    /// Leave the store's graph fingerprint unbound (an all-zero block, reported as "unverifiable" by
    /// [`VectorStore::check_graph`]). The default — use it only when no graph is on hand at import.
    #[default]
    Unbound,
    /// Bind the store to `graph`'s fingerprint (via [`VectorStore::with_fingerprint`]). Pass the SAME
    /// graph whose term ids the rows are keyed by, so a later stale-store query is caught.
    Graph(&'g Graph),
}

/// [OPUS-4.8] (sq-xsq9) The contract for one bulk import: where to write the `.spqv`, the per-row dict
/// ids, and the optional graph binding.
///
/// `ids[i]` is the dictionary term id for row `i` of the imported matrix (the **row → dict-id
/// contract** — see the [module docs](self)). `ids.len()` is the number of rows; it must equal the
/// matrix's `shape[0]` (`.npy`) or determine the file length (numeric dump).
pub struct ImportSpec<'a, P: Into<std::path::PathBuf>> {
    /// Destination path for the finalized `.spqv` store.
    pub spqv_path: P,
    /// Vector dimension — must equal the matrix's `shape[1]` (`.npy`) / the per-row width (dump). The
    /// created store fixes this dim.
    pub dim: usize,
    /// Per-row dictionary ids: `ids[i]` keys row `i`. Length = row count. Must be free of duplicates.
    pub ids: &'a [Id],
    /// Optional graph binding for the fingerprint header.
    pub binding: ImportBinding<'a>,
}

/// The NumPy dtype an array declares, narrowed to the little-endian float kinds this importer reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NpyDtype {
    /// `<f4` / `|f4` / `=f4`: little-endian 32-bit float.
    F32,
    /// `<f8` / `|f8` / `=f8`: little-endian 64-bit float (narrowed to f32 on import).
    F64,
}

impl NpyDtype {
    fn elem_size(self) -> usize {
        match self {
            NpyDtype::F32 => 4,
            NpyDtype::F64 => 8,
        }
    }
}

/// The validated, parsed `.npy` header: dtype, the 2-D `(rows, dim)` shape, and the byte offset where
/// the array data begins. C-order only; big-endian and non-2-D shapes are rejected during parsing.
#[derive(Clone, Copy, Debug)]
struct NpyHeader {
    dtype: NpyDtype,
    rows: usize,
    dim: usize,
    data_offset: usize,
}

/// [OPUS-4.8] (sq-xsq9) Parse and fully validate a `.npy` header from the leading bytes of the file.
///
/// `file_len` is the total length of the file the header came from; it is used to bound the declared
/// `HEADER_LEN` and to let the caller cross-check the declared shape against the real body size before
/// allocating. This function itself touches only the header bytes (≤ [`MAX_NPY_HEADER_LEN`] + the
/// fixed prefix); it never reads or allocates the array body.
fn parse_npy_header(buf: &[u8], file_len: usize) -> Result<NpyHeader, String> {
    // Magic: \x93 N U M P Y, then major.minor version bytes.
    const MAGIC: &[u8] = b"\x93NUMPY";
    if buf.len() < MAGIC.len() + 2 {
        return Err("npy: truncated magic/version".into());
    }
    if &buf[0..6] != MAGIC {
        return Err("npy: bad magic (not a NumPy .npy file)".into());
    }
    let major = buf[6];
    // v1.0 uses a u16 header length; v2.0/v3.0 use a u32. minor (buf[7]) is not load-bearing here.
    let (header_len_field_size, prefix_len) = match major {
        1 => (2usize, 10usize),     // 6 magic + 2 version + 2 len
        2 | 3 => (4usize, 12usize), // 6 magic + 2 version + 4 len
        v => return Err(format!("npy: unsupported major version {v}")),
    };
    if buf.len() < prefix_len {
        return Err("npy: truncated header-length field".into());
    }
    let header_len = match header_len_field_size {
        2 => u16::from_le_bytes(buf[8..10].try_into().unwrap()) as usize,
        _ => u32::from_le_bytes(buf[8..12].try_into().unwrap()) as usize,
    };
    // Bound the declared header length BEFORE trusting it to slice/allocate (sq-tzwa discipline).
    if header_len > MAX_NPY_HEADER_LEN {
        return Err(format!(
            "npy: declared header length {header_len} exceeds the {MAX_NPY_HEADER_LEN}-byte cap"
        ));
    }
    let data_offset = prefix_len
        .checked_add(header_len)
        .ok_or("npy: header length overflows")?;
    if data_offset > file_len {
        return Err(format!(
            "npy: header (offset {data_offset}) extends past the {file_len}-byte file"
        ));
    }
    if buf.len() < data_offset {
        return Err("npy: header dict truncated".into());
    }
    let dict = std::str::from_utf8(&buf[prefix_len..data_offset])
        .map_err(|_| "npy: header dict is not UTF-8/ASCII".to_string())?;

    let dtype = parse_descr(dict)?;
    if header_field(dict, "fortran_order")?.trim() != "False" {
        return Err("npy: fortran_order=True is unsupported; save a C-order array".into());
    }
    let (rows, dim) = parse_shape(dict)?;
    Ok(NpyHeader {
        dtype,
        rows,
        dim,
        data_offset,
    })
}

/// Extract the raw value text for `key` from a NumPy header dict (`'key': value`). Robust to the
/// single/double quote and whitespace variants NumPy emits; does not attempt to be a full Python
/// literal parser (the header grammar is fixed and tiny).
fn header_field<'d>(dict: &'d str, key: &str) -> Result<&'d str, String> {
    // Find `'key'` or `"key"`, then the `:` and the value up to the next top-level comma/brace.
    for q in ['\'', '"'] {
        let needle = format!("{q}{key}{q}");
        if let Some(start) = dict.find(&needle) {
            let after = &dict[start + needle.len()..];
            let colon = after
                .find(':')
                .ok_or_else(|| format!("npy: no ':' after '{key}'"))?;
            let val = &after[colon + 1..];
            // Value ends at the comma that separates dict entries, or the closing brace.
            let end = val
                .char_indices()
                .scan(0i32, |depth, (i, c)| {
                    match c {
                        '(' | '[' => *depth += 1,
                        ')' | ']' => *depth -= 1,
                        ',' if *depth == 0 => return Some((i, true)),
                        '}' if *depth == 0 => return Some((i, true)),
                        _ => {}
                    }
                    Some((i, false))
                })
                .find(|&(_, stop)| stop)
                .map(|(i, _)| i)
                .unwrap_or(val.len());
            return Ok(&val[..end]);
        }
    }
    Err(format!("npy: header dict has no '{key}' field"))
}

/// Validate the `descr` dtype string: little-endian (or not-relevant for 1-byte… but floats are ≥4
/// bytes) `f4`/`f8`. Big-endian (`>`) is rejected — fail closed rather than silently byte-swap.
fn parse_descr(dict: &str) -> Result<NpyDtype, String> {
    let raw = header_field(dict, "descr")?;
    // Strip the surrounding quotes and whitespace, e.g. `'<f4'` -> `<f4`.
    let descr = raw.trim().trim_matches(|c| c == '\'' || c == '"').trim();
    // Byte-order char: '<' little, '>' big, '=' native, '|' not-applicable. On a little-endian
    // target (the only target `.spqv` supports) '<', '=' and '|' are all little-endian.
    let (order, kind) = descr.split_at(descr.chars().next().map_or(0, |c| c.len_utf8()));
    match order {
        "<" | "=" | "|" => {}
        ">" => return Err(format!("npy: big-endian dtype '{descr}' is unsupported")),
        _ => {
            // No explicit byte-order prefix (e.g. "f4"): treat the whole string as the kind.
            return classify_kind(descr).ok_or_else(|| format!("npy: unsupported dtype '{descr}'"));
        }
    }
    classify_kind(kind).ok_or_else(|| {
        format!("npy: unsupported dtype '{descr}' (only little-endian f4/f8 are read)")
    })
}

fn classify_kind(kind: &str) -> Option<NpyDtype> {
    match kind {
        "f4" => Some(NpyDtype::F32),
        "f8" => Some(NpyDtype::F64),
        _ => None,
    }
}

/// Parse the `shape` tuple; only a 2-D `(rows, dim)` is accepted (a flat 1-D matrix carries no row
/// boundary, and >2-D is not an embedding matrix).
fn parse_shape(dict: &str) -> Result<(usize, usize), String> {
    let raw = header_field(dict, "shape")?;
    let inner = raw
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();
    let dims: Vec<usize> = inner
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<usize>()
                .map_err(|_| format!("npy: non-integer shape dim '{s}'"))
        })
        .collect::<Result<_, _>>()?;
    match dims.as_slice() {
        [rows, dim] => Ok((*rows, *dim)),
        other => Err(format!(
            "npy: expected a 2-D (rows, dim) shape, got {}-D",
            other.len()
        )),
    }
}

/// [OPUS-4.8] (sq-xsq9) Decode the array body of a validated `.npy` into row-major `f32` rows. The
/// header has already bounded `rows·dim` against the file length, so the `with_capacity` here is safe.
fn decode_npy_body(header: &NpyHeader, body: &[u8]) -> Result<Vec<f32>, String> {
    let elem = header.dtype.elem_size();
    let want = header
        .rows
        .checked_mul(header.dim)
        .and_then(|n| n.checked_mul(elem))
        .ok_or("npy: rows*dim*elem_size overflows")?;
    if body.len() != want {
        return Err(format!(
            "npy: array body is {} bytes, expected {want} for shape ({}, {}) dtype {:?}",
            body.len(),
            header.rows,
            header.dim,
            header.dtype
        ));
    }
    let count = header.rows * header.dim;
    let mut out = Vec::with_capacity(count);
    match header.dtype {
        NpyDtype::F32 => {
            for chunk in body.chunks_exact(4) {
                out.push(f32::from_le_bytes(chunk.try_into().unwrap()));
            }
        }
        NpyDtype::F64 => {
            for chunk in body.chunks_exact(8) {
                out.push(f64::from_le_bytes(chunk.try_into().unwrap()) as f32);
            }
        }
    }
    Ok(out)
}

/// Build the `.spqv` from already-decoded row-major `f32` rows (`rows × dim`) and the parallel ids,
/// reusing the standard store writer. Shared by both import formats.
///
/// [OPUS-4.8] This buffers the entire dense payload in RAM: [`VectorStore::create`] +
/// [`put`](VectorStore::put) accumulate all rows, then [`finalize`](VectorStore::finalize) writes them
/// out. Combined with the caller already holding the decoded `flat` matrix (and the import paths the
/// whole input file), peak resident memory is ~2× the embedding matrix — see the [module peak-memory
/// note](self#peak-memory).
fn write_store<P: Into<std::path::PathBuf>>(
    spqv_path: P,
    dim: usize,
    ids: &[Id],
    rows: usize,
    flat: &[f32],
    binding: ImportBinding<'_>,
) -> Result<VectorStore, String> {
    if dim == 0 {
        return Err("import: zero dimension".into());
    }
    if ids.len() != rows {
        return Err(format!(
            "import: {} ids supplied for {rows} matrix rows (one id per row required)",
            ids.len()
        ));
    }
    debug_assert_eq!(flat.len(), rows * dim);
    let mut store = match binding {
        ImportBinding::Unbound => VectorStore::create(spqv_path, dim)?,
        ImportBinding::Graph(g) => VectorStore::create(spqv_path, dim)?.with_fingerprint(g),
    };
    for (row, &id) in ids.iter().enumerate() {
        let v = &flat[row * dim..(row + 1) * dim];
        // `put` enforces dim/finite/non-zero/duplicate-id — fail closed on any bad row.
        store.put(id, v)?;
    }
    store.finalize()?;
    Ok(store)
}

impl VectorStore {
    /// [OPUS-4.8] (sq-xsq9) Bulk-import an externally-computed embedding matrix from a NumPy `.npy`
    /// file into a finalized `.spqv` store keyed by dictionary id.
    ///
    /// `spec.ids[i]` is the dict id for row `i` of the matrix (the row → dict-id contract). The file
    /// must be a **2-D, C-order, little-endian `f4`/`f8`** array whose `shape == (spec.ids.len(),
    /// spec.dim)`; any other dtype, byte-order, dimensionality, fortran-order or shape mismatch is a
    /// fail-closed `Err`. The whole file is read into memory up front (`std::fs::read`); the declared
    /// header length and shape are then validated against the actual file length **before the decode
    /// allocation** — i.e. before the flat `Vec<f32>` is allocated, so a hostile header cannot force a
    /// multi-gigabyte decode buffer (sq-tzwa hardening). Note the file-byte read itself is *not* gated
    /// by this check, and `write_store` buffers the whole dense payload before finalize, so peak RAM is
    /// roughly the file size + the decoded matrix (~2× the matrix) — see the [module peak-memory
    /// note](self#peak-memory). `f8` rows are narrowed to `f32`. [OPUS-4.8]
    ///
    /// Returns the finalized, memory-mapped store (same handle [`VectorStore::open`] would yield).
    ///
    /// ```no_run
    /// use sparq_vectors::{ImportBinding, ImportSpec, VectorStore};
    /// // rows of `vecs.npy` line up with `ids` positionally:
    /// let ids = [7u32, 42, 1000];
    /// let store = VectorStore::import_npy(
    ///     ImportSpec { spqv_path: "graph.spqv", dim: 384, ids: &ids, binding: ImportBinding::Unbound },
    ///     "vecs.npy",
    /// ).unwrap();
    /// assert_eq!(store.len(), 3);
    /// ```
    #[cfg(not(target_arch = "wasm32"))]
    pub fn import_npy<P, Q>(spec: ImportSpec<'_, P>, npy_path: Q) -> Result<VectorStore, String>
    where
        P: Into<std::path::PathBuf>,
        Q: AsRef<std::path::Path>,
    {
        let npy_path = npy_path.as_ref();
        let bytes =
            std::fs::read(npy_path).map_err(|e| format!("read {}: {e}", npy_path.display()))?;
        let header = parse_npy_header(&bytes, bytes.len())?;
        if header.dim != spec.dim {
            return Err(format!(
                "npy: array dim {} != store dim {} (shape ({}, {}))",
                header.dim, spec.dim, header.rows, header.dim
            ));
        }
        if header.rows != spec.ids.len() {
            return Err(format!(
                "npy: array has {} rows but {} ids were supplied (one id per row required)",
                header.rows,
                spec.ids.len()
            ));
        }
        let flat = decode_npy_body(&header, &bytes[header.data_offset..])?;
        write_store(
            spec.spqv_path,
            spec.dim,
            spec.ids,
            header.rows,
            &flat,
            spec.binding,
        )
    }

    /// [OPUS-4.8] (sq-xsq9) Bulk-import an externally-computed embedding matrix from a **header-less**
    /// flat little-endian `f32` dump into a finalized `.spqv` store keyed by dictionary id.
    ///
    /// The file must be exactly `spec.ids.len() · spec.dim · 4` bytes of contiguous row-major
    /// (C-order) `f32`: row `i` (bytes `i·dim·4 .. (i+1)·dim·4`) is stored for `spec.ids[i]`. A
    /// length that is not an exact multiple of the expected size — extra or missing bytes — is a
    /// fail-closed `Err` (so a truncated or padded file cannot be silently mis-sliced). This is the
    /// format `numpy.ndarray.astype('<f4').tofile(path)` writes, or raw `Vec<f32>` bytes from any
    /// language.
    ///
    /// The expected length is checked from `metadata` *before* the file is read, so a wrong-size file
    /// is rejected without a large read; but a correctly-sized file is then read fully into memory
    /// (`std::fs::read`) and decoded into a flat `Vec<f32>`, and `write_store` buffers the whole dense
    /// payload before finalize — peak RAM is roughly the file size + the decoded matrix (~2× the
    /// matrix). See the [module peak-memory note](self#peak-memory). [OPUS-4.8]
    ///
    /// Returns the finalized, memory-mapped store.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn import_numeric_dump<P, Q>(
        spec: ImportSpec<'_, P>,
        dump_path: Q,
    ) -> Result<VectorStore, String>
    where
        P: Into<std::path::PathBuf>,
        Q: AsRef<std::path::Path>,
    {
        if spec.dim == 0 {
            return Err("import: zero dimension".into());
        }
        let dump_path = dump_path.as_ref();
        let rows = spec.ids.len();
        // The expected file length is fully determined by ids.len() and dim — check it BEFORE reading
        // the whole file into the decode buffer (fail closed on a wrong size; no trust in file bytes
        // to size an allocation). [OPUS-4.8] sq-tzwa discipline.
        let expect = rows
            .checked_mul(spec.dim)
            .and_then(|n| n.checked_mul(4))
            .ok_or("dump: rows*dim*4 overflows")?;
        let meta = std::fs::metadata(dump_path)
            .map_err(|e| format!("stat {}: {e}", dump_path.display()))?;
        let file_len: usize = meta.len().try_into().map_err(|_| {
            format!(
                "dump: {} is larger than the address space",
                dump_path.display()
            )
        })?;
        if file_len != expect {
            return Err(format!(
                "dump: {} is {file_len} bytes, expected {expect} for {rows} rows x dim {} (f32)",
                dump_path.display(),
                spec.dim
            ));
        }
        let bytes =
            std::fs::read(dump_path).map_err(|e| format!("read {}: {e}", dump_path.display()))?;
        // metadata→read race: re-check the length we actually got.
        if bytes.len() != expect {
            return Err(format!(
                "dump: {} changed size during read ({} bytes, expected {expect})",
                dump_path.display(),
                bytes.len()
            ));
        }
        let mut flat = Vec::with_capacity(rows * spec.dim);
        for chunk in bytes.chunks_exact(4) {
            flat.push(f32::from_le_bytes(chunk.try_into().unwrap()));
        }
        write_store(
            spec.spqv_path,
            spec.dim,
            spec.ids,
            rows,
            &flat,
            spec.binding,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- header parser unit tests (no filesystem) ----

    /// Build a NumPy v1.0 header for the given dict body, padded as NumPy does (total prefix+header
    /// is a multiple of 64, header ends with '\n').
    fn npy_v1(dict: &str) -> Vec<u8> {
        let mut header = dict.as_bytes().to_vec();
        // pad so that 10 (prefix) + header_len is a multiple of 64, ending in '\n'.
        let unpadded = 10 + header.len() + 1;
        let pad = (64 - (unpadded % 64)) % 64;
        header.extend(std::iter::repeat_n(b' ', pad));
        header.push(b'\n');
        let mut out = Vec::new();
        out.extend_from_slice(b"\x93NUMPY");
        out.push(1);
        out.push(0);
        out.extend_from_slice(&(header.len() as u16).to_le_bytes());
        out.extend_from_slice(&header);
        out
    }

    #[test]
    fn parses_canonical_f4_header() {
        let h = npy_v1("{'descr': '<f4', 'fortran_order': False, 'shape': (3, 4), }");
        let file_len = h.len() + 3 * 4 * 4;
        let parsed = parse_npy_header(&h, file_len).unwrap();
        assert_eq!(parsed.dtype, NpyDtype::F32);
        assert_eq!((parsed.rows, parsed.dim), (3, 4));
        assert_eq!(parsed.data_offset, h.len());
    }

    #[test]
    fn parses_f8_and_byteorder_variants() {
        for descr in ["'<f8'", "'=f8'", "'|f8'"] {
            let h = npy_v1(&format!(
                "{{'descr': {descr}, 'fortran_order': False, 'shape': (1, 2), }}"
            ));
            let parsed = parse_npy_header(&h, h.len() + 16).unwrap();
            assert_eq!(parsed.dtype, NpyDtype::F64);
        }
    }

    #[test]
    fn rejects_big_endian_fortran_and_non_2d() {
        let be = npy_v1("{'descr': '>f4', 'fortran_order': False, 'shape': (1, 2), }");
        assert!(parse_npy_header(&be, be.len() + 8)
            .unwrap_err()
            .contains("big-endian"));

        let fo = npy_v1("{'descr': '<f4', 'fortran_order': True, 'shape': (1, 2), }");
        assert!(parse_npy_header(&fo, fo.len() + 8)
            .unwrap_err()
            .contains("fortran_order"));

        let oned = npy_v1("{'descr': '<f4', 'fortran_order': False, 'shape': (6,), }");
        assert!(parse_npy_header(&oned, oned.len() + 24)
            .unwrap_err()
            .contains("2-D"));

        let threed = npy_v1("{'descr': '<f4', 'fortran_order': False, 'shape': (1, 2, 3), }");
        assert!(parse_npy_header(&threed, threed.len() + 100)
            .unwrap_err()
            .contains("2-D"));

        let i4 = npy_v1("{'descr': '<i4', 'fortran_order': False, 'shape': (1, 2), }");
        assert!(parse_npy_header(&i4, i4.len() + 8)
            .unwrap_err()
            .contains("dtype"));
    }

    #[test]
    fn rejects_oversized_or_truncated_header() {
        // Hand-craft a v1 header whose declared HEADER_LEN is absurdly large: must fail closed on the
        // cap, NOT pre-allocate. (mirror sq-tzwa)
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"\x93NUMPY");
        bytes.push(1);
        bytes.push(0);
        bytes.extend_from_slice(&u16::MAX.to_le_bytes()); // declared header len 65535
        bytes.extend(std::iter::repeat_n(b' ', 100)); // but the file is tiny
        let err = parse_npy_header(&bytes, bytes.len()).unwrap_err();
        assert!(
            err.contains("cap") || err.contains("past the"),
            "err: {err}"
        );

        // A header that claims to extend past the file.
        let h = npy_v1("{'descr': '<f4', 'fortran_order': False, 'shape': (3, 4), }");
        let err2 = parse_npy_header(&h, h.len() - 1).unwrap_err();
        assert!(err2.contains("past the"), "err: {err2}");

        // Bad magic.
        assert!(parse_npy_header(b"not numpy at all here", 21)
            .unwrap_err()
            .contains("magic"));
    }

    #[test]
    fn decode_body_rejects_wrong_length() {
        let header = NpyHeader {
            dtype: NpyDtype::F32,
            rows: 2,
            dim: 3,
            data_offset: 0,
        };
        // 2*3*4 = 24 bytes expected.
        assert!(decode_npy_body(&header, &[0u8; 23])
            .unwrap_err()
            .contains("expected 24"));
        let body: Vec<u8> = (0..24).map(|_| 0u8).collect();
        // all-zero decodes fine at this layer (put() is what rejects zero vectors).
        assert_eq!(decode_npy_body(&header, &body).unwrap().len(), 6);
    }
}
