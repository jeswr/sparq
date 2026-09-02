// AUTHORED-BY Claude Sonnet 4.6
//! The real [`object_store`]-backed [`BlobStore`] adapter — the S3 / GCS / Azure / local-filesystem
//! byte backend behind the same seam the in-memory double implements.
//!
//! Until this module landed, NO [`BlobStore`] implementation was backed by durable storage: the only
//! ones were the in-memory double in [`super::blob`] and the decorators that wrap it (e.g.
//! [`super::counting::CountingBlobStore`]). So
//! [`BlobStore::list`] — and therefore the orphaned-bytes reconciler
//! ([`super::reconcile::reconcile_orphans`]) — could not run against a real backend at all. This
//! adapter closes that: it maps the seam onto [`object_store::ObjectStore`], which is one trait over
//! S3 / GCS / Azure / `LocalFileSystem` / `InMemory`.
//!
//! # Not feature-gated, and why
//! `object_store` is ALREADY a mandatory native dependency of this crate (it is declared
//! unconditionally in `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` and, before this
//! module, was used by no source file). So compiling this adapter adds NO crate to the dependency
//! graph and no weight to the default build — the "keep new capabilities behind an opt-in feature"
//! rule exists to stop a heavy dependency being forced onto the default build, and there is no such
//! dependency to force here. Compiling it unconditionally also means its tests run in the ordinary
//! `cargo test` lane rather than only inside a feature-matrix leg. It IS gated on
//! `not(target_arch = "wasm32")`, because `object_store` is a native-only dependency — the wasm
//! request core keeps it out of its graph, which `tests/wasm_dependency_boundary.rs` asserts.
//!
//! # Key ↔ object-path mapping
//! A [`BlobStore`] key is an opaque string (minted by `CompositeStore::mint_blob_key`); an
//! `object_store` location is a [`object_store::path::Path`]. The mapping here is deliberately the
//! ENCODING-FREE one: the location is `"<prefix>/<key>"` handed to [`ObjectPath::parse`], which
//! validates rather than re-encodes, so the key is recovered byte-exactly by stripping the prefix back
//! off [`ObjectPath::as_ref`]. (The other candidate — `Path::from`/`Path::join`, which percent-ENCODE
//! each segment — would need a matching decode to invert, and `object_store` exposes no decode:
//! `Path::parts` hands back the still-encoded segment.) `parse` does silently strip a leading or
//! trailing `/`, so it is not literally a no-op on every input; every key is therefore checked to
//! round-trip before it is used, and one that would not survive the mapping FAILS CLOSED with
//! [`BlobError::Backend`] instead of silently addressing a different object.
//!
//! # Why this adapter reports NO generation
//! [`BlobEntry::generation`] is the compare-and-delete witness for
//! [`BlobStore::delete_if_unchanged`], whose contract is explicit and load-bearing: the compare and
//! the removal must happen in ONE uninterrupted critical section. `object_store` 0.14 exposes no
//! conditional or versioned DELETE on any backend — its optimistic-concurrency surface is
//! conditional PUT only (`PutMode::Create` / `PutMode::Update(UpdateVersion)`); the delete surface is
//! [`object_store::ObjectStore::delete_stream`] (and the [`object_store::ObjectStoreExt::delete`]
//! convenience over it), which takes no precondition. A `head`-then-`delete` pair would NOT be
//! atomic and would reintroduce exactly the TOCTOU the CAS exists to close — the "silent footgun"
//! [`BlobStore::delete_if_unchanged`] documents and forbids.
//!
//! So this adapter reports `generation: None` for every entry, which is the truthful answer for a
//! backend with no usable native write version, and it is the answer the reconciler already knows how
//! to handle: `reconcile_orphans` routes a version-less candidate down its `DeleteUnconditional`
//! path, which is safe because `CompositeStore::mint_blob_key` mints UNIQUE-PER-WRITE keys — a
//! recreate of the same IRI gets a DIFFERENT key, so an orphan's key can never be a live object's.
//! That candidate is still age-gated by the grace window, re-checked against a fresh referenced set,
//! and re-statted before the delete. Reclamation therefore works end-to-end against a real backend
//! (`reconciler_reclaims_an_aged_orphan_through_this_adapter` below drives it), it simply does not go
//! through the CAS.
//!
//! Correspondingly [`ObjectStoreBlobStore::delete_if_unchanged`] never deletes and always reports
//! `Ok(false)`. That is not a fudge: `Ok(false)` means "the witness no longer matches the key's
//! current generation, so NOTHING was removed", and since this adapter never issues a generation, no
//! `u64` a caller can pass was ever a witness this store handed out — the current generation is
//! `None`, which matches no `u64`. Refusing is also the only fail-closed answer available: it can
//! never clobber live bytes.
//!
//! Making the CAS real needs a backend-native conditional delete keyed on the backend's own write
//! version (S3 object versioning / an `If-Match` on DELETE), which `object_store` does not surface
//! today; that is upstream work, not something this adapter can synthesise.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::TryStreamExt;
use object_store::path::Path as ObjectPath;
use object_store::{Error as ObjectStoreError, ObjectStore, ObjectStoreExt};

