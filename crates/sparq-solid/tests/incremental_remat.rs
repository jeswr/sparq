//! [OPUS-4.8] sq-b7k7u (issue #1571) — incremental re-materialization + scoped
//! session-cache invalidation on `put_acl`/`delete_acl`.
//!
//! The load-bearing invariant, exercised over the REAL write-through + session oracle (no
//! mocks): after ANY sequence of atomic `put_acl`/`delete_acl` writes across multiple pods,
//! the store's SCOPED-cache state is byte-for-byte a from-scratch rebuild —
//!
//! - **equivalence**: `store.accessible(s, m)` (the scoped session cache) and
//!   `store.decide(s, r, m)` (the per-origin decision) equal a fresh
//!   `AuthIndex::from_graph(&store.graph)` after EVERY step, for every session/mode/resource;
//! - **revoke / fail-open-critical**: a `delete_acl` (or an emptying `put_acl`) that removes a
//!   grant is reflected on the next decision — no stale grant survives the scoped invalidation;
//! - **cross-pod isolation**: a write to one pod never changes another pod's decisions.
//!
//! The scoped invalidation is sound because a WAC/ACP grant is confined to its ACL's origin
//! (`rules/wac.n3` `acl:accessTo`/`acl:default`, `rules/acp-a.n3` `appliesToResource`); this
//! suite is the differential guard that the confinement + the reuse of untouched origins'
//! slices never diverges from a full rebuild.

use sparq_solid::{AuthIndex, Mode, PodStore, Session};

const ALICE: &str = "https://alice.ex/card#me";
const BOB: &str = "https://bob.ex/card#me";
const POD_A: &str = "https://a.ex";
const POD_B: &str = "https://b.ex";
const POD_C: &str = "https://c.ex";

fn sess(agent: Option<&str>) -> Session<'_> {
    Session { agent, client: None, issuer: None, now: None }
}

/// The four sessions the differential sweeps: alice, bob, anonymous, and alice via a client.
fn probe_sessions() -> Vec<Session<'static>> {
    vec![
        sess(Some(ALICE)),
        sess(Some(BOB)),
        sess(None),
        Session { agent: Some(ALICE), client: Some("https://app.ex"), issuer: None, now: None },
    ]
}

const MODES: [Mode; 4] = [Mode::Read, Mode::Write, Mode::Append, Mode::Control];

/// One resource in each pod, in ascending IRI order (the differential's decide targets).
fn all_resources() -> Vec<String> {
    ["a.ex", "b.ex", "c.ex"]
        .into_iter()
        .map(|p| format!("https://{p}/n1"))
        .collect()
}

/// A three-pod store, each pod holding one content graph, no ACL yet (fail-closed).
fn three_pod_store() -> PodStore {
    let nq = concat!(
        "<https://a.ex/n1#it> <https://ex.dev/ns#k> \"v\" <https://a.ex/n1> .\n",
        "<https://b.ex/n1#it> <https://ex.dev/ns#k> \"v\" <https://b.ex/n1> .\n",
        "<https://c.ex/n1#it> <https://ex.dev/ns#k> \"v\" <https://c.ex/n1> .\n",
    );
    let mut store = PodStore::new(sparq_core::Graph::load_dataset(nq, "nquads").expect("loads"));
    store.materialize_wac().expect("materializes");
    store
}

/// A root `.acl` body (N-Triples, no graph column) for `pod_origin` granting each agent in
/// `agents` `acl:Read` on the whole pod by `acl:default`. Empty `agents` → an empty document
/// (revokes everything under this ACL).
fn root_acl(pod_origin: &str, agents: &[&str]) -> String {
    let mut s = String::new();
    for (i, a) in agents.iter().enumerate() {
        let subj = format!("{pod_origin}/.acl#auth{i}");
        s.push_str(&format!(
            "<{subj}> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> .\n\
             <{subj}> <http://www.w3.org/ns/auth/acl#default> <{pod_origin}/> .\n\
             <{subj}> <http://www.w3.org/ns/auth/acl#agent> <{a}> .\n\
             <{subj}> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> .\n"
        ));
    }
    s
}

