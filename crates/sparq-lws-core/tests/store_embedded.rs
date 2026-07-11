// AUTHORED-BY Claude Opus 4.8
//! Store-trait integration tests against the **embedded** SPARQ backend — re-proving the data-path
//! behaviors on the in-process engine (`EmbeddedSparqClient` over a `CompositeStore`), NOT assuming
//! them from the in-memory double.
//!
//! These mirror the core data-path cases in `tests/store.rs` (meta read, exists, put/rewrite,
//! create-child membership + missing-container guard, delete, delete-if-empty's three outcomes,
//! containment, parent-detach, referenced-blob-keys) but run them against the REAL sparq engine via
//! the embedded client. A green run proves the engine executes the named-graph builders in
//! [`sparq_lws_core::store::sparql`] with the same semantics the HTTP/in-mem impls give — the whole
//! point of the embed (same queries, different transport).
//!
//! The whole file is gated on the opt-in `embedded-sparq` feature (the default build carries no
//! sparq dependency); it is a no-op test binary when the feature is off.
#![cfg(feature = "embedded-sparq")]

use axum::body::Bytes;
use sparq_lws_core::error::ServerError;
use sparq_lws_core::store::embedded::EmbeddedSparqClient;
use sparq_lws_core::store::{
    CompositeStore, DeleteOutcome, InMemoryBlobStore, ResourceMeta, SparqClient, Store,
};

const IRI: &str = "https://pod.example/alice/data";
const TURTLE: &str =
    "<https://pod.example/alice/data#me> <http://xmlns.com/foaf/0.1/name> \"Alice\" .";

/// A composite store over the EMBEDDED SPARQ engine (fresh in-memory graph) + an in-memory blob
/// store — the exact production wiring `PSS_SPARQ_BACKEND=embedded` selects, minus the durable blob
/// backend (out of scope for the data-path proof).
fn store() -> impl Store {
    CompositeStore::new(
        EmbeddedSparqClient::in_memory().expect("empty in-memory embedded graph"),
        InMemoryBlobStore::new(),
    )
}

#[tokio::test]
async fn read_of_a_missing_resource_is_not_found() {
    let s = store();
    assert!(matches!(
        s.read(IRI).await.unwrap_err(),
        ServerError::NotFound
    ));
}

#[tokio::test]
async fn exists_is_false_before_a_write() {
    let s = store();
    assert!(!s.exists(IRI).await.unwrap());
}

#[tokio::test]
async fn write_then_read_round_trips_bytes_and_content_type() {
    let s = store();
    let body = Bytes::from_static(TURTLE.as_bytes());
    let meta = s.write(IRI, body.clone(), "text/turtle").await.unwrap();
    assert_eq!(meta.content_type, "text/turtle");
    assert!(!meta.etag.is_empty());

    assert!(s.exists(IRI).await.unwrap());
    let resource = s.read(IRI).await.unwrap();
    assert_eq!(resource.body, body);
    assert_eq!(resource.meta.content_type, "text/turtle");
    assert_eq!(resource.meta.etag, meta.etag);
}

#[tokio::test]
async fn rewrite_replaces_the_bytes_and_metadata() {
    let s = store();
    s.write(IRI, Bytes::from_static(b"<a> <b> <c> ."), "text/turtle")
        .await
        .unwrap();
    let new_body = Bytes::from_static(b"<a> <b> <d> .");
    s.write(IRI, new_body.clone(), "application/ld+json")
        .await
        .unwrap();
    let resource = s.read(IRI).await.unwrap();
    assert_eq!(resource.body, new_body);
    // The metadata record is REPLACED (single-valued), not accumulated, so a read is deterministic.
    assert_eq!(resource.meta.content_type, "application/ld+json");
}

#[tokio::test]
async fn delete_removes_metadata_and_bytes() {
    let s = store();
    s.write(IRI, Bytes::from_static(TURTLE.as_bytes()), "text/turtle")
        .await
        .unwrap();
    assert!(s.exists(IRI).await.unwrap());

    s.delete(IRI, None).await.unwrap();
    assert!(!s.exists(IRI).await.unwrap());
    assert!(matches!(
        s.read(IRI).await.unwrap_err(),
        ServerError::NotFound
    ));
}

#[tokio::test]
async fn delete_is_idempotent_on_absent() {
    let s = store();
    s.delete(IRI, None).await.unwrap();
}

