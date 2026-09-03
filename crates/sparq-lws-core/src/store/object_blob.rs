// AUTHORED-BY Claude Opus 5
//! The REAL [`BlobStore`] adapter over [`object_store`] — one implementation covering
//! S3 / GCS / Azure / local-filesystem / in-memory backends.
//!
//! Until this module landed, [`InMemoryBlobStore`](super::InMemoryBlobStore) was the ONLY
//! [`BlobStore`] impl, so the orphaned-bytes reconciler ([`super::reconcile`]) could not run against
//! a durable backend at all. [`ObjectStoreBlobStore`] closes that gap: it implements every
//! [`BlobStore`] method — including the two the reconciler needs, [`BlobStore::list`] (the
//! start-of-sweep snapshot) and [`BlobStore::stat`] (the pre-delete re-stat) — against a real
//! [`object_store::ObjectStore`].
//!
//! # Layout: one flat object per blob key, under an EXCLUSIVELY-OWNED prefix
//! A blob key is opaque and, as minted by `CompositeStore::mint_blob_key`, contains no `/` (the IRI's
//! separators are flattened to `_` before the random suffix is appended). This adapter therefore
//! stores each blob as ONE object at `<prefix>/<key>`, where the key is a single
//! [`object_store::path::PathPart`] — so a key that did contain a `/` is percent-encoded into one
//! segment rather than silently becoming a directory.
//!
//! **The prefix must be exclusively owned by this blob store.** [`BlobStore::list`] is the
//! reconciler's view of "which bytes physically exist", and anything it reports that SPARQ does not
//! reference is a GC candidate. Sharing the prefix with another writer would put that writer's
//! objects on the chopping block. Objects that this adapter could not itself have written — one
//! nested deeper than a single segment, or one whose segment is not valid percent-encoded UTF-8 — are
//! therefore **skipped, never reported and never deleted** (fail-safe: an object we cannot prove is
//! ours is one we must not reclaim).
//!
//! # `generation` is `None` here — and why that is the sound answer, not a shortcut
//! [`BlobEntry::generation`] is the compare-and-delete witness for
//! [`BlobStore::delete_if_unchanged`], and that method's contract is explicit that the compare and the
//! removal must happen in ONE critical section with no suspension point between them — a
//! `stat()`-then-`delete()` "CAS" is exactly the TOCTOU the method exists to close.
//!
//! `object_store` 0.14 exposes **no conditional or versioned delete**: its only delete primitives are
//! [`object_store::ObjectStore::delete_stream`] and the [`object_store::ObjectStoreExt::delete`]
//! wrapper over it, neither of which takes an `if_match` / version precondition. (Conditional
//! *writes* exist — [`object_store::PutMode::Update`] and [`object_store::GetOptions::if_match`] —
//! but a conditional put followed by an unconditional delete is two operations, not one, so it does
//! not close the window either.) There is consequently NO way to build an atomic CAS-delete on this
//! API, whatever native write version ([`ObjectMeta::e_tag`](object_store::ObjectMeta::e_tag) /
//! [`ObjectMeta::version`](object_store::ObjectMeta::version)) the backend reports.
//!
//! So this adapter reports `generation: None` rather than mapping the ETag/version into a witness it
//! cannot honour. That is not a loss of safety, and it is not a leak either — it is precisely the
//! versionless path the reconciler already documents (`reconcile.rs`, Finding 2): a versionless
//! orphan is still age-gated against the grace window, still re-checked against a FRESH referenced
//! set, and still re-statted immediately before the delete, and is then reclaimed with an
//! unconditional [`BlobStore::delete`]. That is sound because `CompositeStore::mint_blob_key` mints
//! UNIQUE-PER-WRITE keys: a recreate of the same IRI gets a DIFFERENT key, so a candidate's key can
//! never be a live recreate's key and there is nothing to clobber. Reporting a generation we could
//! not compare-and-delete on would have been the unsound choice.
//!
//! If a future `object_store` release grows a conditional delete, the change is local: report the
//! backend's native version as the `generation` and implement
//! [`delete_if_unchanged`](BlobStore::delete_if_unchanged) on it.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;
use object_store::path::Path as ObjectPath;
use object_store::{ObjectStore, ObjectStoreExt};