/// Assert the store's scoped-cache view (`accessible` + `decide`) equals a from-scratch
/// `AuthIndex::from_graph` rebuild for every probe session, mode, and resource.
fn assert_equals_fresh_rebuild(store: &mut PodStore, step: &str) {
    let fresh = AuthIndex::from_graph(&store.graph);
    let resources = all_resources();
    for s in probe_sessions() {
        for mode in MODES {
            let got: Vec<String> =
                store.accessible(&s, mode).iter().map(|n| n.as_str().to_owned()).collect();
            let want: Vec<String> =
                fresh.accessible(&s, mode).iter().map(|n| n.as_str().to_owned()).collect();
            assert_eq!(got, want, "accessible diverged from fresh rebuild at [{step}] for {s:?}/{mode:?}");

            // decide() uses the per-origin path (accessible_in_origin); its allow must match
            // membership in the fresh full accessible set.
            for r in &resources {
                let allow = store.decide(&s, r, mode).allow;
                let want_allow = want.iter().any(|g| g == r);
                assert_eq!(allow, want_allow, "decide diverged at [{step}] for {s:?}/{mode:?}/{r}");
            }
        }
    }
}

/// A tiny deterministic LCG (no external `rand` dep) — reproducible pseudo-random ops.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self, n: u64) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) % n
    }
}

#[test]
fn scoped_writes_stay_equivalent_to_full_rebuild_after_every_step() {
    let mut store = three_pod_store();
    let pods = [POD_A, POD_B, POD_C];
    // The candidate ACL bodies a write can install per pod (order = agents granted).
    let bodies: [&[&str]; 4] = [&[ALICE], &[BOB], &[ALICE, BOB], &[]];
    let mut rng = Lcg(0x5eed_1571);

    for i in 0..60 {
        // Warm the cache first (so the NEXT write scoped-invalidates warm entries), then
        // apply a pseudo-random put/delete, then assert equivalence to a full rebuild.
        assert_equals_fresh_rebuild(&mut store, &format!("warm-before-{i}"));

        let pod = pods[rng.next(pods.len() as u64) as usize];
        let acl_iri = format!("{pod}/.acl");
        // 1-in-4 ops delete the ACL; the rest PUT one of the candidate bodies.
        if rng.next(4) == 0 {
            store.delete_acl(&acl_iri).expect("delete_acl");
        } else {
            let body = bodies[rng.next(bodies.len() as u64) as usize];
            store.put_acl(&acl_iri, &root_acl(pod, body), "ntriples").expect("put_acl");
        }
        assert_equals_fresh_rebuild(&mut store, &format!("after-op-{i}"));
    }
}

#[test]
fn revoke_leaves_no_stale_grant_and_isolates_other_pods() {
    // Fail-open-critical (mirrors #1577's delete direction): a scoped revoke on pod a must
    // stop granting on pod a AND leave pod b's warm decisions untouched.
    let mut store = three_pod_store();
    let alice = sess(Some(ALICE));

    // Grant alice Read on pods a and b.
    store.put_acl("https://a.ex/.acl", &root_acl(POD_A, &[ALICE]), "ntriples").expect("put a");
    store.put_acl("https://b.ex/.acl", &root_acl(POD_B, &[ALICE]), "ntriples").expect("put b");

    // Warm the cache + confirm the grants are live on both pods.
    assert!(store.decide(&alice, "https://a.ex/n1", Mode::Read).allow, "a granted");
    assert!(store.decide(&alice, "https://b.ex/n1", Mode::Read).allow, "b granted");
    assert_eq!(store.accessible(&alice, Mode::Read).len(), 2, "both pods warm");

    // Scoped revoke on pod a (delete its .acl). No stale grant may survive.
    store.delete_acl("https://a.ex/.acl").expect("delete a");
    assert!(!store.decide(&alice, "https://a.ex/n1", Mode::Read).allow, "a revoked — no stale grant");
    // Pod b is provably unaffected (different origin) — its warm decision still grants.
    assert!(store.decide(&alice, "https://b.ex/n1", Mode::Read).allow, "b untouched by a's revoke");
    let got: Vec<String> =
        store.accessible(&alice, Mode::Read).iter().map(|n| n.as_str().to_owned()).collect();
    assert_eq!(got, vec!["https://b.ex/n1".to_owned()], "only pod b remains");

    // And the whole store still equals a from-scratch rebuild.
    assert_equals_fresh_rebuild(&mut store, "after-revoke");
}

#[test]
fn write_to_one_pod_never_changes_another_pods_decision() {
    // Cross-pod isolation across a burst of writes to pod c, while pod a's grant is fixed.
    let mut store = three_pod_store();
    let alice = sess(Some(ALICE));
    store.put_acl("https://a.ex/.acl", &root_acl(POD_A, &[ALICE]), "ntriples").expect("put a");
    assert!(store.decide(&alice, "https://a.ex/n1", Mode::Read).allow);

    for body in [&[BOB][..], &[ALICE, BOB][..], &[][..], &[ALICE][..]] {
        store.put_acl("https://c.ex/.acl", &root_acl(POD_C, body), "ntriples").expect("put c");
        // Pod a's decision is invariant under any pod-c churn.
        assert!(store.decide(&alice, "https://a.ex/n1", Mode::Read).allow, "pod a stable under pod c churn");
        assert_equals_fresh_rebuild(&mut store, "pod-c-churn");
    }
}