use super::blob::{BlobEntry, BlobError, BlobStore};

/// A [`BlobStore`] backed by any [`object_store::ObjectStore`] — S3, GCS, Azure, `LocalFileSystem`,
/// or `InMemory`.
///
/// All keys live under a single [`ObjectPath`] prefix, so one bucket can host several stores (or a
/// store can share a bucket with unrelated data): [`list`](BlobStore::list) only ever reports objects
/// under that prefix, and every other operation only ever addresses one.
///
/// See the module documentation for the key ↔ object-path mapping and for why this adapter reports
/// no [`BlobEntry::generation`].
#[derive(Debug, Clone)]
pub struct ObjectStoreBlobStore {
    inner: Arc<dyn ObjectStore>,
    /// The prefix every key hangs under. [`ObjectPath::default`] (the store root) when unset.
    prefix: ObjectPath,
}

impl ObjectStoreBlobStore {
    /// Build an adapter over the whole store — keys map to top-level objects.
    pub fn new(inner: Arc<dyn ObjectStore>) -> Self {
        Self {
            inner,
            prefix: ObjectPath::default(),
        }
    }

    /// Build an adapter scoped to `prefix` — every key maps to `"<prefix>/<key>"`, and
    /// [`list`](BlobStore::list) reports only objects under it.
    ///
    /// Returns [`BlobError::Backend`] if `prefix` is not a valid [`ObjectPath`] (an empty path
    /// segment, a `.`/`..` segment, or an ASCII control character). An empty prefix is accepted and
    /// means the store root, exactly as [`new`](Self::new).
    pub fn with_prefix(inner: Arc<dyn ObjectStore>, prefix: &str) -> Result<Self, BlobError> {
        let prefix = ObjectPath::parse(prefix)
            .map_err(|e| BlobError::Backend(format!("invalid blob-store prefix: {}", e)))?;
        Ok(Self { inner, prefix })
    }

    /// The configured prefix, as the raw object-path string (empty for the store root).
    pub fn prefix(&self) -> &str {
        self.prefix.as_ref()
    }

