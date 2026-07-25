// [FABLE-5] sq-ixc3.15 — the ODRL policy tool's native command: author/validate a policy,
// evaluate a request, and preview the access-controlled result set next to the ungated one.
//
// The GUI's in-tab WASM engine ships neither the ODRL evaluator (sparq-policy) nor the
// usage-control enforcement store (sparq-solid's `PodStore` + odrl-bridge), so this whole
// tool is NATIVE-ONLY: exactly ONE command, `odrl_preview`, scoped to the one round-trip the
// webview cannot do itself — parse a Turtle ODRL policy, evaluate the user's request
// (party/action/target), materialize the policy through the odrl-bridge into a `PodStore`,
// and run the SAME SPARQL query as each requester through `query_json_as` (fail-closed
// per-session named-graph gating) alongside the ungated evaluation over the raw dataset.
//
// OPT-IN: the capability sits behind this crate's non-default `odrl` cargo feature (pulling
// the optional `sparq-policy` + `sparq-solid`/odrl-bridge crates). A lean build compiles a
// stub that fails LOUDLY with a rebuild hint — never a silent no-op — the same discipline as
// the `hdt` and `federation` features.
//
// FAIL-CLOSED INVARIANTS (mirrored in the frontend and asserted by the tests below):
//   - A malformed policy materializes NOTHING: every requester sees ZERO rows (an empty
//     authorized-graph set is the PodStore default), and the parse error is returned verbatim
//     as the visible reason. Deny-everything, never a guess.
//   - A policy whose `odrl:conflict` strategy the bridge cannot honor is REFUSED whole
//     (`BridgeOutcome::refused`): again nothing materializes, and the reasons are surfaced.
//   - A requester with no applicable grant gets an `Ok` result with zero rows (authorization
//     never errors) — indistinguishable from the graphs being absent.
//
// WIDENING HAZARD deliberately avoided: this command uses ONLY the one-shot
// `materialize_odrl_policy` path, which honors a bare `odrl:assignee` rule attribute. The
// `*_conditional` bridge variants drop a bare assignee to `auth:Public` (bead sq-9n1q4) and
// are NOT wired here.

/// One requester's pane in the two-pane preview: the ODRL decision for the explicit request,
/// the bridge's materialization notes, and the gated result set (SPARQL 1.1 JSON) produced by
/// `PodStore::query_json_as` under that requester's session.
#[derive(Debug, serde::Serialize)]
pub struct OdrlPane {
    /// The requester's WebID (echoed so the frontend can label the pane).
    pub requester: String,
    /// The ODRL decision for the explicit (action, target, this-party) request.
    pub allow: bool,
    /// Rule ids that produced the decision (granting permission / overriding prohibition).
    pub matched_rules: Vec<String>,
    /// Human-readable reasons each permission did NOT grant (unmet constraints).
    pub unmet_constraints: Vec<String>,
    /// The bridge's materialization notes for this requester across every rule target —
    /// which auth grants/denies were written, or why nothing was.
    pub bridge_notes: Vec<String>,
    /// The gated result set: the SAME query, evaluated through the fail-closed per-session
    /// named-graph view. SPARQL 1.1 JSON results document.
    pub results_json: String,
}

/// The whole preview: policy validation status, the ungated evaluation, and one pane per
/// requester. `policy_ok == false` ⇒ deny-everything (every pane has zero rows) with
/// `policy_error` as the visible reason.
#[derive(Debug, serde::Serialize)]
pub struct OdrlPreview {
    /// Whether the policy parsed as Turtle ODRL. False ⇒ NOTHING was materialized.
    pub policy_ok: bool,
    /// The verbatim parse error when `policy_ok` is false (the visible fail-closed reason).
    pub policy_error: Option<String>,
    /// Parsed rule counts (0 when the policy is malformed).
    pub permissions: usize,
    pub prohibitions: usize,
    /// True when the policy's `odrl:conflict` strategy is one the bridge cannot honor: the
    /// WHOLE policy is refused and nothing materializes (fail-closed, reasons in the panes).
    pub refused: bool,
    /// The ungated evaluation of the same query over the raw dataset (SPARQL 1.1 JSON).
    pub ungated_json: String,
    /// One pane per requester, in the order given.
    pub panes: Vec<OdrlPane>,
}