use super::{BlobEntry, BlobError, BlobStore};

/// A [`BlobStore`] backed by any [`object_store::ObjectStore`] — S3, GCS, Azure, the local
/// filesystem, or `object_store`'s own in-memory store.
///
/// Construct the backend with `object_store` itself and hand it over:
///
/// ```
/// use std::sync::Arc;
/// use sparq_lws_core::store::ObjectStoreBlobStore;
///
/// let backend = Arc::new(object_store::memory::InMemory::new());
/// let blobs = ObjectStoreBlobStore::with_prefix(backend, "pods/alice/blobs");
/// assert_eq!(blobs.prefix().as_ref(), "pods/alice/blobs");
/// ```
///
/// See the module docs for the object layout, the exclusive-prefix requirement, and why
/// [`BlobEntry::generation`] is always `None` on this backend.
#[derive(Debug, Clone)]
pub struct ObjectStoreBlobStore {
    store: Arc<dyn ObjectStore>,
    prefix: ObjectPath,
}

impl ObjectStoreBlobStore {
    /// Store blobs at the ROOT of `store`. Only correct when the bucket/container/directory is
    /// dedicated to this blob store — see the module docs on exclusive ownership.
    pub fn new(store: Arc<dyn ObjectStore>) -> Self {
        Self {
            store,
            prefix: ObjectPath::default(),
        }
    }

    /// Store blobs under `prefix` (a `/`-separated path, e.g. `"pods/alice/blobs"`). The prefix must
    /// be exclusively owned by this blob store — see the module docs.
    pub fn with_prefix(store: Arc<dyn ObjectStore>, prefix: &str) -> Self {
        Self {
            store,
            prefix: ObjectPath::from(prefix),
        }
    }

    /// The prefix every blob object is stored under.
    pub fn prefix(&self) -> &ObjectPath {
        &self.prefix
    }

    /// The object location for a blob key: the key becomes ONE path segment (percent-encoded by
    /// `object_store` if it contains a `/` or another reserved byte), so a key can never expand into
    /// a directory hierarchy.
    fn location(&self, key: &str) -> ObjectPath {
        self.prefix.clone().join(key)
    }

    /// The inverse of [`location`](Self::location): recover the blob key an object location encodes,
    /// or `None` if this adapter could not have written it — the location is outside the prefix,
    /// is nested deeper than one segment, or its segment is not valid percent-encoded UTF-8.
    /// `None` means "not one of ours", and such an object is skipped by [`BlobStore::list`] rather
    /// than offered to the reconciler as a GC candidate.
    fn key_of(&self, location: &ObjectPath) -> Option<String> {
        let mut parts = location.prefix_match(&self.prefix)?;
        let segment = parts.next()?;
        if parts.next().is_some() {
            // Nested below the prefix — `location()` never produces this shape.
            return None;
        }
        // `PathParts` yields the RAW (still percent-encoded) segment, so undo the encoding
        // `PathPart::from` applied on the way in.
        percent_decode(segment.as_ref())
    }

    /// Map an `object_store` error at a read/stat boundary, keeping the [`BlobError`] surface opaque
    /// to clients (the HTTP layer renders every `Storage` error as a generic 500) while retaining
    /// enough detail for an operator reading the logs.
    fn backend_error(operation: &str, error: object_store::Error) -> BlobError {
        BlobError::Backend(format!("object_store {operation} failed: {error}"))
    }

    /// The [`BlobEntry`] for one listed/statted object. `generation` is always `None` — see the
    /// module docs.
    fn entry(key: String, meta: &object_store::ObjectMeta) -> BlobEntry {
        BlobEntry {
            key,
            last_modified: system_time_from_epoch(
                meta.last_modified.timestamp(),
                meta.last_modified.timestamp_subsec_nanos(),
            ),
            generation: None,
        }
    }
}

#[async_trait]
impl BlobStore for ObjectStoreBlobStore {
    async fn get(&self, key: &str) -> Result<Bytes, BlobError> {
        let result = match self.store.get(&self.location(key)).await {
            Ok(result) => result,
            Err(object_store::Error::NotFound { .. }) => return Err(BlobError::NotFound),
            Err(error) => return Err(Self::backend_error("get", error)),
        };
        result
            .bytes()
            .await
            .map_err(|error| Self::backend_error("get body", error))
    }

