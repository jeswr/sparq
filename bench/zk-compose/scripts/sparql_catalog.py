#!/usr/bin/env python3
# [OPUS-4.8] written while Fable 5 unavailable — re-review when Fable returns.
#
# sq-1s2.1.2: Comprehensive SPARQL ZK gate-cost benchmark CATALOG generator.
#
# Builds bench/zk-compose/sparql_feature_catalog.json — a coverage map AND an
# optimisation-target list for the ZK-SPARQL engine, faithfully implementing the
# §6 catalog design of research/zk-field-native-encoding.md (PR #765).
#
# What it does (and, just as importantly, what it does NOT do):
#
#   * The QUERY catalog (the SPARQL 1.1 feature span, the representative query
#     shapes, and the member-NAME mapping) is authored here as CATALOG, below —
#     the design's §6.2 table, faithfully transcribed.
#   * The `circuit_size` for every COVERED query is NOT written here. It is
#     JOINED from the regression-gated snapshot
#     (crates/sparq-zk-compose/tests/gate_count_snapshot.json) — the SAME source
#     of truth bench/zk-compose/scripts/gate_counts.sh and the in-crate
#     `gate_count_regression` test (tests/gate_count.rs) use. So a member's gate
#     count is NEVER hand-copied into two files that can disagree, and the
#     benchmark can never drift from the regression-gated members.
#   * For a feature with NO ZK circuit today (general property-path traversal,
#     aggregates, BIND expression eval, string predicates, dateTime compare,
#     negation, boolean FILTER) we record `circuit_size: null` and
#     `status: "NO ZK CIRCUIT YET (gap)"`. We NEVER fabricate a gate number.
#
# Honesty is load-bearing (§6.1): the benchmark is a coverage map, not a
# cherry-picked win list. The `gap` rows are the larger story; surfacing them is
# the point. The high-gate covered members (the blake3-bound `filter_*` family at
# 17,416) are auto-flagged HIGH_GATE_* as the value-hook reduction target (§2.6).
#
# sq-3kd2g.11 refresh — status now reflects ACCEPTED SYNTAX at the verifier
# fragment gate (crates/sparq-zk/src/verify.rs::fragment_query), not merely
# whether a desugared form compiles to some circuit. The fragment-gate extension
# (sq-3kd2g.3 gate + sq-3kd2g.6 builder/dispatch, research/zksparql-fragment-
# extension.md §§1,3,4,7) made property paths, UNION, VALUES and subqueries
# ACCEPTED, while OPTIONAL/MINUS/BIND/aggregates/EXISTS stay REJECTED fail-closed.
# So a row is `covered` only when the gate accepts the syntax AND a dedicated
# circuit member proves it; `partial` when accepted but composed verifier-side
# (no dedicated member); `gap` when the gate rejects it fail-closed. This fixes
# the §1 Q13/Q14 drift where `covered` had meant only "a desugared form compiles"
# while the path syntax was still rejected. The bounded closures p?/p+/p* map to
# the path_reach family under an EXISTENCE-ONLY bounded statement (§4) and carry
# a `bounded` honesty flag; the depth bound is a public input, never a claim of
# unbounded closure.
#
# It does NOT require live `bb`/`nargo`: every number comes off the committed
# snapshot, so the harness is deterministic and non-flaky (CI-safe). To refresh
# the underlying gate counts, re-run bench/zk-compose/scripts/gate_counts.sh and
# re-baseline the snapshot; this generator then re-joins the new numbers.
#
# Usage:
#   bench/zk-compose/scripts/sparql_catalog.py \
#       > bench/zk-compose/sparql_feature_catalog.json
#
# The committed JSON is regression-gated by the Rust test
# `crates/sparq-zk-compose/tests/sparql_catalog.rs`, which fails if any covered
# query names a member absent from the snapshot, if any covered `circuit_size`
# disagrees with the snapshot, or if any gap carries a non-null gate number.

from __future__ import annotations

import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
SNAPSHOT_JSON = os.path.normpath(
    os.path.join(HERE, "..", "..", "..", "crates", "sparq-zk-compose", "tests", "gate_count_snapshot.json")
)