/// Author/validate → evaluate → preview, in one native round-trip. `dataset` is TriG or
/// N-Quads (`format`), `policy` is Turtle ODRL, `action`/`target` are IRIs and `requesters`
/// WebIDs; `query` is the SPARQL SELECT/ASK run once ungated and once per requester through
/// the fail-closed gated view.
#[tauri::command]
pub fn odrl_preview(
    dataset: String,
    format: String,
    policy: String,
    action: String,
    target: String,
    requesters: Vec<String>,
    query: String,
) -> Result<OdrlPreview, String> {
    run_odrl_preview(&dataset, &format, &policy, &action, &target, requesters, &query)
}

/// The command body, factored out so it is unit-testable without a Tauri runtime (the same
/// pattern as `federation::run_service_query`).
#[cfg(feature = "odrl")]
fn run_odrl_preview(
    dataset: &str,
    format: &str,
    policy_text: &str,
    action: &str,
    target: &str,
    requesters: Vec<String>,
    query: &str,
) -> Result<OdrlPreview, String> {
    use sparq_policy::{evaluate, parse_policy_str, Request};
    use sparq_solid::{Mode, PodStore, Session};

    // An empty dataset is a legal store (author a policy before loading data).
    let graph = if dataset.trim().is_empty() {
        sparq_core::Graph::new()
    } else {
        sparq_core::Graph::load_dataset(dataset, format)?
    };

    // The UNGATED pane: the same query over the raw dataset, before any gating. Evaluated
    // first because `PodStore::new` takes ownership of the graph.
    let ungated_json = sparq_engine::query_json(&graph, query)?;

    // The enforcement store: strips reserved graphs, starts with an EMPTY auth index — the
    // fail-closed default every requester keeps unless the policy grants otherwise.
    let mut store = PodStore::new(graph);

    // (a) Parse/validate. Malformed ⇒ deny-everything: materialize NOTHING and surface the
    // parse error verbatim; the gated queries below then run against the empty auth index.
    let (policy, policy_error) = match parse_policy_str(policy_text, "turtle") {
        Ok(p) => (Some(p), None),
        Err(e) => (None, Some(e)),
    };

    // (b)+(c) Evaluate the explicit request per requester, then materialize the policy for
    // every (requester × rule-target) pair so the preview reflects the WHOLE policy — a rule
    // targeting another graph still shapes that requester's view. Grants/denies written by
    // the one-shot bridge are strictly per-principal, so requesters cannot widen each other.
    let mut refused = false;
    let mut panes: Vec<OdrlPane> = Vec::with_capacity(requesters.len());
    for webid in &requesters {
        let mut pane = OdrlPane {
            requester: webid.clone(),
            allow: false, // fail-closed default (empty/malformed policy denies)
            matched_rules: Vec::new(),
            unmet_constraints: Vec::new(),
            bridge_notes: Vec::new(),
            results_json: String::new(),
        };
        if let Some(policy) = &policy {
            let request = Request::new(action).on(target).by(webid.as_str());
            let decision = evaluate(policy, &request);
            pane.allow = decision.allow;
            pane.matched_rules = decision.matched_rules;
            pane.unmet_constraints = decision.unmet_constraints;

            // Every distinct rule target, with the explicit request target included — sorted
            // (BTreeSet) so the notes are deterministic.
            let mut targets: std::collections::BTreeSet<String> = policy
                .permissions
                .iter()
                .chain(policy.prohibitions.iter())
                .filter_map(|r| r.target.clone())
                .collect();
            targets.insert(target.to_owned());
            for t in &targets {
                let req = Request::new(action).on(t.as_str()).by(webid.as_str());
                let outcome = store.materialize_odrl_policy(policy, &req);
                if outcome.refused {
                    refused = true;
                }
                if let Some((p, m, g)) = &outcome.grant_triple {
                    pane.bridge_notes.push(format!("grant: {p} {m} {g}"));
                }
                if let Some((p, m, g)) = &outcome.deny_triple {
                    pane.bridge_notes.push(format!("deny: {p} {m} {g}"));
                }
                for reason in outcome.reasons {
                    pane.bridge_notes.push(format!("{t}: {reason}"));
                }
            }
        } else if let Some(err) = &policy_error {
            pane.bridge_notes
                .push(format!("policy malformed — nothing materialized (deny-everything): {err}"));
        }
        panes.push(pane);
    }

    // The GATED panes: the SAME query per requester through the fail-closed per-session
    // named-graph view. Run AFTER all materialization so every pane sees the final auth
    // state. Authorization never errors: no grant ⇒ zero rows, not an Err.
    for pane in &mut panes {
        let session = Session {
            agent: Some(pane.requester.as_str()),
            ..Session::default()
        };
        pane.results_json = store.query_json_as(&session, Mode::Read, query)?;
    }

    let (permissions, prohibitions) = policy
        .as_ref()
        .map(|p| (p.permissions.len(), p.prohibitions.len()))
        .unwrap_or((0, 0));
    Ok(OdrlPreview {
        policy_ok: policy_error.is_none(),
        policy_error,
        permissions,
        prohibitions,
        refused,
        ungated_json,
        panes,
    })
}