#[tokio::test]
async fn create_in_container_records_membership() {
    let s = store();
    let container = "https://pod.example/alice/";
    let child = "https://pod.example/alice/note1";
    s.write(
        container,
        Bytes::from_static(b"<#c> <#p> \"C\" ."),
        "text/turtle",
    )
    .await
    .unwrap();
    s.create_in_container(
        container,
        child,
        Bytes::from_static(b"<#it> <#p> \"x\" ."),
        "text/turtle",
    )
    .await
    .unwrap();

    assert_eq!(
        s.list_children(container)
            .await
            .unwrap()
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>(),
        vec![child]
    );
    assert!(s.exists(child).await.unwrap());
}

#[tokio::test]
async fn create_in_a_missing_container_is_not_found() {
    let s = store();
    let err = s
        .create_in_container(
            "https://pod.example/missing/",
            "https://pod.example/missing/child",
            Bytes::from_static(b"<#it> <#p> \"x\" ."),
            "text/turtle",
        )
        .await
        .unwrap_err();
    assert!(matches!(err, ServerError::NotFound));
    assert!(!s.exists("https://pod.example/missing/child").await.unwrap());
}

#[tokio::test]
async fn create_in_container_twice_keeps_a_single_membership() {
    let s = store();
    let container = "https://pod.example/alice/";
    let child = "https://pod.example/alice/note1";
    s.write(
        container,
        Bytes::from_static(b"<#c> <#p> \"C\" ."),
        "text/turtle",
    )
    .await
    .unwrap();
    for _ in 0..2 {
        s.create_in_container(
            container,
            child,
            Bytes::from_static(b"<#it> <#p> \"x\" ."),
            "text/turtle",
        )
        .await
        .unwrap();
    }
    assert_eq!(
        s.list_children(container)
            .await
            .unwrap()
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>(),
        vec![child],
        "a re-create of the same child IRI must not duplicate the membership edge"
    );
}

#[tokio::test]
async fn create_child_commits_metadata_and_membership_atomically_on_the_engine() {
    // Directly on the EMBEDDED SparqClient: a missing container is refused with nothing written; with
    // the container indexed, the child's metadata AND its edge land together.
    let sparq = EmbeddedSparqClient::in_memory().unwrap();
    let container = "https://pod.example/alice/";
    let child = "https://pod.example/alice/note1";
    let meta = ResourceMeta {
        content_type: "text/turtle".into(),
        blob_key: "k".into(),
        etag: "\"e\"".into(),
        last_modified: None,
    };

    let err = sparq
        .create_child(container, child, meta.clone())
        .await
        .unwrap_err();
    assert!(matches!(err, sparq_lws_core::store::SparqError::NotFound));
    assert!(!sparq.exists(child).await.unwrap());
    assert!(sparq.list_children(container).await.unwrap().is_empty());

    sparq.put_meta(container, meta.clone()).await.unwrap();
    sparq.create_child(container, child, meta).await.unwrap();
    assert!(sparq.exists(child).await.unwrap());
    assert_eq!(
        sparq.list_children(container).await.unwrap(),
        vec![child.to_string()]
    );
}

#[tokio::test]
async fn modified_time_round_trips_through_the_real_engine() {
    // The `pss:modified` `xsd:dateTime` written by `put_meta` must survive storage + retrieval
    // through the ACTUAL sparq-engine (not just the query-string builders): a whole-second instant
    // in ⇒ the same instant back out of `get_meta`. This is the end-to-end proof the OPTIONAL SELECT
    // + the typed-literal storage work on a real engine — the basis of a correct `If-Modified-Since`.
    use std::time::{Duration, UNIX_EPOCH};

    let sparq = EmbeddedSparqClient::in_memory().unwrap();
    let iri = "https://pod.example/alice/data";
    // Whole seconds (the write truncates sub-second precision).
    let t = UNIX_EPOCH + Duration::from_secs(1_783_254_896);
    let meta = ResourceMeta {
        content_type: "text/turtle".into(),
        blob_key: "k".into(),
        etag: "\"e\"".into(),
        last_modified: Some(t),
    };
    sparq.put_meta(iri, meta).await.unwrap();
    let read_back = sparq.get_meta(iri).await.unwrap();
    assert_eq!(
        read_back.last_modified,
        Some(t),
        "the modification time must round-trip exactly through the engine"
    );

    // A re-write with a NEW time REPLACES it (single-valued), never accumulates.
    let t2 = UNIX_EPOCH + Duration::from_secs(1_783_254_900);
    let meta2 = ResourceMeta {
        content_type: "text/turtle".into(),
        blob_key: "k2".into(),
        etag: "\"e2\"".into(),
        last_modified: Some(t2),
    };
    sparq.put_meta(iri, meta2).await.unwrap();
    let after = sparq.get_meta(iri).await.unwrap();
    assert_eq!(
        after.last_modified,
        Some(t2),
        "a re-write bumps the modification time (single-valued replace)"
    );

    // A record written with NO modification time reads back `None` (the untracked case → 200).
    let none_iri = "https://pod.example/alice/untimed";
    let meta_none = ResourceMeta {
        content_type: "text/turtle".into(),
        blob_key: "k3".into(),
        etag: "\"e3\"".into(),
        last_modified: None,
    };
    sparq.put_meta(none_iri, meta_none).await.unwrap();
    assert_eq!(
        sparq.get_meta(none_iri).await.unwrap().last_modified,
        None,
        "no recorded modification time reads back as None"
    );
}