# The blake3-bound numeric-FILTER members all land on this gate count; any covered
# query whose joined circuit_size meets or exceeds it is the value-hook reduction
# target (research/zk-field-native-encoding.md §2.6). Sourced as a member's joined
# value, not asserted: see HIGH_GATE_THRESHOLD use below.
HIGH_GATE_THRESHOLD = 17416

# Status strings — the gap label MUST contain "NO ZK CIRCUIT YET" so the Rust test
# and any downstream surface can detect a gap by substring.
STATUS_COVERED = "covered"
STATUS_PARTIAL = "partial (verifier-side / desugars to a covered primitive)"
STATUS_GAP = "NO ZK CIRCUIT YET (gap)"

# The value-hook reduction target reference, shared by every flagged high-gate
# numeric-FILTER member.
VALUE_HOOK_TARGET = "value-hook (research/zk-field-native-encoding.md §2.6)"
# An ESTIMATE, explicitly NOT a measured number — it MUST be re-measured with
# `bb gates` once the value-hook lands (§6.3). Carried as prose with the ESTIMATE
# qualifier so it can never be mistaken for a measurement.
VALUE_HOOK_PROJECTED = "ESTIMATE ~3200 — MUST be re-measured with bb gates (NOT a measurement)"

# The compiled bounded-depth path-reachability family (the sq-3kd2g.2 circuits,
# dispatched by sq-3kd2g.6's builder under the opt-in `extended-fragment` feature).
# These members carry the property-path closures p?/p+/p* under the BOUNDED,
# EXISTENCE-ONLY statement of research/zksparql-fragment-extension.md §4 — the depth
# bound is a PUBLIC input; a bounded proof never asserts absence or completeness.
# circuit_size is joined from the snapshot like every other member (never fabricated).
PATH_REACH_MEMBERS = [
    "path_reach_d2_k1_n16",
    "path_reach_d4_k1_n16",
    "path_reach_d4_k2_n16",
    "path_reach_d8_k1_n16",
]