/// Lean-build stub: the ODRL evaluator + enforcement store are compiled out, so the tool
/// fails LOUDLY with an actionable rebuild hint instead of silently mis-reporting (the same
/// discipline as the `hdt` / `federation` features).
#[cfg(not(feature = "odrl"))]
fn run_odrl_preview(
    _dataset: &str,
    _format: &str,
    _policy: &str,
    _action: &str,
    _target: &str,
    _requesters: Vec<String>,
    _query: &str,
) -> Result<OdrlPreview, String> {
    Err(
        "The ODRL policy tool is not compiled into this build — rebuild the desktop app with \
         `cargo build --features odrl` to evaluate policies and preview gated results."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two named graphs: one public report, one secret memo.
    const DATASET_TRIG: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:public { ex:doc1 ex:title "Public report" . }
        ex:secret { ex:doc2 ex:title "Secret memo" . }
    "#;

    /// The SAME query every pane runs: everything, per named graph.
    const QUERY: &str =
        "SELECT ?g ?s ?title WHERE { GRAPH ?g { ?s <http://example.org/title> ?title } }";

    const ODRL_READ: &str = "http://www.w3.org/ns/odrl/2/read";
    const ALICE: &str = "http://example.org/alice";
    const BOB: &str = "http://example.org/bob";

    /// Permissions on BOTH graphs for anyone (no assignee).
    #[cfg(feature = "odrl")]
    const POLICY_PERMIT_ALL: &str = r#"
        @prefix odrl: <http://www.w3.org/ns/odrl/2/> .
        @prefix ex: <http://example.org/> .
        ex:policy1 a odrl:Set ;
          odrl:permission [ odrl:action odrl:read ; odrl:target ex:public ] ;
          odrl:permission [ odrl:action odrl:read ; odrl:target ex:secret ] .
    "#;

    /// The same permissions PLUS a prohibition scoped to alice on the secret graph.
    #[cfg(feature = "odrl")]
    const POLICY_PROHIBIT_ALICE_SECRET: &str = r#"
        @prefix odrl: <http://www.w3.org/ns/odrl/2/> .
        @prefix ex: <http://example.org/> .
        ex:policy1 a odrl:Set ;
          odrl:permission [ odrl:action odrl:read ; odrl:target ex:public ] ;
          odrl:permission [ odrl:action odrl:read ; odrl:target ex:secret ] ;
          odrl:prohibition [ odrl:action odrl:read ; odrl:target ex:secret ;
                             odrl:assignee ex:alice ] .
    "#;

    #[cfg(feature = "odrl")]
    fn preview(policy: &str) -> OdrlPreview {
        run_odrl_preview(
            DATASET_TRIG,
            "trig",
            policy,
            ODRL_READ,
            "http://example.org/secret",
            vec![ALICE.to_string(), BOB.to_string()],
            QUERY,
        )
        .expect("preview must succeed")
    }

    /// ACCEPTANCE (sq-ixc3.15): an `odrl:prohibition` flips a PREVIOUSLY VISIBLE named graph
    /// to hidden in the requester's preview pane — while the other requester (and the ungated
    /// pane) still see it. Mutating the bridge to skip the deny triple flips this red.
    #[test]
    #[cfg(feature = "odrl")]
    fn prohibition_flips_a_previously_visible_graph_to_hidden() {
        // WITHOUT the prohibition: alice sees the secret memo (previously visible)…
        let before = preview(POLICY_PERMIT_ALL);
        assert!(before.policy_ok);
        assert!(
            before.panes[0].results_json.contains("Secret memo"),
            "alice must see the secret graph before the prohibition: {}",
            before.panes[0].results_json
        );
        assert!(before.panes[0].allow, "the explicit request must be permitted");

        // …WITH the prohibition: alice's pane hides it, bob's and the ungated pane keep it.
        let after = preview(POLICY_PROHIBIT_ALICE_SECRET);
        assert!(after.policy_ok);
        assert_eq!(after.prohibitions, 1);
        assert!(
            !after.panes[0].results_json.contains("Secret memo"),
            "the prohibition must hide the secret graph from alice: {}",
            after.panes[0].results_json
        );
        assert!(
            after.panes[0].results_json.contains("Public report"),
            "the prohibition must NOT hide the still-permitted public graph: {}",
            after.panes[0].results_json
        );
        assert!(!after.panes[0].allow, "the explicit request must now be denied");
        assert!(
            after.panes[1].results_json.contains("Secret memo"),
            "bob is not the prohibition's assignee — his pane keeps the graph: {}",
            after.panes[1].results_json
        );
        assert!(after.panes[1].allow);
        assert!(
            after.ungated_json.contains("Secret memo"),
            "the ungated pane is never gated: {}",
            after.ungated_json
        );
    }

    /// ACCEPTANCE (sq-ixc3.15): a MALFORMED policy denies everything (fail-closed) with the
    /// parse error as the visible reason — zero rows in every gated pane even though the
    /// ungated pane proves the data is there.
    #[test]
    #[cfg(feature = "odrl")]
    fn malformed_policy_denies_everything_with_a_visible_reason() {
        let out = preview("this is @@ not turtle ;;");
        assert!(!out.policy_ok);
        let reason = out.policy_error.as_deref().expect("the parse error must be visible");
        assert!(!reason.is_empty());
        assert_eq!((out.permissions, out.prohibitions), (0, 0));
        for pane in &out.panes {
            assert!(
                !pane.allow,
                "a malformed policy must deny {}",
                pane.requester
            );
            assert!(
                !pane.results_json.contains("Public report")
                    && !pane.results_json.contains("Secret memo"),
                "a malformed policy must materialize NOTHING — got rows for {}: {}",
                pane.requester,
                pane.results_json
            );
        }
        assert!(
            out.ungated_json.contains("Secret memo"),
            "the ungated pane must prove the data exists: {}",
            out.ungated_json
        );
    }

    /// Fail-closed default: a requester with NO applicable grant gets zero rows (`Ok`, never
    /// an error) — a permission scoped to bob alone leaves alice's pane empty.
    #[test]
    #[cfg(feature = "odrl")]
    fn requester_without_a_grant_sees_nothing() {
        let out = run_odrl_preview(
            DATASET_TRIG,
            "trig",
            r#"
              @prefix odrl: <http://www.w3.org/ns/odrl/2/> .
              @prefix ex: <http://example.org/> .
              ex:policy1 a odrl:Set ;
                odrl:permission [ odrl:action odrl:read ; odrl:target ex:secret ;
                                  odrl:assignee ex:bob ] .
            "#,
            ODRL_READ,
            "http://example.org/secret",
            vec![ALICE.to_string(), BOB.to_string()],
            QUERY,
        )
        .expect("preview must succeed");
        assert!(!out.panes[0].allow, "alice is not the assignee");
        assert!(
            !out.panes[0].results_json.contains("Secret memo"),
            "no grant ⇒ no rows for alice: {}",
            out.panes[0].results_json
        );
        assert!(out.panes[1].allow, "bob is the assignee");
        assert!(
            out.panes[1].results_json.contains("Secret memo"),
            "bob's grant must surface the graph: {}",
            out.panes[1].results_json
        );
    }

    /// Lean build (feature OFF): the stub fails LOUDLY with the actionable rebuild hint —
    /// never a silent no-op or a fabricated preview.
    #[test]
    #[cfg(not(feature = "odrl"))]
    fn lean_build_fails_loudly_with_a_rebuild_hint() {
        let err = run_odrl_preview(
            DATASET_TRIG,
            "trig",
            "",
            ODRL_READ,
            "http://example.org/secret",
            vec![ALICE.to_string(), BOB.to_string()],
            QUERY,
        )
        .expect_err("the lean-build stub must be an Err");
        assert!(err.contains("--features odrl"), "got: {err}");
    }
}