    async fn put(&self, key: &str, body: Bytes) -> Result<(), BlobError> {
        self.store
            .put(&self.location(key), body.into())
            .await
            .map(|_| ())
            .map_err(|error| Self::backend_error("put", error))
    }

    async fn exists(&self, key: &str) -> Result<bool, BlobError> {
        match self.store.head(&self.location(key)).await {
            Ok(_) => Ok(true),
            Err(object_store::Error::NotFound { .. }) => Ok(false),
            Err(error) => Err(Self::backend_error("head", error)),
        }
    }

    /// Idempotent, per the trait contract: a backend that reports `NotFound` for an absent key (the
    /// local filesystem, GCS, Azure) is mapped to `Ok(())`, matching the backends that already return
    /// success (S3, the in-memory store). The authoritative existence decision lives in the index.
    async fn delete(&self, key: &str) -> Result<(), BlobError> {
        match self.store.delete(&self.location(key)).await {
            Ok(()) => Ok(()),
            Err(object_store::Error::NotFound { .. }) => Ok(()),
            Err(error) => Err(Self::backend_error("delete", error)),
        }
    }

    /// Enumerate every blob under the prefix via [`object_store::ObjectStore::list`], mapping each
    /// [`ObjectMeta`](object_store::ObjectMeta)'s `location` back to its blob key and its
    /// `last_modified` to a [`SystemTime`] (the reconciler's AGE witness for the grace window).
    ///
    /// A timestamp outside [`SystemTime`]'s representable range yields `last_modified: None`, which
    /// the reconciler treats fail-closed — such a blob is counted `skipped_unknown_age` and is NEVER
    /// GC'd, because we cannot prove it is old enough to be safe to delete.
    ///
    /// `generation` is always `None` here (see the module docs), so the reconciler takes the
    /// versionless unconditional-delete path, which is sound under unique-per-write blob keys.
    ///
    /// Objects under the prefix that this adapter could not have written are SKIPPED, not reported:
    /// reporting one would offer somebody else's data to the GC as an unreferenced orphan.
    async fn list(&self) -> Result<Vec<BlobEntry>, BlobError> {
        let mut stream = self.store.list(Some(&self.prefix));
        let mut entries = Vec::new();
        while let Some(item) = stream.next().await {
            let meta = item.map_err(|error| Self::backend_error("list", error))?;
            let Some(key) = self.key_of(&meta.location) else {
                continue;
            };
            entries.push(Self::entry(key, &meta));
        }
        Ok(entries)
    }

    /// The O(1) single-key re-stat the trait asks a HEAD-capable backend to provide, rather than the
    /// default's whole-store enumeration. `NotFound` maps to `Ok(None)` (the key is gone — the
    /// reconciler then skips it); any other backend error propagates and is counted fail-closed.
    async fn stat(&self, key: &str) -> Result<Option<BlobEntry>, BlobError> {
        match self.store.head(&self.location(key)).await {
            Ok(meta) => Ok(Some(Self::entry(key.to_string(), &meta))),
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(error) => Err(Self::backend_error("head", error)),
        }
    }

    /// UNSUPPORTED on this backend, and deliberately so: `object_store` 0.14 has no conditional or
    /// versioned delete, so no ATOMIC compare-and-delete can be built on it (see the module docs).
    ///
    /// This adapter therefore never hands out a [`BlobEntry::generation`], and the reconciler only
    /// calls this method for a candidate whose snapshot carried one — so the path is unreachable
    /// through [`super::reconcile::reconcile_orphans`], which routes every candidate from this store
    /// down the versionless unconditional-delete branch instead.
    ///
    /// Should any caller invoke it anyway, it FAILS rather than faking the CAS. The two alternatives
    /// are both worse: a `stat()`-then-`delete()` would reintroduce the exact TOCTOU the method
    /// exists to close, and a silent `Ok(false)` would report "the witness moved" for a comparison
    /// that never happened. The `Ok(false)` spelling is the dangerous one for the future: if a later
    /// change started reporting a `generation` here without also implementing the CAS, every sweep
    /// would silently decline to reclaim anything and orphans would accumulate forever with no
    /// signal. Erroring instead surfaces as a `delete_errors` tick in every
    /// [`ReconcileReport`](super::ReconcileReport). Either way NOTHING is removed, so this is
    /// fail-closed in the direction that matters: live bytes are never clobbered.
    async fn delete_if_unchanged(
        &self,
        _key: &str,
        _expected_generation: u64,
    ) -> Result<bool, BlobError> {
        Err(BlobError::Backend(
            "object_store exposes no atomic conditional delete: this backend reports no generation, \
             so delete_if_unchanged is not reachable through the reconciler and refuses rather than \
             performing a non-atomic compare-then-delete"
                .into(),
        ))
    }
}