# ---------------------------------------------------------------------------
# The query catalog — §6.2 of research/zk-field-native-encoding.md, transcribed.
#
# Each entry is the AUTHORITATIVE design row: an id, the SPARQL 1.1 feature it
# exercises, a representative query, the ZK circuit member(s) it compiles to (by
# NAME — circuit_size is joined from the snapshot), a coverage status, and, for
# covered high-gate members, a reduction target. NO circuit_size literal lives
# here; that is the anti-drift contract.
#
# `zk_members`: member names that MUST exist in the snapshot (joined → circuit_size).
#               Empty list ⇒ gap or verifier-side-only (circuit_size = null).
# `representative_members`: for a covered query that SPANS a family (e.g. a BGP scan
#               over the (k,n,r) lattice), the subset whose joined sizes are reported
#               as the cost RANGE. Defaults to zk_members when omitted.
# ---------------------------------------------------------------------------
CATALOG: list[dict] = [
    {
        "id": "Q01_bgp_single_pattern",
        "feature": "BGP (1 triple pattern)",
        "sparql": "SELECT ?s ?o WHERE { ?s :p ?o }",
        "zk_members": ["scan_k1_n16_r4", "scan_k1_n16_r8", "scan_k1_n64_r4", "scan_k1_n64_r8"],
        "status": STATUS_COVERED,
        "note": "Maps to the disclosed-row scan family over the (n,r) lattice (k=1, one graph).",
    },
    {
        "id": "Q02_bgp_multi_pattern",
        "feature": "BGP (multi-pattern, single graph)",
        "sparql": "SELECT ?s ?v WHERE { ?s :p ?o . ?s :q ?v }",
        "zk_members": ["scan_k2_n16_r4", "scan_k2_n16_r8", "scan_k2_n64_r4", "scan_k2_n64_r8"],
        "status": STATUS_COVERED,
        "note": "Multiple shared-subject patterns map to the k=2 scan lattice (k = committed-graph / pattern width).",
    },
    {
        "id": "Q03_filter_integer_ge",
        "feature": "FILTER xsd:integer (>=, <, =, !=)",
        "sparql": "SELECT ?p WHERE { ?p :age ?age FILTER(?age >= 18) }",
        "zk_members": ["filter_int_d1", "filter_int_d2", "filter_int_d3", "filter_int_d4"],
        "status": STATUS_COVERED,
        "reduction_target": VALUE_HOOK_TARGET,
        "projected_after": VALUE_HOOK_PROJECTED,
        "note": "The complex numeric FILTER family. Gate-identical across d (the blake3 token fits one 64-byte block for d<=19).",
    },
    {
        "id": "Q04_filter_signed_integer_ge",
        "feature": "FILTER signed xsd:integer (negative operands)",
        "sparql": "SELECT ?a WHERE { ?a :balance ?bal FILTER(?bal >= -100) }",
        "zk_members": ["filter_signed_int_d2", "filter_signed_int_d4"],
        "status": STATUS_COVERED,
        "reduction_target": VALUE_HOOK_TARGET,
        "projected_after": VALUE_HOOK_PROJECTED,
        "note": "Signed-int lane; same blake3-binding cost driver as filter_int.",
    },
    {
        "id": "Q05_filter_decimal_le",
        "feature": "FILTER xsd:decimal",
        "sparql": "SELECT ?o WHERE { ?o :amount ?amt FILTER(?amt <= 199.99) }",
        "zk_members": ["filter_decimal_i3_f2"],
        "status": STATUS_COVERED,
        "reduction_target": VALUE_HOOK_TARGET,
        "projected_after": VALUE_HOOK_PROJECTED,
        "note": "Scaled-decimal lane (i integer digits, f fractional digits).",
    },
    {
        "id": "Q06_filter_double_gt",
        "feature": "FILTER xsd:double (IEEE-754 compare)",
        "sparql": "SELECT ?x WHERE { ?x :reading ?d FILTER(?d > 42.0) }",
        "zk_members": ["filter_f64_d1", "filter_f64_d2", "filter_f64_d3", "filter_f64_d4"],
        # The raw-compare floor (a pure sparq_ieee754 comparison, no string hash) is
        # reported alongside as the lower bound the value-hook is reaching toward.
        "floor_member": "filter_f64",
        "status": STATUS_COVERED,
        "reduction_target": VALUE_HOOK_TARGET,
        "projected_after": VALUE_HOOK_PROJECTED,
        "note": "Manifest-composable double FILTER, blake3-bound like filter_int; the raw-compare floor `filter_f64` shows the achievable lower bound.",
    },
    {
        "id": "Q07_filter_boolean_eq",
        "feature": "FILTER boolean / term = literal",
        "sparql": "SELECT ?x WHERE { ?x :active ?flag FILTER(?flag = true) }",
        "zk_members": [],
        "status": STATUS_GAP,
        "reduction_target": "boolean VALUE_HOOK proposed (research/zk-field-native-encoding.md §2.5)",
        "note": "Rejected fail-closed by the fragment gate — the expression estate is phase 3 (not yet bound). No boolean predicate circuit today; a boolean VALUE_HOOK is design-only.",
    },
    {
        "id": "Q08_filter_string_ops",
        "feature": "FILTER string ops (STR, REGEX, CONTAINS, STRLEN)",
        "sparql": 'SELECT ?x WHERE { ?x :name ?n FILTER(CONTAINS(STR(?n), "a")) }',
        "zk_members": [],
        "status": STATUS_GAP,
        "note": "Rejected fail-closed by the fragment gate (expression estate is phase 3). String lane commits the opaque term hash only; no in-circuit string predicate.",
    },
    {
        "id": "Q09_filter_datetime_compare",
        "feature": "FILTER xsd:dateTime compare",
        "sparql": 'SELECT ?e WHERE { ?e :when ?d FILTER(?d >= "2020-01-01T00:00:00Z"^^xsd:dateTime) }',
        "zk_members": [],
        "status": STATUS_GAP,
        "reduction_target": "dateTime VALUE_HOOK design-only (research/zk-field-native-encoding.md §2.5)",
        "note": "Rejected fail-closed by the fragment gate (expression estate is phase 3). No dateTime ordering circuit; a dateTime VALUE_HOOK (epoch-seconds field handle) is design-only.",
    },
    {
        "id": "Q10_optional",
        "feature": "OPTIONAL (left join)",
        "sparql": "SELECT ?s ?v WHERE { ?s :p ?o OPTIONAL { ?s :q ?v } }",
        "zk_members": [],
        "status": STATUS_GAP,
        "note": "REJECTED fail-closed by the fragment gate (VerifyError::UnsupportedFragment). OUT by design, not a build backlog: OPTIONAL/LeftJoin is non-monotone — an unbound optional side asserts no compatible extension exists (a closed-world claim). Rewrite to Join, or to a UNION of explicit cases (design record §3.2).",
    },
    {
        "id": "Q11_union",
        "feature": "UNION",
        "sparql": "SELECT ?s WHERE { { ?s :p ?o } UNION { ?s :q ?o } }",
        "zk_members": [],
        "status": STATUS_PARTIAL,
        "note": "ACCEPTED by the fragment gate as per-branch attribution; each branch is a scan proof, the union composed verifier-side (no dedicated circuit member).",
    },
    {
        "id": "Q12_join_shared_var_hidden",
        "feature": "JOIN on a shared variable (hidden equi-join)",
        "sparql": "SELECT ?p ?s WHERE { ?p :worksFor :ACME . ?p :hasSalary ?s }",
        "zk_members": ["join_eq_na16_nb16", "join_eq_na16_nb64", "join_eq_na64_nb16", "join_eq_na64_nb64"],
        "status": STATUS_COVERED,
        "note": "Hidden equi-join over the (na,nb) bucket lattice.",
    },
    {
        "id": "Q13_path_sequence",
        "feature": "Property path / (sequence)",
        "sparql": "SELECT ?o WHERE { ?s :p/:q ?o }",
        "zk_members": ["scan_k2_n16_r4", "scan_k2_n16_r8", "scan_k2_n64_r4", "scan_k2_n64_r8"],
        "status": STATUS_COVERED,
        "note": "Path syntax now ACCEPTED by the fragment gate (rewritten to a fresh-intermediate BGP — the SPARQL 1.1 translation, design §4); the desugared multi-pattern BGP maps to the k=2 scan lattice. (Pre-extension this row was 'covered' only because the desugared form compiled while the path syntax was still rejected — §1 drift, now resolved.)",
    },
    {
        "id": "Q14_path_inverse",
        "feature": "Property path ^ (inverse)",
        "sparql": "SELECT ?s WHERE { ?o ^:p ?s }",
        "zk_members": ["scan_k1_n16_r4", "scan_k1_n16_r8", "scan_k1_n64_r4", "scan_k1_n64_r8"],
        "status": STATUS_COVERED,
        "note": "Path syntax now ACCEPTED by the fragment gate (endpoint swap, design §4); maps to the k=1 scan family (no extra cost over a forward pattern). (Pre-extension this row was 'covered' only because the desugared form compiled while the path syntax was still rejected — §1 drift, now resolved.)",
    },
    {
        "id": "Q15_path_alternative",
        "feature": "Property path | (alternative)",
        "sparql": "SELECT ?o WHERE { ?s (:p|:q) ?o }",
        "zk_members": [],
        "status": STATUS_PARTIAL,
        "note": "ACCEPTED by the fragment gate; desugars to a UNION of per-predicate branches, composed verifier-side (as Q11).",
    },
    {
        "id": "Q16_path_zero_or_one",
        "feature": "Property path ? (zero-or-one)",
        "sparql": "SELECT ?o WHERE { ?s :p? ?o }",
        "zk_members": PATH_REACH_MEMBERS,
        "status": STATUS_COVERED,
        "bounded": True,
        "note": "ACCEPTED by the fragment gate; maps to the bounded path_reach family (opt-in `extended-fragment`). p? is pinned to depth bound 1 (PathClosure::fixed_k) with the zero-length case (endpoint equality + an occurrence witness). EXISTENCE-ONLY bounded statement (design §4); internally re-audited, external cryptographer sign-off pending (sq-qhy4) — NOT a proven soundness claim.",
    },
    {
        "id": "Q17_path_one_or_more",
        "feature": "Property path + (one-or-more, transitive closure)",
        "sparql": "SELECT ?o WHERE { :s :p+ ?o }",
        "zk_members": PATH_REACH_MEMBERS,
        "status": STATUS_COVERED,
        "bounded": True,
        "note": "ACCEPTED by the fragment gate; maps to the bounded path_reach family (opt-in `extended-fragment`) proving a chain of 1..k committed `p`-triples, where k is a PUBLIC depth-bound input. NOT the unbounded SPARQL closure: a bounded proof never asserts longer paths are absent nor that the reachable set is complete (design §4, existence-only). Internally re-audited, external sign-off pending (sq-qhy4).",
    },
    {
        "id": "Q18_path_zero_or_more",
        "feature": "Property path * (zero-or-more, reflexive transitive closure)",
        "sparql": "SELECT ?o WHERE { :s :p* ?o }",
        "zk_members": PATH_REACH_MEMBERS,
        "status": STATUS_COVERED,
        "bounded": True,
        "note": "ACCEPTED by the fragment gate; maps to the bounded path_reach family (opt-in `extended-fragment`) as Q17 plus the zero-length case (0..k steps; the zero-length path requires an occurrence witness, not bare equality). BOUNDED existence-only, k is a public input (design §4); NOT the unbounded reflexive-transitive closure. Internally re-audited, external sign-off pending (sq-qhy4).",
    },
    {
        "id": "Q19_aggregate_group_by",
        "feature": "Aggregate / GROUP BY (COUNT, SUM, AVG, MIN, MAX)",
        "sparql": "SELECT ?g (COUNT(?x) AS ?n) WHERE { ?x :inGroup ?g } GROUP BY ?g",
        "zk_members": [],
        "status": STATUS_GAP,
        "note": "Rejected fail-closed by the fragment gate — OUT by design: an aggregate value is a completeness claim over the whole pattern (closed-world), and composed-pattern completeness is not proved. The COUNT-forgery guard in scan.nr is anti-forgery over disclosed rows, NOT a proven aggregate.",
    },
    {
        "id": "Q20_subquery",
        "feature": "Subquery (nested SELECT)",
        "sparql": "SELECT ?p WHERE { ?p :a ?x { SELECT ?p WHERE { ?p :b ?y } } }",
        "zk_members": [],
        "status": STATUS_PARTIAL,
        "note": "ACCEPTED by the fragment gate (inner projection = existential quantification, non-projected variables renamed apart); composed sub-proofs, the outer/inner join verifier-side (no dedicated member).",
    },
    {
        "id": "Q21_bind_expression",
        "feature": "BIND (expression evaluation)",
        "sparql": "SELECT ?c WHERE { ?s :a ?a . ?s :b ?b BIND(?a + ?b AS ?c) }",
        "zk_members": [],
        "status": STATUS_GAP,
        "note": "Rejected fail-closed by the fragment gate — BIND (deterministic Extend) is phase 3 of the fragment-extension program; no in-circuit expression evaluation (arithmetic / function eval over bound vars) yet.",
    },
    {
        "id": "Q22_values",
        "feature": "VALUES (inline data)",
        "sparql": "SELECT ?x WHERE { VALUES ?x { :a :b } ?x :p ?o }",
        "zk_members": [],
        "status": STATUS_PARTIAL,
        "note": "ACCEPTED by the fragment gate; rows are public constants of the query text (UNDEF cells are wildcards), checked verifier-side against disclosed solutions — no dedicated member.",
    },
    {
        "id": "Q23_negation_minus_notexists",
        "feature": "Negation (FILTER NOT EXISTS / MINUS)",
        "sparql": "SELECT ?s WHERE { ?s :p ?o FILTER NOT EXISTS { ?s :q ?v } }",
        "zk_members": [],
        "status": STATUS_GAP,
        "note": "Rejected fail-closed by the fragment gate — OUT by design: FILTER NOT EXISTS / MINUS is closed-world negation (non-monotone), requiring proven ABSENCE. scan completeness (scan.nr) is the closest primitive but is not a negation proof.",
    },
    {
        "id": "Q24_revocation_status",
        "feature": "Revocation / status (hidden status-list index)",
        "sparql": "ASK { ?cred :revocationStatus ?bit FILTER(?bit = 0) }",
        "zk_members": ["revoke_unset_d10"],
        "status": STATUS_COVERED,
        "note": "Hidden status-list index proving the credential's status bit is unset.",
    },
    {
        "id": "Q25_issuer_attestation_hidden",
        "feature": "Issuer attestation (hidden tier, Schnorr)",
        "sparql": "ASK { ?cred :issuer ?iss FILTER(?iss IN (:tierA, :tierB)) }",
        "zk_members": ["hidden_issuer_d4"],
        "status": STATUS_COVERED,
        "reduction_target": "scalar-mul projective coords (research/zk-age-gatecount-reduction.md §3.3; bead sq-hb75)",
        "note": "Hidden-issuer Schnorr verification; the scalar-mul is the dominant cost (reduction target).",
    },
    {
        "id": "Q26_holder_possession_hidden",
        "feature": "Holder possession (hidden binding)",
        "sparql": "ASK { ?cred :holder ?h FILTER(?h = :me) }",
        "zk_members": ["holder_pok", "holder_set_d4"],
        "status": STATUS_COVERED,
        "note": "Holder proof-of-possession (holder_pok) and set-membership variant (holder_set_d4).",
    },
]


