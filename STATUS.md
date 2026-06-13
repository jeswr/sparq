# zk-trace engine module (module B) — STATUS

Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade when Fable returns).
Branch: zk-trace-engine (worktree sparq-zktrace). NEVER push/merge.

## Done
- Salvaged the predecessor's per-operator trace upgrade (applied cleanly to
  merged main): named-graph attribution, operator boundaries (Enter/Exit),
  FILTER operand dedup, canonical determinism, EXISTS/empty-pattern/path
  scaffolding.
- Wired the call sites the salvage left open: GRAPH ?g per-iteration tag,
  result-preserving plan changes under an armed recorder (try_count / try_capped
  / split_sargable all bypass to the materialising path so the input set is
  always captured in full), Op::Path / Op::QuotedTriples fail-closed markers,
  exists_scope, record_empty_pattern at unsat BGP returns.
- Zero-cost-when-off: cfg-gated the one non-default param (eval_translated gname)
  so the default/wasm build carries no zk threading.

## Coverage (per-operator)
BGP scan / bind-join / WCOJ, FILTER (incl. sargable now residualised under
recorder), OPTIONAL / UNION / MINUS / GRAPH(const+var) / DISTINCT / GROUP-agg,
unsatisfiable (empty witness), ASK/COUNT (pushdown disabled). Property paths +
RDF-1.2 quoted triples: fail-closed markers (first_uncaptured). EXISTS: tagged,
suppressed (outside stage-1 fragment).

## Tests
- engine zk unit: 3; engine zk integration (operators): 14; differential 10k
  (zk-on == zk-off across all shapes): green.
- sparq-zk: trace_guard 7 (Q6 bnode), trace_named_graph 3 (attribution),
  full crate 37+.

## Open / flagged
- Derived-triple (entailmentRegime != none) classification: NOT in this module
  (needs reasoner base set; lives in sparq-zk trace_infer + sparq-reason
  explain ProofTrees). Documented in zk.rs.
- OPTIONAL/MINUS/UNION sets are conservative supersets (post-scan consumed);
  flat commitments make supersets padding not soundness (plan §2.2).
- wasm byte delta: +8B vs main's committed artifact, within documented
  ±8B path noise; only non-cfg'd change is a codegen-neutral match-arm block.

## Results
- Workspace gate (cargo test --workspace --exclude sparq-py --release
  --no-fail-fast): EXIT=0, 734 passed / 0 failed across 113 suites.
- engine zk: 16 operator tests + 10k differential (zk-on == zk-off) green.
- Trace overhead (credential scale, proving path): ~2.5-4x untraced — a
  constant factor (materialises the full consumed input set + disables
  result-only pushdowns). Disarmed: one thread-local read per scan.
- wasm: default/wasm build byte count vs main within documented ±8B path
  noise (all zk code cfg'd out; the one non-default param is cfg-gated).
- roborev: latest commit reviewed "No issues found"; all prior findings fixed.