// ── [OPUS-4.8] sq-b7k7u perf harness (ignored; work-box timings are NON-canonical) ──────
// `cargo test -p sparq-solid --release --test incremental_remat -- --ignored --nocapture`
// The PSS scenario is "reads at full QPS, ACL writes as events", so this isolates the two
// per-REQUEST wins (which run once per read) from the write cost (which runs once per ACL
// change and is UNCHANGED by this slice — the materializer stays whole-store, deferred):
//   A. decide latency — held_modes went from building the whole store's accessible set 4×
//      to one origin's via accessible_in_origin;
//   B. post-scoped-write query latency for a session on ANOTHER pod — its cache slice
//      survives instead of cold-starting (before: a full clear + full re-derive).
//   C. one put_acl (materialize + index rebuild) — reported for honesty; UNCHANGED.
// Run on this branch and on origin/main (this file is unchanged public API) to get before/after.
#[test]
#[ignore]
fn perf_multi_pod_decide_and_query() {
    use std::time::Instant;
    const PODS: usize = 200; // tenants sharing one store
    const RES_PER_POD: usize = 20;

    // PODS pods; each root .acl grants alice Read on its whole subtree — alice's full
    // accessible set is PODS*RES_PER_POD graphs, but each pod's slice is only RES_PER_POD.
    let mut nq = String::new();
    for p in 0..PODS {
        for r in 0..RES_PER_POD {
            nq.push_str(&format!("<https://pod{p}.ex/r{r}#it> <https://ex.dev/ns#k> \"v\" <https://pod{p}.ex/r{r}> .\n"));
        }
        nq.push_str(&root_acl_nq(&format!("https://pod{p}.ex")));
    }
    let mut store = PodStore::new(sparq_core::Graph::load_dataset(&nq, "nquads").expect("loads"));
    store.materialize_wac().expect("materializes");
    let alice = sess(Some(ALICE));

    // A. Pure per-request decide latency (warm index, no writes).
    let k = 20_000usize;
    let t = Instant::now();
    let mut acc = 0usize;
    for i in 0..k {
        let p = i % PODS;
        let r = i % RES_PER_POD;
        if store.decide(&alice, &format!("https://pod{p}.ex/r{r}"), Mode::Read).allow {
            acc += 1;
        }
    }
    let a = t.elapsed();
    println!("[sq-b7k7u] A decide: {k} decides over {PODS} pods -> {a:?} ({:?}/decide), allow={acc}", a / k as u32);

    // B. Post-scoped-write query latency for a DIFFERENT pod's session. Warm two sessions,
    // write pod0's .acl, then time the pod-1-only session's next accessible() (its slice is
    // untouched by the pod0 write; before this change the full clear cold-started it).
    let s_far = Session { agent: Some(ALICE), client: Some("https://only-pod1.ex"), issuer: None, now: None };
    let _ = store.accessible(&alice, Mode::Read).len();
    let _ = store.accessible(&s_far, Mode::Read).len(); // warm the far session too
    store.put_acl("https://pod0.ex/.acl", &root_acl("https://pod0.ex", &[ALICE]), "ntriples").expect("put");
    let t = Instant::now();
    let n = store.accessible(&s_far, Mode::Read).len();
    let b = t.elapsed();
    println!("[sq-b7k7u] B post-write query (far session, {n} graphs): {b:?}");

    // C. One put_acl = materialize + index rebuild (UNCHANGED whole-store cost; honesty).
    let t = Instant::now();
    store.put_acl("https://pod0.ex/.acl", &root_acl("https://pod0.ex", &[ALICE]), "ntriples").expect("put");
    println!("[sq-b7k7u] C one put_acl (materialize+reindex, UNCHANGED): {:?}", t.elapsed());
}

/// A pod root `.acl` in NQUADS (graph column = the `.acl`) granting alice Read by default.
fn root_acl_nq(pod: &str) -> String {
    format!(
        "<{pod}/.acl#o> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/ns/auth/acl#Authorization> <{pod}/.acl> .\n\
         <{pod}/.acl#o> <http://www.w3.org/ns/auth/acl#default> <{pod}/> <{pod}/.acl> .\n\
         <{pod}/.acl#o> <http://www.w3.org/ns/auth/acl#agent> <{ALICE}> <{pod}/.acl> .\n\
         <{pod}/.acl#o> <http://www.w3.org/ns/auth/acl#mode> <http://www.w3.org/ns/auth/acl#Read> <{pod}/.acl> .\n"
    )
}