def load_snapshot() -> dict:
    with open(SNAPSHOT_JSON, encoding="utf-8") as fh:
        return json.load(fh)


def build_catalog(snapshot: dict) -> dict:
    members: dict[str, int] = snapshot["members"]

    # Fail loudly on a typo in CATALOG rather than silently emitting a gap: every
    # named member MUST exist in the regression-gated snapshot (the anti-drift
    # contract, §6.3). This is the generator-side half; the Rust test re-checks the
    # committed JSON.
    for entry in CATALOG:
        for m in entry.get("zk_members", []):
            if m not in members:
                raise SystemExit(
                    f"catalog error: query {entry['id']} names member '{m}' "
                    f"absent from {SNAPSHOT_JSON} — fix the member name or re-baseline the snapshot"
                )
        fm = entry.get("floor_member")
        if fm is not None and fm not in members:
            raise SystemExit(
                f"catalog error: query {entry['id']} floor_member '{fm}' absent from snapshot"
            )

    queries: dict[str, dict] = {}
    covered = partial = gap = 0
    high_gate: list[str] = []

    for entry in CATALOG:
        member_names = entry.get("zk_members", [])
        rep = entry.get("representative_members", member_names)

        # Join circuit_size from the snapshot — NEVER from CATALOG. Gaps and
        # verifier-side-only entries (no member) get a null circuit_size and an
        # empty per-member map. A real number is impossible to fabricate here:
        # it can only come from the snapshot's regression-gated value.
        per_member = {m: members[m] for m in member_names}
        rep_sizes = [members[m] for m in rep]
        circuit_size = max(rep_sizes) if rep_sizes else None
        circuit_size_min = min(rep_sizes) if rep_sizes else None

        status = entry["status"]
        if status == STATUS_COVERED:
            covered += 1
        elif status == STATUS_PARTIAL:
            partial += 1
        else:
            gap += 1

        # Flag high-gate covered members as reduction targets (§6.3). The flag is
        # derived from the JOINED snapshot value, never asserted. Two distinct
        # high-gate causes, kept apart so the value-hook target is not conflated
        # with a merely-large lattice corner:
        #   * HIGH_GATE_blake3_binding — the numeric-FILTER family (filter_*) whose
        #     17,416 cost is the blake3 token-binding the value-hook removes (§2.6).
        #     Identified by the presence of the value-hook reduction_target, NOT by
        #     size alone.
        #   * HIGH_GATE_lattice — a scan/join member whose JOINED size exceeds the
        #     threshold because of the (k,n,r)/(na,nb) lattice corner, not blake3.
        #     This is honest about WHY it is large (it is not the value-hook target).
        flag = None
        if status == STATUS_GAP:
            flag = "NO_ZK_CIRCUIT_YET"
        elif status == STATUS_PARTIAL:
            flag = "VERIFIER_SIDE_OR_DESUGARED"
        elif entry.get("reduction_target") == VALUE_HOOK_TARGET:
            # The blake3-bound numeric-FILTER family: the value-hook target.
            flag = "HIGH_GATE_blake3_binding"
            high_gate.append(entry["id"])
        elif circuit_size is not None and circuit_size >= HIGH_GATE_THRESHOLD:
            flag = "HIGH_GATE_lattice"
            high_gate.append(entry["id"])

        out: dict = {
            "feature": entry["feature"],
            "sparql": entry["sparql"],
            "zk_members": member_names,
            "circuit_size": circuit_size,
            "status": status,
            "flag": flag,
        }
        # Honesty flag for the bounded property-path closures (p?/p+/p*): they are
        # covered ONLY under the EXISTENCE-ONLY bounded statement (§4), never the
        # unbounded SPARQL closure. Surfaced so a consumer can never read a bounded
        # path proof as an unbounded-reachability claim.
        if entry.get("bounded"):
            out["bounded"] = True
        # Report the per-member sizes and the covered range so a site surface can
        # render either the headline (max) or the spread.
        if per_member:
            out["circuit_size_per_member"] = per_member
            if circuit_size_min != circuit_size:
                out["circuit_size_range"] = [circuit_size_min, circuit_size]
        if "floor_member" in entry:
            out["floor_member"] = entry["floor_member"]
            out["floor_circuit_size"] = members[entry["floor_member"]]
        if "reduction_target" in entry:
            out["reduction_target"] = entry["reduction_target"]
        if "projected_after" in entry:
            out["projected_after"] = entry["projected_after"]
        if "note" in entry:
            out["note"] = entry["note"]

        queries[entry["id"]] = out

    return {
        "_comment": (
            "[OPUS-4.8] sq-1s2.1.2 comprehensive SPARQL ZK gate-cost catalog. A "
            "COVERAGE MAP (which circuit member each SPARQL 1.1 feature compiles to, "
            "or null for a gap) AND an OPTIMISATION-TARGET LIST (per-member bb-gates "
            "circuit_size, with the blake3-bound filter_* family flagged HIGH_GATE_* "
            "as the value-hook reduction target). circuit_size is JOINED from "
            "crates/sparq-zk-compose/tests/gate_count_snapshot.json (the "
            "regression-gated source of truth) so it can never drift; gaps carry "
            "null and are NEVER given a fabricated number. STATUS derives from the "
            "verifier fragment gate (crates/sparq-zk/src/verify.rs::fragment_query): "
            "accepted-with-a-dedicated-member = covered, accepted-but-composed-"
            "verifier-side = partial, rejected-fail-closed = gap — NOT whether a "
            "desugared form merely compiles (sq-3kd2g.11 fixes the §1 Q13/Q14 drift). "
            "Regenerate with bench/zk-compose/scripts/sparql_catalog.py; gated by "
            "crates/sparq-zk-compose/tests/sparql_catalog.rs. Design: "
            "research/zk-field-native-encoding.md §6 (PR #765) + "
            "research/zksparql-fragment-extension.md §§1,3,4 (epic sq-3kd2g / #1591)."
        ),
        "tool": snapshot.get("tool", "bb gates -s ultra_honk"),
        "bb_version": snapshot.get("bb_version_baselined"),
        "nargo_version": snapshot.get("nargo_version_baselined"),
        "snapshot_source": "crates/sparq-zk-compose/tests/gate_count_snapshot.json",
        "high_gate_threshold": HIGH_GATE_THRESHOLD,
        "summary": {
            "total_queries": len(CATALOG),
            "covered": covered,
            "partial": partial,
            "gaps": gap,
            "high_gate_flagged": high_gate,
        },
        "queries": queries,
    }


def main() -> int:
    snapshot = load_snapshot()
    catalog = build_catalog(snapshot)
    json.dump(catalog, sys.stdout, indent=2)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