    /// Map an opaque blob key to the object location it addresses, FAILING CLOSED unless the mapping
    /// round-trips exactly.
    ///
    /// The round-trip check is the load-bearing part. [`ObjectPath::parse`] silently strips a leading
    /// or trailing `/`, so without it a key like `"k/"` would address the object `"k"` — a DIFFERENT
    /// object than the caller named, which for a delete would mean removing the wrong bytes. Any key
    /// whose recovered form is not byte-identical is rejected rather than mapped.
    fn location(&self, key: &str) -> Result<ObjectPath, BlobError> {
        if key.is_empty() {
            return Err(BlobError::Backend("blob key must not be empty".into()));
        }
        let raw = if self.prefix.as_ref().is_empty() {
            key.to_string()
        } else {
            format!("{}/{}", self.prefix.as_ref(), key)
        };
        let location = ObjectPath::parse(&raw).map_err(|e| {
            BlobError::Backend(format!("blob key is not a valid object path: {}", e))
        })?;
        if self.key_of(&location).as_deref() != Some(key) {
            return Err(BlobError::Backend(
                "blob key does not round-trip through an object path".into(),
            ));
        }
        Ok(location)
    }

    /// The inverse of [`location`](Self::location): recover the blob key a location addresses, or
    /// `None` when the location is not a key of THIS store (not under the prefix, or the prefix
    /// itself).
    ///
    /// `None` makes [`list`](BlobStore::list) skip the object entirely, which is the fail-closed
    /// direction: a key the reconciler never sees is a key it can never delete.
    fn key_of(&self, location: &ObjectPath) -> Option<String> {
        let raw = location.as_ref();
        let rest = if self.prefix.as_ref().is_empty() {
            raw
        } else {
            raw.strip_prefix(self.prefix.as_ref())?.strip_prefix('/')?
        };
        (!rest.is_empty()).then(|| rest.to_string())
    }

    /// Read one object's metadata, mapping a backend `NotFound` to `Ok(None)`.
    async fn head_entry(&self, key: &str) -> Result<Option<BlobEntry>, BlobError> {
        let location = self.location(key)?;
        match self.inner.head(&location).await {
            Ok(meta) => Ok(Some(BlobEntry {
                key: key.to_string(),
                last_modified: epoch_to_system_time(
                    meta.last_modified.timestamp(),
                    meta.last_modified.timestamp_subsec_nanos(),
                ),
                generation: None,
            })),
            Err(ObjectStoreError::NotFound { .. }) => Ok(None),
            Err(e) => Err(map_backend_error(e)),
        }
    }
}

/// Map an `object_store` error onto the seam's opaque [`BlobError`].
///
/// The message deliberately carries only the error's KIND, never `object_store`'s rendered error
/// (which embeds the bucket and the full object path). [`BlobError`] is documented as opaque "so it
/// never leaks backend detail to a client"; `BlobError::Backend(msg)` becomes
/// `ServerError::Storage(msg)`, whose HTTP body is already scrubbed to `"internal server error"` by
/// `ServerError`'s `IntoResponse`, so this is defence-in-depth — it also keeps the bucket and object
/// path out of the error's `Display`, and hence out of logs.
fn map_backend_error(err: ObjectStoreError) -> BlobError {
    match err {
        ObjectStoreError::NotFound { .. } => BlobError::NotFound,
        ObjectStoreError::PermissionDenied { .. } | ObjectStoreError::Unauthenticated { .. } => {
            BlobError::Backend("blob backend refused the request".into())
        }
        ObjectStoreError::NotSupported { .. } | ObjectStoreError::NotImplemented { .. } => {
            BlobError::Backend("blob backend does not support the operation".into())
        }
        _ => BlobError::Backend("blob backend request failed".into()),
    }
}

/// Convert a Unix-epoch instant (whole seconds, which may be negative, plus sub-second nanoseconds)
/// into a [`SystemTime`], or `None` if it is not representable on this platform.
///
/// This is the `ObjectMeta.last_modified` (`chrono::DateTime<Utc>`) → [`SystemTime`] step, written
/// against `timestamp()` / `timestamp_subsec_nanos()` so the conversion neither names `chrono` (not a
/// direct dependency of this crate) nor depends on a transitively-enabled `chrono` feature for the
/// `From` impl. `None` propagates to [`BlobEntry::last_modified`], where an unknown age is already
/// defined as fail-closed: the reconciler never GCs a blob it cannot prove is old enough.
fn epoch_to_system_time(secs: i64, subsec_nanos: u32) -> Option<SystemTime> {
    let whole = Duration::from_secs(secs.unsigned_abs());
    let subsec = Duration::from_nanos(u64::from(subsec_nanos));
    let base = if secs >= 0 {
        UNIX_EPOCH.checked_add(whole)?
    } else {
        UNIX_EPOCH.checked_sub(whole)?
    };
    base.checked_add(subsec)
}