#[tokio::test]
async fn delete_detaches_from_parent_container() {
    let s = store();
    let container = "https://pod.example/alice/";
    let child = "https://pod.example/alice/note1";
    s.write(
        container,
        Bytes::from_static(b"<#c> <#p> \"C\" ."),
        "text/turtle",
    )
    .await
    .unwrap();
    s.create_in_container(
        container,
        child,
        Bytes::from_static(b"<#it> <#p> \"x\" ."),
        "text/turtle",
    )
    .await
    .unwrap();
    assert_eq!(
        s.list_children(container)
            .await
            .unwrap()
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>(),
        vec![child]
    );

    // Deleting the child WITH its parent detaches the membership edge atomically.
    s.delete(child, Some(container)).await.unwrap();
    assert!(!s.exists(child).await.unwrap());
    assert!(
        s.list_children(container).await.unwrap().is_empty(),
        "the child's membership edge must be detached from the parent on delete"
    );
}

#[tokio::test]
async fn delete_container_if_empty_refuses_a_populated_container() {
    // Safety invariant: a non-empty container is NEVER deleted — NotEmpty, nothing touched.
    let s = store();
    let container = "https://pod.example/alice/";
    let child = "https://pod.example/alice/note1";
    s.write(
        container,
        Bytes::from_static(b"<#c> <#p> \"C\" ."),
        "text/turtle",
    )
    .await
    .unwrap();
    s.create_in_container(
        container,
        child,
        Bytes::from_static(b"<#it> <#p> \"x\" ."),
        "text/turtle",
    )
    .await
    .unwrap();

    let outcome = s.delete_container_if_empty(container, None).await.unwrap();
    assert_eq!(outcome, DeleteOutcome::NotEmpty);
    assert!(
        s.exists(container).await.unwrap(),
        "NotEmpty must leave the container present"
    );
    assert!(
        s.exists(child).await.unwrap(),
        "NotEmpty must leave the child present (not orphaned)"
    );
    assert_eq!(
        s.list_children(container)
            .await
            .unwrap()
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>(),
        vec![child],
        "NotEmpty must leave the membership edge intact"
    );
}

#[tokio::test]
async fn delete_container_if_empty_deletes_an_empty_container() {
    let s = store();
    let container = "https://pod.example/alice/empty/";
    s.write(
        container,
        Bytes::from_static(b"<#c> <#p> \"C\" ."),
        "text/turtle",
    )
    .await
    .unwrap();
    assert!(s.exists(container).await.unwrap());

    let outcome = s.delete_container_if_empty(container, None).await.unwrap();
    assert_eq!(outcome, DeleteOutcome::Deleted);
    assert!(!s.exists(container).await.unwrap());
    assert!(matches!(
        s.read(container).await.unwrap_err(),
        ServerError::NotFound
    ));
}

#[tokio::test]
async fn delete_container_if_empty_reports_not_found_for_an_absent_container() {
    let s = store();
    let outcome = s
        .delete_container_if_empty("https://pod.example/alice/nope/", None)
        .await
        .unwrap();
    assert_eq!(outcome, DeleteOutcome::NotFound);
}