/// Percent-decode ONE `object_store` path segment back to the blob key it encodes, or `None` if the
/// result is not valid UTF-8.
///
/// This is the exact inverse of the encoding [`object_store::path::PathPart`] applies on the way in:
/// that encoder emits `%XX` for every byte in its reserved set (`/`, `%`, `~`, `#`, `?`, control
/// bytes, …) and passes everything else through, and it encodes a literal `%` as `%25` — so decoding
/// every `%XX` sequence recovers the original bytes exactly, even for a key that itself contained a
/// `%` or a `/`. A `%` not followed by two hex digits cannot come from that encoder, so it is treated
/// as "not one of ours" (`None`) rather than guessed at.
fn percent_decode(segment: &str) -> Option<String> {
    if !segment.contains('%') {
        return Some(segment.to_string());
    }
    let raw = segment.as_bytes();
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'%' {
            let high = char::from(*raw.get(i + 1)?).to_digit(16)?;
            let low = char::from(*raw.get(i + 2)?).to_digit(16)?;
            out.push(u8::try_from(high * 16 + low).ok()?);
            i += 3;
        } else {
            out.push(raw[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Convert a backend's epoch-relative timestamp to a [`SystemTime`], or `None` when it is not
/// representable.
///
/// Kept as a plain `(seconds, nanoseconds)` function so the conversion is unit-testable without
/// naming `chrono` — which reaches this crate only transitively, through `object_store`, and which
/// the storage layer deliberately does not take as a direct dependency (see [`super::timestamp`]).
///
/// TOTAL by construction: every arithmetic step is checked, so an absurd backend timestamp yields
/// `None` (⇒ `last_modified: None` ⇒ the reconciler never GCs that blob) instead of panicking inside
/// a sweep.
fn system_time_from_epoch(seconds: i64, subsec_nanos: u32) -> Option<SystemTime> {
    // `chrono` documents the sub-second count as reaching into the second *past* 1e9 during a leap
    // second; clamping keeps `Duration::new` from carrying a spurious extra second into the result.
    let nanos = subsec_nanos.min(999_999_999);
    if seconds >= 0 {
        UNIX_EPOCH.checked_add(Duration::new(u64::try_from(seconds).ok()?, nanos))
    } else {
        UNIX_EPOCH
            .checked_sub(Duration::new(seconds.unsigned_abs(), 0))
            .and_then(|t| t.checked_add(Duration::from_nanos(u64::from(nanos))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::store::{
        reconcile_orphans, InMemorySparqClient, ReconcileOptions, ResourceMeta, SparqClient,
    };
    use object_store::memory::InMemory;

    /// A blob key containing characters `object_store`'s `PathPart` encoder rewrites: `/` (which must
    /// NOT become a directory), `%` (which must not be double-decoded), `~` and `#` (both in the
    /// reserved set), plus a non-ASCII byte sequence.
    const AWKWARD_KEY: &str = "https_//pod.example/~alice/a#b%2Fc-café-0123456789abcdef";

    fn store() -> ObjectStoreBlobStore {
        ObjectStoreBlobStore::with_prefix(Arc::new(InMemory::new()), "pods/alice/blobs")
    }

    /// A store over a REAL `LocalFileSystem` backend in a fresh temp directory. Needed because the
    /// in-memory backend returns `Ok` when deleting an absent key, so it cannot witness the
    /// `NotFound => Ok(())` idempotency mapping — the local filesystem (like GCS and Azure) errors
    /// instead, which is exactly the case that mapping exists for.
    fn local_store() -> (ObjectStoreBlobStore, std::path::PathBuf) {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "sparq-lws-object-blob-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).expect("the temp directory must be creatable");
        let backend = object_store::local::LocalFileSystem::new_with_prefix(&dir)
            .expect("the local object store must open on a fresh directory");
        (ObjectStoreBlobStore::new(Arc::new(backend)), dir)
    }

    #[tokio::test]
    async fn round_trips_bytes_and_reports_presence() {
        let blobs = store();
        assert!(!blobs.exists("k").await.unwrap());
        assert!(matches!(blobs.get("k").await, Err(BlobError::NotFound)));

        blobs.put("k", Bytes::from_static(b"body")).await.unwrap();
        assert!(blobs.exists("k").await.unwrap());
        assert_eq!(blobs.get("k").await.unwrap(), Bytes::from_static(b"body"));

        // Overwrite replaces the bytes rather than erroring or appending.
        blobs.put("k", Bytes::from_static(b"v2")).await.unwrap();
        assert_eq!(blobs.get("k").await.unwrap(), Bytes::from_static(b"v2"));
    }

    #[tokio::test]
    async fn delete_is_idempotent_on_a_backend_that_errors_on_a_missing_key() {
        // The trait contract: deleting an absent key is `Ok(())` — the authoritative existence
        // decision lives in the index, not here. Run against the LOCAL FILESYSTEM backend, whose
        // delete genuinely fails with `NotFound` on a missing key, so this is non-vacuous.
        // MUTATION-CHECK: drop the `NotFound => Ok(())` arm in `delete` and the second delete below
        // returns `Err`.
        let (blobs, dir) = local_store();
        blobs.put("k", Bytes::from_static(b"body")).await.unwrap();
        assert_eq!(blobs.get("k").await.unwrap(), Bytes::from_static(b"body"));
        blobs.delete("k").await.unwrap();
        blobs.delete("k").await.unwrap();

        // The same `NotFound` mapping on the three read paths, against the same real backend.
        assert!(!blobs.exists("k").await.unwrap());
        assert!(blobs.stat("k").await.unwrap().is_none());
        assert!(matches!(blobs.get("k").await, Err(BlobError::NotFound)));
        assert!(blobs.list().await.unwrap().is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn the_local_filesystem_backend_round_trips_a_listed_key() {
        // The durable backend an operator would actually point at a disk: a key written through the
        // adapter comes back from `list` unchanged, with a real timestamp, so the reconciler's
        // snapshot works against a filesystem-backed pod and not only the in-memory double.
        const KEY: &str = "https_//pod.example/note-0123456789abcdef";
        let (blobs, dir) = local_store();
        blobs.put(KEY, Bytes::from_static(b"x")).await.unwrap();

        let listed = blobs.list().await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].key, KEY);
        assert!(listed[0].last_modified.is_some());
        assert!(listed[0].generation.is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn list_reports_every_written_key_exactly() {
        // The reconciler's start-of-sweep snapshot. Every key written must come back BYTE-IDENTICAL
        // — including one carrying the characters `PathPart` percent-encodes — because the
        // reconciler diffs these strings against SPARQ's referenced-key set, and a key that did not
        // round-trip would look unreferenced and be GC'd.
        let blobs = store();
        for key in ["plain", AWKWARD_KEY, "with spaces and *stars*"] {
            blobs.put(key, Bytes::from_static(b"x")).await.unwrap();
        }

        let mut listed: Vec<String> =
            blobs.list().await.unwrap().into_iter().map(|e| e.key).collect();
        listed.sort();
        let mut expected = vec![
            "plain".to_string(),
            AWKWARD_KEY.to_string(),
            "with spaces and *stars*".to_string(),
        ];
        expected.sort();
        assert_eq!(listed, expected, "every key must survive the Path round-trip");

        // ...and each listed key is independently readable through `get`, so the round-trip is not
        // merely self-consistent inside `list`.
        for key in &expected {
            assert_eq!(blobs.get(key).await.unwrap(), Bytes::from_static(b"x"));
        }
    }

    #[tokio::test]
    async fn a_key_containing_a_slash_stays_one_object_and_is_never_nested() {
        // `location()` joins the key as ONE path segment, so a `/` inside a key is percent-encoded
        // rather than creating a directory. MUTATION-CHECK: building the location with
        // `ObjectPath::from(format!("{prefix}/{key}"))` instead splits on the `/`, the object lands
        // one level deeper, and `key_of` then rejects it as nested — so `list` returns empty here.
        let blobs = store();
        blobs.put("a/b", Bytes::from_static(b"x")).await.unwrap();
        assert_eq!(blobs.location("a/b").as_ref(), "pods/alice/blobs/a%2Fb");
        let listed = blobs.list().await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].key, "a/b");
    }

    #[tokio::test]
    async fn objects_outside_the_prefix_or_nested_below_it_are_not_reported() {
        // Fail-safe: `list` is the reconciler's "which bytes exist" view, and anything it reports
        // that SPARQ does not reference becomes a GC candidate. An object this adapter could not
        // have written must therefore be invisible to it — never offered to the GC.
        let backend: Arc<dyn ObjectStore> = Arc::new(InMemory::new());
        let blobs = ObjectStoreBlobStore::with_prefix(Arc::clone(&backend), "pods/alice/blobs");
        blobs.put("mine", Bytes::from_static(b"x")).await.unwrap();
        // A sibling prefix, and a nested object *under* our prefix, both written behind our back.
        backend
            .put(
                &ObjectPath::from("pods/alice/blobs-other/theirs"),
                Bytes::from_static(b"y").into(),
            )
            .await
            .unwrap();
        backend
            .put(
                &ObjectPath::from("pods/alice/blobs/nested/deeper"),
                Bytes::from_static(b"z").into(),
            )
            .await
            .unwrap();

        let listed: Vec<String> =
            blobs.list().await.unwrap().into_iter().map(|e| e.key).collect();
        assert_eq!(
            listed,
            vec!["mine".to_string()],
            "only objects this adapter wrote may be reported to the reconciler"
        );
    }

    #[tokio::test]
    async fn stat_reports_a_real_timestamp_and_no_generation() {
        // `last_modified` is the AGE witness the grace window is evaluated against, so it must be a
        // REAL backend timestamp, not a placeholder. `generation` is `None` because `object_store`
        // has no conditional delete to CAS on (see the module docs).
        let blobs = store();
        let before = SystemTime::now() - Duration::from_secs(5);
        blobs.put("k", Bytes::from_static(b"x")).await.unwrap();
        let after = SystemTime::now() + Duration::from_secs(5);

        let entry = blobs.stat("k").await.unwrap().expect("the key exists");
        assert_eq!(entry.key, "k");
        let stamp = entry.last_modified.expect("the backend records a timestamp");
        assert!(
            stamp >= before && stamp <= after,
            "stat must surface the backend's own write time, not a placeholder"
        );
        assert!(
            entry.generation.is_none(),
            "no native write version is exposed, so no CAS witness may be advertised"
        );

        // The same two properties on the list path, and `None` once the key is gone.
        let listed = blobs.list().await.unwrap();
        assert_eq!(listed.len(), 1);
        assert!(listed[0].last_modified.is_some() && listed[0].generation.is_none());
        blobs.delete("k").await.unwrap();
        assert!(blobs.stat("k").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn delete_if_unchanged_refuses_and_removes_nothing() {
        // The CAS cannot be built atomically on `object_store` 0.14, so it must FAIL rather than
        // degrade into a `stat()`-then-`delete()`. MUTATION-CHECK: implementing it as that
        // non-atomic pair would make this call return `Ok(true)` AND remove the bytes — both
        // assertions below go red.
        let blobs = store();
        blobs.put("k", Bytes::from_static(b"live")).await.unwrap();
        assert!(matches!(
            blobs.delete_if_unchanged("k", 1).await,
            Err(BlobError::Backend(_))
        ));
        assert_eq!(
            blobs.get("k").await.unwrap(),
            Bytes::from_static(b"live"),
            "a refused CAS must never remove bytes"
        );
    }

    #[tokio::test]
    async fn reconciler_reclaims_an_orphan_and_keeps_a_referenced_blob() {
        // The end-to-end payoff: the orphaned-bytes sweep now runs against a REAL backend. With
        // `generation: None` every candidate takes the versionless unconditional-delete path, which
        // is sound under unique-per-write keys. A zero grace window makes "old enough" immediate so
        // the test needs no sleep and no back-dating.
        let sparq = InMemorySparqClient::new();
        let blobs = store();

        // A live resource: written through the index, so its minted key IS referenced.
        sparq
            .put_meta(
                "https://pod.example/live",
                ResourceMeta {
                    content_type: "text/turtle".to_string(),
                    blob_key: "live-key".to_string(),
                    etag: "\"e\"".to_string(),
                    last_modified: Some(SystemTime::now()),
                },
            )
            .await
            .expect("index write");
        blobs
            .put("live-key", Bytes::from_static(b"live"))
            .await
            .unwrap();
        // ...and bytes with no index row at all — the orphan.
        blobs
            .put("orphan-key", Bytes::from_static(b"orphan"))
            .await
            .unwrap();

        let referenced = sparq.referenced_blob_keys().await.expect("referenced set");
        assert!(
            referenced.contains("live-key"),
            "the index must reference the live blob for this test to be meaningful"
        );

        let report = reconcile_orphans(
            &sparq,
            &blobs,
            &ReconcileOptions {
                grace: Duration::ZERO,
                dry_run: false,
            },
        )
        .await
        .expect("the sweep must run against the object_store backend");

        assert_eq!(report.scanned, 2);
        assert_eq!(report.referenced, 1);
        assert_eq!(report.orphaned, 1);
        assert_eq!(
            report.deleted, 1,
            "a versionless orphan must be RECLAIMED, not leaked (reconcile.rs Finding 2)"
        );
        assert_eq!(report.delete_errors, 0, "the CAS path must not be reached");
        assert!(!blobs.exists("orphan-key").await.unwrap());
        assert!(
            blobs.exists("live-key").await.unwrap(),
            "a referenced blob must never be GC'd"
        );
    }

    #[test]
    fn percent_decode_is_the_exact_inverse_of_the_path_encoder() {
        // Round-trip the key through the SAME encoder `location()` uses, then decode. MUTATION-CHECK:
        // a decoder that skipped the `%25` case (treating a literal `%` as an escape introducer)
        // fails on the third and fourth keys.
        for key in [
            "plain-key",
            AWKWARD_KEY,
            "%",
            "%2F",
            "%zz",
            ".",
            "..",
            "a/b/c",
            "~tilde~",
            "curly{}^`[]\"<>|*?#\\",
        ] {
            let encoded = ObjectPath::default().join(key);
            assert_eq!(
                percent_decode(encoded.as_ref()).as_deref(),
                Some(key),
                "encoding then decoding {key:?} must be the identity"
            );
        }
    }

    #[test]
    fn percent_decode_rejects_a_segment_the_encoder_could_not_have_produced() {
        // A truncated or non-hex escape is not something `PathPart` emits, so it is "not one of
        // ours" rather than something to guess at — `list` skips such an object instead of handing
        // the GC a key it cannot verify.
        assert_eq!(percent_decode("%"), None);
        assert_eq!(percent_decode("%2"), None);
        assert_eq!(percent_decode("%zz"), None);
        // ...and an escape sequence that decodes to invalid UTF-8 is rejected too.
        assert_eq!(percent_decode("%FF"), None);
    }

    #[test]
    fn epoch_conversion_is_total_and_handles_the_edges() {
        assert_eq!(system_time_from_epoch(0, 0), Some(UNIX_EPOCH));
        assert_eq!(
            system_time_from_epoch(1, 500_000_000),
            Some(UNIX_EPOCH + Duration::new(1, 500_000_000))
        );
        // Pre-epoch instants land before the epoch rather than saturating or panicking.
        assert_eq!(
            system_time_from_epoch(-1, 0),
            Some(UNIX_EPOCH - Duration::from_secs(1))
        );
        // A leap-second sub-second count is clamped, so it can never carry an extra whole second.
        assert_eq!(
            system_time_from_epoch(10, 1_500_000_000),
            Some(UNIX_EPOCH + Duration::new(10, 999_999_999))
        );
    }
}