#[async_trait]
impl BlobStore for ObjectStoreBlobStore {
    async fn get(&self, key: &str) -> Result<Bytes, BlobError> {
        let location = self.location(key)?;
        let result = self.inner.get(&location).await.map_err(map_backend_error)?;
        result.bytes().await.map_err(map_backend_error)
    }

    async fn put(&self, key: &str, body: Bytes) -> Result<(), BlobError> {
        let location = self.location(key)?;
        self.inner
            .put(&location, body.into())
            .await
            .map_err(map_backend_error)?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, BlobError> {
        Ok(self.head_entry(key).await?.is_some())
    }

    /// Idempotent, per the trait contract: deleting an absent key is `Ok(())`. Backends differ here —
    /// `LocalFileSystem` maps its `ErrorKind::NotFound` from `remove_file` onto
    /// `object_store::Error::NotFound` — so the `NotFound` arm normalises them.
    async fn delete(&self, key: &str) -> Result<(), BlobError> {
        let location = self.location(key)?;
        match self.inner.delete(&location).await {
            Ok(()) | Err(ObjectStoreError::NotFound { .. }) => Ok(()),
            Err(e) => Err(map_backend_error(e)),
        }
    }

    /// Enumerate every object under the configured prefix via
    /// [`object_store::ObjectStore::list`], mapping each `ObjectMeta` to a [`BlobEntry`]:
    /// `location` → the key (prefix stripped), `last_modified` → the age witness.
    ///
    /// `generation` is `None` for every entry — see the module documentation: `object_store` has no
    /// conditional delete, so there is no write version this adapter could honour as a CAS witness,
    /// and the reconciler's version-less (unconditional-delete) path is the correct, safe route.
    ///
    /// The whole listing is drained into a `Vec` because that is the trait's shape, so a very large
    /// bucket is held in memory for the duration of a sweep. That is an accepted cost here: the only
    /// caller is the reconciler, an occasional GC job rather than a request-path operation (the LDP
    /// request path must never enumerate the blob store — see the trait's own docs).
    async fn list(&self) -> Result<Vec<BlobEntry>, BlobError> {
        let mut stream = self.inner.list(Some(&self.prefix));
        let mut entries = Vec::new();
        while let Some(meta) = stream.try_next().await.map_err(map_backend_error)? {
            let Some(key) = self.key_of(&meta.location) else {
                continue;
            };
            entries.push(BlobEntry {
                key,
                last_modified: epoch_to_system_time(
                    meta.last_modified.timestamp(),
                    meta.last_modified.timestamp_subsec_nanos(),
                ),
                generation: None,
            });
        }
        Ok(entries)
    }

    /// O(1) single-key re-stat via `object_store`'s HEAD, rather than the trait default's whole-store
    /// enumeration. A backend `NotFound` is `Ok(None)` (the key is gone); any other error propagates.
    async fn stat(&self, key: &str) -> Result<Option<BlobEntry>, BlobError> {
        self.head_entry(key).await
    }

    /// Always `Ok(false)` — this adapter NEVER deletes through the compare-and-delete path.
    ///
    /// `object_store` 0.14 offers no conditional or versioned delete on any backend, so the trait's
    /// atomicity contract (compare and remove in one uninterrupted critical section) cannot be
    /// honoured; a `head`-then-`delete` pair would silently reintroduce the TOCTOU the CAS exists to
    /// close. Because this adapter also never issues a [`BlobEntry::generation`], the current
    /// generation of every key is `None`, which equals no `u64` — so "the witness does not match" is
    /// literally true for every call, and `Ok(false)` ("witness mismatched, nothing was removed") is
    /// the correct as well as the only fail-closed answer. Reclamation happens instead through the
    /// reconciler's version-less unconditional-delete path; see the module documentation.
    async fn delete_if_unchanged(
        &self,
        _key: &str,
        _expected_generation: u64,
    ) -> Result<bool, BlobError> {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::reconcile::{reconcile_orphans, ReconcileOptions};
    use crate::store::sparq::{InMemorySparqClient, ResourceMeta, SparqClient};
    use object_store::memory::InMemory;

    fn store() -> ObjectStoreBlobStore {
        ObjectStoreBlobStore::new(Arc::new(InMemory::new()))
    }

    fn prefixed(prefix: &str) -> ObjectStoreBlobStore {
        ObjectStoreBlobStore::with_prefix(Arc::new(InMemory::new()), prefix).expect("valid prefix")
    }

    fn sorted_keys(entries: &[BlobEntry]) -> Vec<String> {
        let mut keys: Vec<String> = entries.iter().map(|e| e.key.clone()).collect();
        keys.sort();
        keys
    }

    #[tokio::test]
    async fn round_trips_bytes_through_the_real_backend() {
        let blob = store();
        blob.put("k", Bytes::from_static(b"body")).await.unwrap();
        assert_eq!(blob.get("k").await.unwrap(), Bytes::from_static(b"body"));
        assert!(blob.exists("k").await.unwrap());
        blob.delete("k").await.unwrap();
        assert!(!blob.exists("k").await.unwrap());
        assert!(matches!(blob.get("k").await, Err(BlobError::NotFound)));
    }

    #[tokio::test]
    async fn delete_of_an_absent_key_is_idempotent() {
        // The trait contract: "deleting an absent key is Ok(())". Mutation witness: drop the
        // `NotFound` arm of `delete` and a backend that reports NotFound (LocalFileSystem) errors
        // here instead.
        let blob = store();
        blob.delete("never-written").await.unwrap();
    }

    #[tokio::test]
    async fn list_reports_every_stored_key_with_an_age_but_no_generation() {
        // THE headline guard for `list`. Mutation witness: return an empty Vec, drop the prefix
        // strip, or stop mapping `last_modified`, and one of the three assertions goes red.
        let blob = store();
        for key in ["a", "b", "c"] {
            blob.put(key, Bytes::from_static(b"x")).await.unwrap();
        }

        let entries = blob.list().await.unwrap();
        assert_eq!(sorted_keys(&entries), vec!["a", "b", "c"]);
        assert!(
            entries.iter().all(|e| e.last_modified.is_some()),
            "the age witness must survive the ObjectMeta -> BlobEntry mapping"
        );
        assert!(
            entries.iter().all(|e| e.generation.is_none()),
            "object_store has no conditional delete, so no generation may be advertised"
        );
    }

    #[tokio::test]
    async fn list_is_scoped_to_the_prefix_and_strips_it_from_the_key() {
        // Two adapters over ONE backing store. Each must see only its own objects, and must report
        // the BARE key. Mutation witness: drop the `strip_prefix` in `key_of` and the keys come back
        // as "pod-a/k" (assertion red); drop the prefix argument to `list` and each store sees the
        // other's key (assertion red).
        let backend: Arc<dyn ObjectStore> = Arc::new(InMemory::new());
        let a = ObjectStoreBlobStore::with_prefix(Arc::clone(&backend), "pod-a").unwrap();
        let b = ObjectStoreBlobStore::with_prefix(Arc::clone(&backend), "pod-b").unwrap();
        a.put("k", Bytes::from_static(b"a")).await.unwrap();
        b.put("k", Bytes::from_static(b"b")).await.unwrap();

        assert_eq!(sorted_keys(&a.list().await.unwrap()), vec!["k"]);
        assert_eq!(sorted_keys(&b.list().await.unwrap()), vec!["k"]);
        assert_eq!(a.get("k").await.unwrap(), Bytes::from_static(b"a"));
        assert_eq!(b.get("k").await.unwrap(), Bytes::from_static(b"b"));
        assert_eq!(a.prefix(), "pod-a");
    }

    #[tokio::test]
    async fn stat_is_the_single_key_view_of_list() {
        let blob = prefixed("pod");
        blob.put("k", Bytes::from_static(b"x")).await.unwrap();

        let stat = blob.stat("k").await.unwrap().expect("k exists");
        assert_eq!(stat.key, "k");
        assert!(stat.last_modified.is_some());
        assert!(stat.generation.is_none());
        assert!(blob.stat("absent").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn delete_if_unchanged_never_removes_anything() {
        // THE headline guard for the CAS refusal. Mutation witness: make `delete_if_unchanged`
        // delete (or return `Ok(true)`) and both assertions go red.
        let blob = store();
        blob.put("k", Bytes::from_static(b"live")).await.unwrap();

        for witness in [0u64, 1, u64::MAX] {
            assert!(
                !blob.delete_if_unchanged("k", witness).await.unwrap(),
                "a store that advertises no generation can never match a CAS witness"
            );
        }
        assert_eq!(blob.get("k").await.unwrap(), Bytes::from_static(b"live"));
    }

    #[tokio::test]
    async fn a_key_that_would_not_round_trip_fails_closed() {
        // `ObjectPath::parse` strips a trailing '/', so "k/" would silently address "k". Mutation
        // witness: delete the round-trip comparison in `location` and "k/" writes to "k", so the
        // `list()` assertion below sees a key it was never given.
        let blob = store();
        blob.put("k", Bytes::from_static(b"live")).await.unwrap();

        for bad in ["", "k/", "/k", "a//b", ".."] {
            assert!(
                blob.put(bad, Bytes::from_static(b"x")).await.is_err(),
                "key {:?} must be rejected, not silently remapped",
                bad
            );
        }
        assert_eq!(sorted_keys(&blob.list().await.unwrap()), vec!["k"]);
        assert_eq!(blob.get("k").await.unwrap(), Bytes::from_static(b"live"));
    }

    #[tokio::test]
    async fn keys_containing_a_delimiter_round_trip_exactly() {
        // Blob keys are opaque. `mint_blob_key` never emits '/', but the seam does not forbid it, so
        // a multi-segment key must map to a nested object AND come back byte-identical.
        let blob = prefixed("pod");
        blob.put("nested/key", Bytes::from_static(b"x"))
            .await
            .unwrap();

        assert_eq!(sorted_keys(&blob.list().await.unwrap()), vec!["nested/key"]);
        assert_eq!(
            blob.stat("nested/key").await.unwrap().expect("present").key,
            "nested/key"
        );
    }

    #[tokio::test]
    async fn a_clone_addresses_the_same_backend() {
        // The adapter is `Arc`-backed and `Clone` so it can be handed BY VALUE to the periodic
        // reconciler runner (`spawn_periodic` takes its seams by value) while the request path keeps
        // its own handle. A clone must therefore see the original's objects, not a fresh store.
        let blob = prefixed("pod");
        let clone = blob.clone();
        blob.put("k", Bytes::from_static(b"x")).await.unwrap();

        assert_eq!(sorted_keys(&clone.list().await.unwrap()), vec!["k"]);
        assert_eq!(clone.prefix(), "pod");
        assert!(
            format!("{:?}", clone).contains("ObjectStoreBlobStore"),
            "the Debug rendering names the adapter"
        );
    }

    #[test]
    fn with_prefix_rejects_a_malformed_prefix() {
        let backend: Arc<dyn ObjectStore> = Arc::new(InMemory::new());
        assert!(ObjectStoreBlobStore::with_prefix(Arc::clone(&backend), "a//b").is_err());
        assert!(ObjectStoreBlobStore::with_prefix(Arc::clone(&backend), "..").is_err());
        // An empty prefix is the store root, not an error.
        assert_eq!(
            ObjectStoreBlobStore::with_prefix(backend, "")
                .expect("empty prefix is the root")
                .prefix(),
            ""
        );
    }

    #[test]
    fn epoch_to_system_time_handles_both_sides_of_the_epoch() {
        // Direct unit test of the ObjectMeta timestamp conversion. Mutation witness: drop the
        // sub-second term, or take `secs.unsigned_abs()` without the negative branch, and one of
        // these goes red.
        assert_eq!(epoch_to_system_time(0, 0), Some(UNIX_EPOCH));
        assert_eq!(
            epoch_to_system_time(2, 500_000_000),
            Some(UNIX_EPOCH + Duration::new(2, 500_000_000))
        );
        assert_eq!(
            epoch_to_system_time(-2, 0),
            Some(UNIX_EPOCH - Duration::from_secs(2))
        );
        assert_eq!(
            epoch_to_system_time(-2, 250),
            Some(UNIX_EPOCH - Duration::from_secs(2) + Duration::from_nanos(250))
        );
        // Extreme inputs must never PANIC. Whether they are also representable is platform-dependent
        // (Linux's `timespec` holds both ends), so this asserts the no-panic property rather than a
        // `None`. Mutation witness: swap either `checked_*` for the plain `+`/`-` operator and the
        // saturating arithmetic in `Duration`'s `Add`/`Sub` for `SystemTime` panics here.
        let _ = epoch_to_system_time(i64::MIN, u32::MAX);
        let _ = epoch_to_system_time(i64::MAX, u32::MAX);
    }

    #[tokio::test]
    async fn reconciler_reclaims_an_aged_orphan_through_this_adapter() {
        // The point of the whole issue: with a real BlobStore::list the orphaned-bytes reconciler can
        // finally run against a non-test backend. A referenced blob is kept, an unreferenced one is
        // reclaimed — through the version-less unconditional-delete path, because this adapter
        // advertises no generation. Mutation witness: return an empty Vec from `list` and `scanned`
        // drops to 0; make `list` report a `generation` and the sweep takes the CAS path, whose
        // `delete_if_unchanged` refuses, so `deleted` drops to 0.
        let sparq = InMemorySparqClient::new();
        let blob = store();

        sparq
            .put_meta(
                "https://pod.example/r",
                ResourceMeta {
                    content_type: "text/turtle".into(),
                    blob_key: "live".into(),
                    etag: "\"e\"".into(),
                    last_modified: None,
                },
            )
            .await
            .unwrap();
        blob.put("live", Bytes::from_static(b"referenced"))
            .await
            .unwrap();
        blob.put("orphan", Bytes::from_static(b"unreferenced"))
            .await
            .unwrap();

        // `InMemory` stamps `last_modified` at write time and cannot be back-dated, so the grace
        // window is set to zero to make "older than the grace window" true for a just-written object.
        let opts = ReconcileOptions {
            grace: Duration::ZERO,
            dry_run: false,
        };
        let report = reconcile_orphans(&sparq, &blob, &opts).await.unwrap();

        assert_eq!(report.scanned, 2);
        assert_eq!(report.referenced, 1);
        assert_eq!(report.orphaned, 1);
        assert_eq!(report.deleted, 1);
        assert_eq!(report.delete_errors, 0);
        assert!(
            blob.exists("live").await.unwrap(),
            "a referenced blob must never be reclaimed"
        );
        assert!(
            !blob.exists("orphan").await.unwrap(),
            "an aged, unreferenced blob must be reclaimed through the real backend"
        );
    }
}