#[tokio::test]
async fn delete_container_if_empty_detaches_from_parent_and_routes_recreate_clean() {
    // The atomic empty-delete detaches the (deleted) container from its parent's containment, and a
    // container re-created at the same IRI inherits NO stale membership.
    let s = store();
    let parent = "https://pod.example/alice/";
    let container = "https://pod.example/alice/sub/";
    let child = "https://pod.example/alice/sub/note1";
    s.write(
        parent,
        Bytes::from_static(b"<#c> <#p> \"P\" ."),
        "text/turtle",
    )
    .await
    .unwrap();
    // The sub-container is a child of the parent.
    s.create_in_container(
        parent,
        container,
        Bytes::from_static(b"<#c> <#p> \"S\" ."),
        "text/turtle",
    )
    .await
    .unwrap();
    // Give the sub-container a child of its own, then remove it so the sub-container is empty.
    s.create_in_container(
        container,
        child,
        Bytes::from_static(b"<#it> <#p> \"x\" ."),
        "text/turtle",
    )
    .await
    .unwrap();
    s.delete(child, Some(container)).await.unwrap();

    // Now the empty sub-container deletes AND detaches from the parent.
    let outcome = s
        .delete_container_if_empty(container, Some(parent))
        .await
        .unwrap();
    assert_eq!(outcome, DeleteOutcome::Deleted);
    assert!(!s.exists(container).await.unwrap());
    assert!(
        s.list_children(parent).await.unwrap().is_empty(),
        "the deleted sub-container must be detached from the parent's containment"
    );

    // Re-create a container at the SAME IRI under the parent: it must inherit no stale membership.
    s.create_in_container(
        parent,
        container,
        Bytes::from_static(b"<#c> <#p> \"S2\" ."),
        "text/turtle",
    )
    .await
    .unwrap();
    assert!(
        s.list_children(container).await.unwrap().is_empty(),
        "a container re-created at the same IRI must inherit no stale containment"
    );
    assert_eq!(
        s.list_children(parent)
            .await
            .unwrap()
            .iter()
            .map(|c| c.as_str())
            .collect::<Vec<_>>(),
        vec![container],
        "the parent re-contains the recreated sub-container exactly once"
    );
}

#[tokio::test]
async fn referenced_blob_keys_reflects_live_pointers() {
    // The reconciler's referenced-set: every live metadata record's blob key is referenced; a deleted
    // resource's key drops out.
    let s = store();
    let m1 = s
        .write(
            "https://pod.example/a",
            Bytes::from_static(b"<a> <b> <c> ."),
            "text/turtle",
        )
        .await
        .unwrap();
    let m2 = s
        .write(
            "https://pod.example/b",
            Bytes::from_static(b"<a> <b> <d> ."),
            "text/turtle",
        )
        .await
        .unwrap();
    // Reach the SparqClient through a second composite over the same engine is not possible (the engine
    // is moved into the store), so assert via list/containment-independent behaviour: deleting `a`
    // must drop its key from the live set. We re-prove referenced_blob_keys directly on a standalone
    // embedded client below; here we assert the store-level delete leaves `b` readable.
    s.delete("https://pod.example/a", None).await.unwrap();
    assert!(!s.exists("https://pod.example/a").await.unwrap());
    assert!(s.exists("https://pod.example/b").await.unwrap());
    assert_ne!(
        m1.blob_key, m2.blob_key,
        "distinct resources get distinct blob keys"
    );
}

#[tokio::test]
async fn referenced_blob_keys_on_the_embedded_client_collects_all_pointers() {
    // Directly on the embedded SparqClient: the referenced set is exactly the live records' keys.
    let sparq = EmbeddedSparqClient::in_memory().unwrap();
    let m = |bk: &str| ResourceMeta {
        content_type: "text/turtle".into(),
        blob_key: bk.into(),
        etag: "\"e\"".into(),
        last_modified: None,
    };
    sparq
        .put_meta("https://pod.example/a", m("k1"))
        .await
        .unwrap();
    sparq
        .put_meta("https://pod.example/b", m("k2"))
        .await
        .unwrap();
    let keys = sparq.referenced_blob_keys().await.unwrap();
    assert!(keys.contains("k1") && keys.contains("k2"), "got {keys:?}");
    assert_eq!(keys.len(), 2);

    // After deleting `a`, its key is no longer referenced.
    sparq.delete_meta("https://pod.example/a").await.unwrap();
    let keys = sparq.referenced_blob_keys().await.unwrap();
    assert!(
        !keys.contains("k1") && keys.contains("k2"),
        "k1 should drop out: {keys:?}"
    );
}
