#!/usr/bin/env python3
"""Divergence classifier + agreement report + fail-closed gate for the ODRE lane.

Bead sq-i6du2.8 (epic sq-i6du2, issue #1613). Consumes the sparq decisions in
`cases.json` (from `exporter/`) and the ODRE decisions in `odre-decisions.json` (from
`odre_adapter.py`), and produces `agreement-report.json` — validated against
`agreement-report.schema.json` before it is written.

The invariant this file enforces, verbatim from the bead:

    honest divergence ledger — agreement claimed ONLY from run output; every divergence
    classified (mapping-gap / semantics-gap / implementation-bug); NO unqualified
    correctness claim for either system.

Concretely:

* An agreement RATE is emitted only when ODRE actually ran and the comparable set is
  non-empty. Otherwise it is `null` and `status` says why. There is no default, no
  carry-forward, and no "assumed agreement".
* Every disagreement must match an entry in `known-divergences.json`. One that does not
  is reported under `unclassified` and the process exits 3.
* A run is scored only against the PINNED ODRE release, over the WHOLE corpus. A
  different installed version, or a decision file that does not cover every corpus case
  under every declared encoding exactly once, is an input error (exit 2) — not a batch of
  skips inside an otherwise `complete` report.
* Cases using a construct ODRE's own capability table declares unsupported are counted as
  OUT-OF-SCOPE-FOR-SYSTEM and excluded from the comparable set — reported, never dropped.
* No statement is made about either system being right. A divergence class explains WHERE
  the difference comes from, not who is correct.

stdlib-only. Exit codes:
  0  report written; complete or honestly skipped, with every divergence classified
  2  usage / input error — including an unpinned ODRE version and an incomplete decision
     file (see `input_errors`)
  3  UNCLASSIFIED DIVERGENCE — the gate the bead's acceptance test names
  5  the produced report is not schema-complete (a harness bug; never published)
"""

import argparse
import json
import os
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
SCHEMA_PATH = os.path.join(HERE, "agreement-report.schema.json")
LEDGER_PATH = os.path.join(HERE, "known-divergences.json")
CAPABILITIES_PATH = os.path.join(HERE, "odre-capabilities.json")

EXIT_OK = 0
EXIT_USAGE = 2
EXIT_UNCLASSIFIED = 3
EXIT_INCOMPLETE = 5

# The statuses a per-encoding result may carry once ODRE has actually run. Each records a
# real outcome of handing the case to the pinned release. `missing` and `skipped-no-odre`
# are ABSENCES, not outcomes: for a `present` run `input_errors` refuses them, so key
# coverage cannot masquerade as verdict coverage (see item 4 there).
REAL_RUN_STATUSES = ("ok", "odre-error", "encoding-error", "encoding-unrepresentable")
# Of those, the ones whose `error` text the classifier and the out-of-scope reason read.
ERROR_STATUSES = ("odre-error", "encoding-error", "encoding-unrepresentable")
DECISION_VALUES = ("allow", "deny")

HARNESS_INTERVENTIONS = [
    {
        "id": "clock-pinning",
        "what": "ODRE's odrl:dateTime left-operand function is bound to the corpus's pinned evaluation instant instead of datetime.now().",
        "why": "pyodre resolves odrl:dateTime to the wall clock, so an unpinned run would decide every temporal constraint by the calendar date of the run and the two systems would not be answering the same question. sparq is evaluated at the same pinned instant. Applied through ODRE's own interpreter extension point.",
        "applies_to": ["standard", "odre-native", "projected"],
    },
    {
        "id": "prefix-registration",
        "what": "The sparq acl action namespace is registered in ODRE's interpreter prefix map.",
        "why": "pyodre's interpreter raises `Unknown URI` for any term outside its prefix map, which would turn every case into an error rather than a decision. Registration uses the documented `add_prefix_mapping` extension point and adds no behaviour: no acl_* function exists, so ODRE reports the action as unsupported-but-permitted.",
        "applies_to": ["standard", "odre-native", "projected"],
    },
    {
        "id": "constraint-inlining",
        "what": "Constraint properties are copied from the odrl:constraint node onto the rule node.",
        "why": "pyodre's constraint query reads odrl:leftOperand/operator/rightOperand directly from the rule node, so an ODRL 2.2 constraint node is not reached and every rule would fire unconstrained. Reported as a divergence in the `standard` encoding AND normalised away in `odre-native`, so the study shows both.",
        "applies_to": ["odre-native", "projected"],
    },
    {
        "id": "tz-naive-datetime",
        "what": "xsd:dateTime right operands are rewritten from `...Z` to the timezone-naive lexical form.",
        "why": "pyodre casts right operands with `%Y-%m-%d %H:%M:%S`, which raises on an offset-aware literal. The instant is unchanged; only the lexical form is.",
        "applies_to": ["odre-native", "projected"],
    },
    {
        "id": "request-binding",
        "what": "Rules whose odrl:assignee does not bind the requesting party (directly or through supplied collection membership), or which do not carry the requested action, are removed before the policy reaches ODRE.",
        "why": "ODRE.enforce takes no request, so without this step it answers 'does any rule in this policy hold' rather than 'may THIS party do THIS'. Performing the binding in the harness is the charitable comparison; it is a harness step, not an ODRE capability, and the `standard` / `odre-native` encodings show what happens without it.",
        "applies_to": ["projected"],
    },
    {
        "id": "target-binding-not-performed",
        "what": "No per-rule odrl:target filtering is applied.",
        "why": "The acbench ODRL compiler attaches odrl:target at the policy node rather than per rule, and each case's policy is already scoped to the requested resource by construction. Declared as a scope limit so the absence is not read as an equivalence.",
        "applies_to": ["standard", "odre-native", "projected"],
    },
]

CLAIMS_RUN = [
    "Agreement figures in this report are computed from the run recorded here and from nothing else.",
    "PASS means: on this corpus, at this pinned instant, under the declared harness interventions, the two systems returned the same allow/deny for every comparable case, or every difference matched a pre-registered, sourced ledger entry.",
    "It does NOT mean either system is correct. No correctness, conformance, soundness or security property of sparq or of ODRE is claimed or tested here.",
    "Cases marked out-of-scope-for-ODRE are excluded from the comparable set because ODRE's own capability table declares the construct unsupported — that is a scope statement about the comparison, not a defect finding about ODRE.",
    "A ledger entry whose evidence is `source-read` is a PREDICTION derived from reading the pinned implementation's source; it is confirmed only once a run observes a divergence matching it.",
]

CLAIMS_SKIPPED = [
    "NO agreement figure is stated: the pinned ODRE implementation was not importable, so no ODRE decision exists to compare against.",
    "This report is complete in SHAPE only — every corpus case is present with a SKIPPED verdict and sparq's own decision recorded.",
    "Nothing here says anything about agreement, correctness or conformance of either system.",
]


# --------------------------------------------------------------- schema validation


def validate(instance, schema, path="$", errors=None):
    """Validate against the subset of JSON Schema this report's schema uses.

    Supported keywords: type, const, enum, required, properties, items. Unknown keywords
    are ignored. Deliberately small and dependency-free — its job is to make the schema
    file load-bearing (an incomplete report must fail the lane), not to be a general
    validator.
    """
    if errors is None:
        errors = []

    if "const" in schema and instance != schema["const"]:
        errors.append("%s: expected const %r, got %r" % (path, schema["const"], instance))
    if "enum" in schema and instance not in schema["enum"]:
        errors.append("%s: %r not in %r" % (path, instance, schema["enum"]))
    if "type" in schema:
        types = schema["type"]
        if isinstance(types, str):
            types = [types]
        if not any(_is_type(instance, t) for t in types):
            errors.append("%s: expected type %r, got %s" % (path, types, type(instance).__name__))
            return errors
    for key in schema.get("required", []):
        if not isinstance(instance, dict) or key not in instance:
            errors.append("%s: missing required key %r" % (path, key))
    if isinstance(instance, dict):
        for key, subschema in schema.get("properties", {}).items():
            if key in instance:
                validate(instance[key], subschema, "%s.%s" % (path, key), errors)
    if isinstance(instance, list) and "items" in schema:
        for i, item in enumerate(instance):
            validate(item, schema["items"], "%s[%d]" % (path, i), errors)
    return errors


def _is_type(instance, t):
    if t == "object":
        return isinstance(instance, dict)
    if t == "array":
        return isinstance(instance, list)
    if t == "string":
        return isinstance(instance, str)
    if t == "integer":
        return isinstance(instance, int) and not isinstance(instance, bool)
    if t == "number":
        return isinstance(instance, (int, float)) and not isinstance(instance, bool)
    if t == "boolean":
        return isinstance(instance, bool)
    if t == "null":
        return instance is None
    return True


# --------------------------------------------------------------- input gate


def input_errors(corpus, decisions, ledger, capabilities):
    """Everything that must hold BEFORE an agreement figure may be computed at all.

    These are input errors, not results: each one means the run is not the run the report
    would claim it is, so the lane refuses rather than publishing a `complete` report with
    a footnote.

    1. **The ledger and the capability table pin the same release.** Classifying against a
       mismatched pair explains divergences with rules transcribed from a different
       implementation.
    2. **The ODRE that ran IS that pinned release.** `odre_adapter.py` runs whatever
       `pyodre` `ODRE_PYTHON` can import, and records `unknown` if even the version is
       unreadable — but the classifier then excuses divergences using source-read entries
       attributed to the pinned version specifically, and the report names that version as
       the system under comparison. Scoring anything else would attribute to a release
       behaviour nobody read out of it.
    3. **The decision file covers the corpus exactly**: every case once, no duplicate id,
       no id the corpus does not contain, and every declared encoding present for each.
       Without this a truncated or corrupt `odre-decisions.json` degrades silently — the
       absent results become `skipped` verdicts, drop out of the comparable set, and the
       surviving cases still produce a `complete` report with a confident agreement rate
       over whatever fraction of the corpus happened to survive.
    4. **Every result IS a result.** Item 3 only proves the encoding KEYS are all there,
       and key coverage is not verdict coverage: under a `present` ODRE a result object
       reading `{"status": "missing"}` or `{"status": "skipped-no-odre"}` carries the key
       but no verdict, and `build_report` would count it `skipped`, drop it from the
       comparable set, and still publish `complete` with a rate over the remainder — the
       exact silent-degradation item 3 exists to prevent, one level down. So for a
       `present` run every case/encoding result must carry a real-run status
       (`REAL_RUN_STATUSES`) with the fields that status implies: an `ok` needs an
       allow/deny decision, an error-shaped status needs the error text the classifier and
       the out-of-scope reason are built from. For an ABSENT run the mirror holds — every
       result must be `skipped-no-odre`, since nothing can have been decided.

    Returns a list of human-readable errors; empty means the inputs are usable.
    """
    errors = []

    if ledger["pinned_odre_version"] != capabilities["pinned_version"]:
        errors.append(
            "the divergence ledger pins ODRE %s but the capability table pins %s — "
            "refusing to classify against a mismatched pair."
            % (ledger["pinned_odre_version"], capabilities["pinned_version"])
        )

    odre_meta = decisions["odre"]
    pinned = ledger["pinned_odre_version"]
    if odre_meta["status"] == "present" and odre_meta.get("version") != pinned:
        errors.append(
            "ODRE reports version %r, but this lane's capability table and divergence "
            "ledger are transcribed from %s and its divergence excuses are sourced from "
            "%s's source. Install the pinned release (bench/ac/odre/run.sh --setup), or "
            "re-pin deliberately: bump requirements.txt, odre-capabilities.json and "
            "known-divergences.json together, re-transcribed against the new release."
            % (odre_meta.get("version"), pinned, pinned)
        )

    corpus_ids = [c["id"] for c in corpus["cases"]]
    decided_ids = [c["id"] for c in decisions["cases"]]
    duplicates = sorted({i for i in decided_ids if decided_ids.count(i) > 1})
    if duplicates:
        errors.append(
            "the ODRE decision file records %d case id(s) more than once (%s%s) — one "
            "result would silently shadow the other."
            % (len(duplicates), ", ".join(duplicates[:5]), " …" if len(duplicates) > 5 else "")
        )
    absent = sorted(set(corpus_ids) - set(decided_ids))
    if absent:
        errors.append(
            "%d corpus case(s) have NO ODRE result (%s%s) — the decision file does not "
            "cover the corpus it is scored against."
            % (len(absent), ", ".join(absent[:5]), " …" if len(absent) > 5 else "")
        )
    unexpected = sorted(set(decided_ids) - set(corpus_ids))
    if unexpected:
        errors.append(
            "%d ODRE result(s) name a case the corpus does not contain (%s%s) — the two "
            "files are not from the same run."
            % (len(unexpected), ", ".join(unexpected[:5]), " …" if len(unexpected) > 5 else "")
        )

    expected_encodings = set(decisions["encodings"])
    incomplete = []
    for case in decisions["cases"]:
        got = set(case.get("encodings") or {})
        if got != expected_encodings:
            incomplete.append(
                "%s (missing %s%s)"
                % (
                    case["id"],
                    ", ".join(sorted(expected_encodings - got)) or "-",
                    ""
                    if got <= expected_encodings
                    else "; undeclared %s" % ", ".join(sorted(got - expected_encodings)),
                )
            )
    if incomplete:
        errors.append(
            "%d case(s) do not carry a result for every declared encoding (%s%s)."
            % (len(incomplete), "; ".join(incomplete[:5]), " …" if len(incomplete) > 5 else "")
        )

    present = odre_meta["status"] == "present"
    malformed = []
    for case in decisions["cases"]:
        got = case.get("encodings") or {}
        for enc in sorted(expected_encodings & set(got)):
            result = got[enc]
            where = "%s/%s" % (case["id"], enc)
            if not isinstance(result, dict) or "status" not in result:
                malformed.append("%s: not a result object" % where)
                continue
            status = result["status"]
            if not present:
                if status != "skipped-no-odre":
                    malformed.append(
                        "%s: status %r, but ODRE is %r — nothing can have been decided"
                        % (where, status, odre_meta["status"])
                    )
            elif status not in REAL_RUN_STATUSES:
                malformed.append(
                    "%s: status %r records no ODRE outcome (expected one of %s)"
                    % (where, status, ", ".join(REAL_RUN_STATUSES))
                )
            elif status == "ok" and result.get("decision") not in DECISION_VALUES:
                malformed.append(
                    "%s: status 'ok' but decision is %r, not one of %s"
                    % (where, result.get("decision"), ", ".join(DECISION_VALUES))
                )
            elif status in ERROR_STATUSES and not (result.get("error") or "").strip():
                malformed.append(
                    "%s: status %r carries no error text to classify or report against"
                    % (where, status)
                )
    if malformed:
        errors.append(
            "%d case/encoding result(s) do not record a usable outcome (%s%s) — carrying "
            "the encoding key is not the same as carrying a verdict, and such a result "
            "would be counted as a skip inside an otherwise `complete` report."
            % (len(malformed), "; ".join(malformed[:5]), " …" if len(malformed) > 5 else "")
        )

    return errors


# --------------------------------------------------------------- classification


def odre_scope(constructs, capabilities):
    """`(in_scope, reason)` for a case, from ODRE's DECLARED capability table."""
    unsupported = [
        c
        for c in constructs
        if capabilities["left_operands"].get(c) == "unsupported"
    ]
    if unsupported:
        return False, "ODRE declares %s unsupported" % ", ".join(sorted(unsupported))
    return True, None


def classify(case, encoding, odre_result, ledger):
    """Match a divergence against the ledger; `None` when nothing explains it."""
    sparq_decision = case["sparq"]["decision"]
    odre_decision = odre_result["decision"]
    if odre_result["status"] in ("odre-error", "encoding-error"):
        odre_decision = "error"
    deny_kinds = set(case["sparq"].get("deny_reason_kinds") or [])
    error_text = odre_result.get("error") or ""

    for entry in ledger["entries"]:
        when = entry["when"]
        if "encodings" in when and encoding not in when["encodings"]:
            continue
        if "sparq_decision" in when and when["sparq_decision"] != sparq_decision:
            continue
        if "odre_decision" in when and when["odre_decision"] != odre_decision:
            continue
        if "error_contains" in when and when["error_contains"] not in error_text:
            continue
        if "deny_kinds_any" in when and not (deny_kinds & set(when["deny_kinds_any"])):
            continue
        if "deny_kinds_subset_of" in when:
            # DELIBERATELY not the set-theoretic reading: the empty set is a subset of
            # everything, but a deny we have NO recorded reason for is a deny we cannot
            # attribute to this entry's cause. Treating it as a match would let an
            # unexplained divergence inherit a pre-registered excuse. Empty => no match
            # => unclassified => the run goes red, which is the fail-closed direction.
            # Pinned by self-test scenario 5; do not "fix" this into `issubset` alone.
            if not deny_kinds or not deny_kinds.issubset(set(when["deny_kinds_subset_of"])):
                continue
        if "constructs_any" in when:
            if not set(case["constructs"]) & set(when["constructs_any"]):
                continue
        return entry
    return None


def build_report(corpus, decisions, ledger, capabilities):
    """Produce the agreement report plus the list of unclassified divergences."""
    odre_meta = decisions["odre"]
    encodings = decisions["encodings"]
    headline = decisions["headline_encoding"]
    skipped = odre_meta["status"] != "present"

    by_id = {c["id"]: c for c in decisions["cases"]}

    tallies = {
        enc: {
            "comparable": 0,
            "agree": 0,
            "disagree": 0,
            "out_of_scope_odre": 0,
            "odre_error": 0,
            "skipped": 0,
            "agreement_rate": None,
        }
        for enc in encodings
    }
    # NON-VACUITY: an agreement rate is only as meaningful as what ODRE was given to
    # decide. If it never saw a constraint, or never returned a deny, the comparable set
    # did not exercise its evaluator and the report says so instead of leaving a reader
    # to infer a stronger result than the run supports.
    non_vacuity = {
        enc: {
            "compared": 0,
            "odre_saw_rules": 0,
            "odre_saw_constraints": 0,
            "odre_allow": 0,
            "odre_deny": 0,
        }
        for enc in encodings
    }
    report_cases = []
    divergences = []
    unclassified = []
    construct_cases = {}
    construct_observed = {}

    for case in corpus["cases"]:
        constructs = case["constructs"]
        for c in constructs:
            construct_cases[c] = construct_cases.get(c, 0) + 1
        in_scope, scope_reason = odre_scope(constructs, capabilities)
        odre_case = by_id.get(case["id"], {"encodings": {}})
        verdicts = {}

        for enc in encodings:
            # Both `missing` renderings below are unreachable from the CLI: `input_errors`
            # refuses a decision file that does not cover every case under every encoding
            # AND, for a `present` ODRE, one whose results are not real-run outcomes — so
            # `skipped` here is reachable only on the honest ODRE-absent path, never as an
            # absence hiding inside a `complete` report. Kept as the rendering for a direct
            # build_report call, never as a fallback the gate relies on.
            result = odre_case["encodings"].get(
                enc,
                {"status": "missing", "decision": None, "actions": [], "error": "no ODRE result for this case/encoding"},
            )
            # Per-encoding, because a case can be in ODRE's declared scope yet
            # unrepresentable under one particular encoding.
            enc_scope_reason = scope_reason
            if result["status"] == "skipped-no-odre" or result["status"] == "missing":
                verdict = "skipped"
                tallies[enc]["skipped"] += 1
            elif not in_scope:
                verdict = "out-of-scope-odre"
                tallies[enc]["out_of_scope_odre"] += 1
            elif result["status"] == "encoding-unrepresentable":
                # The policy cannot be handed to ODRE under this encoding without
                # changing its meaning. Out of scope WITH a reason — not a decision, not
                # a divergence, and never silently encoded anyway.
                verdict = "out-of-scope-odre"
                enc_scope_reason = result.get("error")
                tallies[enc]["out_of_scope_odre"] += 1
            elif result["status"] in ("odre-error", "encoding-error"):
                verdict = "odre-error"
                tallies[enc]["odre_error"] += 1
            elif result["decision"] == case["sparq"]["decision"]:
                verdict = "agree"
                tallies[enc]["comparable"] += 1
                tallies[enc]["agree"] += 1
            else:
                verdict = "disagree"
                tallies[enc]["comparable"] += 1
                tallies[enc]["disagree"] += 1

            entry = {
                "verdict": verdict,
                "odre_decision": result.get("decision"),
                "odre_status": result["status"],
                "interventions": result.get("interventions", []),
                "odre_rules": result.get("rules"),
                "odre_constraints": result.get("constraints"),
            }
            if verdict in ("agree", "disagree"):
                nv = non_vacuity[enc]
                nv["compared"] += 1
                if result.get("rules"):
                    nv["odre_saw_rules"] += 1
                if result.get("constraints"):
                    nv["odre_saw_constraints"] += 1
                if result.get("decision") == "allow":
                    nv["odre_allow"] += 1
                else:
                    nv["odre_deny"] += 1
            if verdict == "out-of-scope-odre":
                entry["reason"] = enc_scope_reason
            if verdict in ("disagree", "odre-error"):
                matched = classify(case, enc, result, ledger)
                if matched is None:
                    entry["divergence"] = None
                    unclassified.append(
                        {
                            "case": case["id"],
                            "encoding": enc,
                            "why": "no entry in known-divergences.json matches (sparq=%s, odre=%s, deny_kinds=%s, error=%s)"
                            % (
                                case["sparq"]["decision"],
                                result.get("decision") or result["status"],
                                sorted(case["sparq"].get("deny_reason_kinds") or []),
                                (result.get("error") or "")[:160],
                            ),
                            "sparq_decision": case["sparq"]["decision"],
                            "odre_decision": result.get("decision"),
                        }
                    )
                else:
                    entry["divergence"] = {"class": matched["class"], "ledger_id": matched["id"]}
                    divergences.append(
                        {
                            "case": case["id"],
                            "encoding": enc,
                            "class": matched["class"],
                            "ledger_id": matched["id"],
                            "evidence": matched["evidence"],
                            "sparq_decision": case["sparq"]["decision"],
                            "odre_decision": result.get("decision"),
                        }
                    )
            verdicts[enc] = entry
            for c in constructs:
                obs = construct_observed.setdefault(c, {})
                key = "%s/%s" % (enc, verdict)
                obs[key] = obs.get(key, 0) + 1

        report_cases.append(
            {
                "id": case["id"],
                "use_case": case["use_case"],
                "constructs": constructs,
                "sparq_decision": case["sparq"]["decision"],
                "sparq_deny_reason_kinds": case["sparq"].get("deny_reason_kinds") or [],
                "verdicts": verdicts,
            }
        )

    for enc, t in tallies.items():
        # An agreement RATE exists only when a real ODRE run produced a non-empty
        # comparable set. Never a default, never an assumption.
        if not skipped and t["comparable"] > 0:
            t["agreement_rate"] = round(t["agree"] / t["comparable"], 6)

    # How often each pre-registered ledger entry actually fired. An entry with zero
    # matches stays a PREDICTION; one with matches was confirmed by this run. Recorded
    # here rather than hand-edited into the ledger's `evidence` field.
    ledger_matches = {}
    for entry in ledger["entries"]:
        ledger_matches[entry["id"]] = {
            "class": entry["class"],
            "declared_evidence": entry["evidence"],
            "matched_in_this_run": sum(1 for d in divergences if d["ledger_id"] == entry["id"]),
        }

    coverage = []
    for construct in sorted(construct_cases):
        coverage.append(
            {
                "construct": construct,
                "cases": construct_cases[construct],
                "odre_declared": capabilities["left_operands"].get(
                    construct,
                    capabilities["rule_kinds"].get(construct, "not-declared"),
                ),
                "observed": construct_observed.get(construct, {}),
            }
        )

    report = {
        "schema": "sparq.acbench.odre.agreement/1",
        "lane": "bench/ac/odre",
        "bead": "sq-i6du2.8",
        "status": "skipped-no-odre" if skipped else "complete",
        "systems": {
            "sparq": {
                "path": corpus["sparq_decision_path"],
                "role": "system under test (the ODRL evaluator the sparq-solid odrl-bridge materialises from)",
            },
            "odre": {
                "package": odre_meta["package"],
                "version": odre_meta["version"],
                "status": odre_meta["status"],
                "paper": odre_meta["paper"],
                "reason": odre_meta.get("reason"),
            },
        },
        "corpus": {
            "generator": corpus["producer"],
            "use_cases": sorted({c["use_case"] for c in corpus["cases"]}),
            "case_count": len(corpus["cases"]),
            "eval_instant": corpus["eval_instant"],
            "params": corpus["params"],
        },
        "headline_encoding": headline,
        "harness_interventions": HARNESS_INTERVENTIONS,
        "coverage": coverage,
        "encodings": tallies,
        "non_vacuity": non_vacuity,
        "ledger_matches": ledger_matches,
        "cases": report_cases,
        "divergences": divergences,
        "unclassified": unclassified,
        "claims": CLAIMS_SKIPPED if skipped else CLAIMS_RUN + _evidence_caveats(non_vacuity[headline], headline),
    }
    return report, unclassified


def _evidence_caveats(nv, headline):
    """Caveats the RUN itself forces onto the claim list.

    These are computed, not authored: if the comparable set never handed ODRE a
    constraint, or only ever drew one decision value out of it, the agreement figure is
    weaker than it looks and the report has to say so in its own claims rather than
    leaving it to whoever reads the table.
    """
    out = []
    if not nv["compared"]:
        out.append(
            "No case was comparable under the headline encoding (%s): this run supports no agreement claim at all."
            % headline
        )
        return out
    if nv["odre_saw_constraints"] == 0:
        out.append(
            "WEAK EVIDENCE: not one comparable case handed ODRE a constraint to evaluate, so the headline figure "
            "measures agreement on RULE SELECTION only and says nothing about constraint semantics — the very "
            "dimension the U3/U4 corpora exist to stress."
        )
    if nv["odre_allow"] == 0 or nv["odre_deny"] == 0:
        out.append(
            "WEAK EVIDENCE: ODRE returned a single decision value across every comparable case, so agreement here "
            "cannot distinguish 'the two systems concur' from 'one system is constant'."
        )
    return out


def summarize(report, stream=sys.stderr):
    """One-screen human summary. Prints figures ONLY when the run produced them."""
    p = lambda s: print(s, file=stream)  # noqa: E731
    p("")
    p("=== ODRE decision-agreement lane (sq-i6du2.8) ===")
    p("status         : %s" % report["status"])
    p(
        "odre           : %s %s (%s)"
        % (
            report["systems"]["odre"]["package"],
            report["systems"]["odre"]["version"] or "-",
            report["systems"]["odre"]["status"],
        )
    )
    p(
        "corpus         : %d cases from %s at %s"
        % (
            report["corpus"]["case_count"],
            ", ".join(report["corpus"]["use_cases"]),
            report["corpus"]["eval_instant"],
        )
    )
    if report["status"] == "skipped-no-odre":
        p("SKIPPED        : %s" % (report["systems"]["odre"]["reason"] or "pyodre not importable"))
        p("                 no agreement figure is stated (see `claims`)")
        return
    p("")
    p("encoding      comparable  agree  disagree  out-of-scope  odre-error  agreement")
    for enc, t in report["encodings"].items():
        rate = "-" if t["agreement_rate"] is None else "%.3f" % t["agreement_rate"]
        marker = " (headline)" if enc == report["headline_encoding"] else ""
        p(
            "%-13s %10d %6d %9d %13d %11d %10s%s"
            % (
                enc,
                t["comparable"],
                t["agree"],
                t["disagree"],
                t["out_of_scope_odre"],
                t["odre_error"],
                rate,
                marker,
            )
        )
    p("")
    nv = report["non_vacuity"][report["headline_encoding"]]
    p(
        "non-vacuity (%s): %d compared; ODRE saw rules in %d, constraints in %d; decisions %d allow / %d deny"
        % (
            report["headline_encoding"],
            nv["compared"],
            nv["odre_saw_rules"],
            nv["odre_saw_constraints"],
            nv["odre_allow"],
            nv["odre_deny"],
        )
    )
    if nv["compared"] and nv["odre_saw_constraints"] == 0:
        p("  WEAK: no compared case gave ODRE a constraint to evaluate — the rate reflects rule selection only.")
    if nv["compared"] and (nv["odre_allow"] == 0 or nv["odre_deny"] == 0):
        p("  WEAK: ODRE returned only one decision value across the comparable set.")
    p("")
    classes = {}
    for d in report["divergences"]:
        classes[d["class"]] = classes.get(d["class"], 0) + 1
    if classes:
        p("divergences by class: %s" % ", ".join("%s=%d" % kv for kv in sorted(classes.items())))
    else:
        p("divergences: none")
    if report["unclassified"]:
        p("UNCLASSIFIED : %d — the run FAILS (see `unclassified` in the report)" % len(report["unclassified"]))


# --------------------------------------------------------------- self-test


def _st_lane_inputs(ledger):
    """A minimal but COMPLETE (corpus, decisions) pair: two cases, two encodings, agreeing.

    The baseline the input-gate scenarios below mutate. Deliberately a PASSING pair, so
    each mutation isolates exactly one defect.
    """
    ids = ["st-0001", "st-0002"]
    encodings = ["standard", "projected"]
    corpus = {
        "producer": "self-test",
        "sparq_decision_path": "self-test",
        "eval_instant": "2026-07-06T00:00:00Z",
        "params": {},
        "cases": [
            {
                "id": cid,
                "use_case": "self-test",
                "constructs": ["rule:permission"],
                "sparq": {"decision": "allow", "deny_reason_kinds": []},
            }
            for cid in ids
        ],
    }
    decisions = {
        "odre": {
            "package": "pyodre",
            "version": ledger["pinned_odre_version"],
            "status": "present",
            "paper": "arXiv:2409.17602",
            "reason": None,
        },
        "encodings": encodings,
        "headline_encoding": "projected",
        "cases": [
            {
                "id": cid,
                "encodings": {
                    enc: {
                        "status": "ok",
                        "decision": "allow",
                        "actions": [],
                        "interventions": [],
                        "error": None,
                    }
                    for enc in encodings
                },
            }
            for cid in ids
        ],
    }
    return corpus, decisions


def _st_run_lane(ledger, mutate):
    """Drive the REAL `main` over a temp input pair; `(exit_code, report_was_written)`.

    End-to-end on purpose: what has to be proven is that the LANE refuses, not merely
    that a predicate returned a string.
    """
    corpus, decisions = _st_lane_inputs(ledger)
    mutate(corpus, decisions)
    with tempfile.TemporaryDirectory() as tmp:
        cases_path = os.path.join(tmp, "cases.json")
        odre_path = os.path.join(tmp, "odre-decisions.json")
        out_path = os.path.join(tmp, "agreement-report.json")
        for path, obj in ((cases_path, corpus), (odre_path, decisions)):
            with open(path, "w", encoding="utf-8") as f:
                json.dump(obj, f)
        code = main(["--cases", cases_path, "--odre", odre_path, "--out", out_path])
        return code, os.path.exists(out_path)


def self_test():
    """Prove the gate is non-vacuous: an unexplained divergence must FAIL.

    The anti-vacuity discipline the workload engine uses for its by-construction oracle
    (design record §2.3: 'the oracle's oracle'). Synthetic scenarios are driven through
    the real classifier and the real gate:

      1. agreement            -> no divergence, gate passes
      2. explained divergence -> classified from the ledger, gate passes
      3. unexplained one      -> unclassified, gate FAILS with exit 3
      4. ODRE absent          -> schema-complete report, and NO agreement rate
      5. unattributable deny  -> must not inherit a ledger entry's excuse
      6. input gate           -> a truncated, duplicated or unpinned-version decision
                                 file — or one whose encoding keys are all present but
                                 whose results record no verdict — must fail the LANE
                                 (exit 2, nothing published) rather than degrade into
                                 skips in a `complete` report

    If scenario 3 or 6 ever passes, this lane's headline invariant is broken and the
    self-test fails loudly instead of the lane quietly reporting green.
    """
    ledger = load_json(LEDGER_PATH)
    capabilities = load_json(CAPABILITIES_PATH)
    schema = load_json(SCHEMA_PATH)
    failures = []

    def scenario(name, sparq_decision, deny_kinds, odre_decision):
        corpus = {
            "producer": "self-test",
            "sparq_decision_path": "self-test",
            "eval_instant": "2026-07-06T00:00:00Z",
            "params": {},
            "cases": [
                {
                    "id": "st-0000",
                    "use_case": "self-test",
                    "constructs": ["rule:permission", "scope:resource"],
                    "sparq": {"decision": sparq_decision, "deny_reason_kinds": deny_kinds},
                }
            ],
        }
        decisions = {
            "odre": {
                "package": "pyodre",
                "version": "self-test",
                "status": "present",
                "paper": "arXiv:2409.17602",
                "reason": None,
            },
            "encodings": ["standard"],
            "headline_encoding": "standard",
            "cases": [
                {
                    "id": "st-0000",
                    "encodings": {
                        "standard": {
                            "status": "ok",
                            "decision": odre_decision,
                            "actions": [],
                            "interventions": [],
                            "error": None,
                        }
                    },
                }
            ],
        }
        report, unclassified = build_report(corpus, decisions, ledger, capabilities)
        schema_errors = validate(report, schema)
        return report, unclassified, schema_errors

    # 1. agreement
    report, unclassified, schema_errors = scenario("agree", "allow", [], "allow")
    if unclassified:
        failures.append("scenario 1 (agreement) produced unclassified divergences")
    if schema_errors:
        failures.append("scenario 1 report is not schema-complete: %s" % schema_errors[:3])
    if report["encodings"]["standard"]["agreement_rate"] != 1.0:
        failures.append("scenario 1 agreement rate is not 1.0")

    # 2. an explained divergence: sparq denies on assignee-mismatch, ODRE allows —
    #    the `odre-no-request-binding` mapping gap.
    report, unclassified, schema_errors = scenario(
        "explained", "deny", ["assignee-mismatch"], "allow"
    )
    if unclassified:
        failures.append("scenario 2 (explained divergence) was NOT classified: %s" % unclassified)
    if not report["divergences"] or report["divergences"][0]["class"] != "mapping-gap":
        failures.append("scenario 2 was not classified as mapping-gap: %s" % report["divergences"])

    # 3. an unexplained divergence: sparq ALLOWS, ODRE denies, no ledger entry covers it.
    report, unclassified, schema_errors = scenario("unexplained", "allow", [], "deny")
    if not unclassified:
        failures.append(
            "GATE IS VACUOUS: an unexplained sparq-allow/ODRE-deny divergence was not "
            "reported as unclassified"
        )
    if report["divergences"]:
        failures.append("scenario 3 was wrongly attributed to a ledger entry")

    # 5. a deny with NO recorded reason kind must NOT inherit a `deny_kinds_subset_of`
    #    entry's excuse, even though the empty set is technically a subset of anything.
    #    An unattributable deny is an unexplained one.
    report, unclassified, schema_errors = scenario("unattributable", "deny", [], "allow")
    if not unclassified:
        failures.append(
            "a deny with no recorded reason kind was matched to a deny_kinds_subset_of "
            "ledger entry — an unattributable divergence must stay unclassified"
        )

    # 4. the report is still schema-complete when ODRE is absent (the SKIP path must not
    #    be able to publish a malformed report).
    corpus = {
        "producer": "self-test",
        "sparq_decision_path": "self-test",
        "eval_instant": "2026-07-06T00:00:00Z",
        "params": {},
        "cases": [
            {
                "id": "st-0001",
                "use_case": "self-test",
                "constructs": ["rule:permission"],
                "sparq": {"decision": "deny", "deny_reason_kinds": ["no-permission"]},
            }
        ],
    }
    decisions = {
        "odre": {
            "package": "pyodre",
            "version": None,
            "status": "absent",
            "paper": "arXiv:2409.17602",
            "reason": "self-test",
        },
        "encodings": ["standard"],
        "headline_encoding": "standard",
        "cases": [
            {
                "id": "st-0001",
                "encodings": {
                    "standard": {
                        "status": "skipped-no-odre",
                        "decision": None,
                        "actions": [],
                        "interventions": [],
                        "error": None,
                    }
                },
            }
        ],
    }
    report, unclassified = build_report(corpus, decisions, ledger, capabilities)
    schema_errors = validate(report, load_json(SCHEMA_PATH))
    if schema_errors:
        failures.append("scenario 4 (ODRE absent) report is not schema-complete: %s" % schema_errors[:3])
    if report["encodings"]["standard"]["agreement_rate"] is not None:
        failures.append("scenario 4 stated an agreement rate with no ODRE run — honesty invariant broken")

    # 6. the INPUT gate: an ODRE run that did not cover the corpus, or that was not the
    #    pinned release, must stop the lane before any figure is computed. The intact pair
    #    is the positive control — without it a gate that rejected everything would look
    #    identical to one that rejects the right things.
    def drop_case(_corpus, decisions):
        del decisions["cases"][1]

    def drop_encoding(_corpus, decisions):
        del decisions["cases"][0]["encodings"]["projected"]

    def duplicate_case(_corpus, decisions):
        decisions["cases"].append(dict(decisions["cases"][0]))

    def unexpected_case(_corpus, decisions):
        extra = dict(decisions["cases"][0], id="st-9999")
        decisions["cases"].append(extra)

    def wrong_version(_corpus, decisions):
        decisions["odre"]["version"] = "0.0.0-not-the-pinned-release"

    def unknown_version(_corpus, decisions):
        decisions["odre"]["version"] = "unknown"

    # …and the same file with every encoding key PRESENT, but a result object behind one
    # of them that records no verdict. Key coverage is not verdict coverage: each of these
    # used to sail through the gate and be tallied as a `skipped` inside a `complete`
    # report, publishing an agreement rate over whatever fraction still had a real result.
    def _absence(status):
        return {
            "status": status,
            "decision": None,
            "actions": [],
            "interventions": [],
            "error": None,
        }

    def missing_result_object(_corpus, decisions):
        decisions["cases"][0]["encodings"]["projected"] = _absence("missing")

    def skipped_result_in_present_run(_corpus, decisions):
        decisions["cases"][1]["encodings"]["standard"] = _absence("skipped-no-odre")

    def unknown_status(_corpus, decisions):
        decisions["cases"][0]["encodings"]["standard"]["status"] = "not-a-status"

    def null_ok_decision(_corpus, decisions):
        decisions["cases"][0]["encodings"]["standard"]["decision"] = None

    def errorless_odre_error(_corpus, decisions):
        decisions["cases"][0]["encodings"]["standard"].update(
            {"status": "odre-error", "decision": None, "error": None}
        )

    def odre_absent(_corpus, decisions):
        """The HONEST skip: ODRE never ran, so every result is `skipped-no-odre`."""
        decisions["odre"].update({"status": "absent", "version": None, "reason": "self-test"})
        for case in decisions["cases"]:
            for enc in case["encodings"]:
                case["encodings"][enc] = _absence("skipped-no-odre")

    def odre_absent_but_decided(_corpus, decisions):
        odre_absent(_corpus, decisions)
        decisions["cases"][0]["encodings"]["standard"] = {
            "status": "ok",
            "decision": "allow",
            "actions": [],
            "interventions": [],
            "error": None,
        }

    for name, mutate, expected in (
        ("intact inputs", lambda c, d: None, EXIT_OK),
        ("a corpus case with no ODRE result", drop_case, EXIT_USAGE),
        ("a missing encoding result", drop_encoding, EXIT_USAGE),
        ("a duplicated case id", duplicate_case, EXIT_USAGE),
        ("a case id the corpus does not contain", unexpected_case, EXIT_USAGE),
        ("an ODRE version that is not the pin", wrong_version, EXIT_USAGE),
        ("an unreadable ODRE version", unknown_version, EXIT_USAGE),
        ("a `missing` result inside a present run", missing_result_object, EXIT_USAGE),
        ("a `skipped-no-odre` result inside a present run", skipped_result_in_present_run, EXIT_USAGE),
        ("a result carrying an unknown status", unknown_status, EXIT_USAGE),
        ("an `ok` result with no decision", null_ok_decision, EXIT_USAGE),
        ("an `odre-error` result with no error text", errorless_odre_error, EXIT_USAGE),
        ("an honest ODRE-absent run", odre_absent, EXIT_OK),
        ("an ODRE-absent run that decided a case anyway", odre_absent_but_decided, EXIT_USAGE),
    ):
        code, written = _st_run_lane(ledger, mutate)
        if code != expected:
            failures.append(
                "scenario 6 (%s): the lane exited %d, expected %d" % (name, code, expected)
            )
        if expected != EXIT_OK and written:
            failures.append(
                "scenario 6 (%s): the lane published a report anyway — an incomplete or "
                "unpinned run must produce no report at all" % name
            )

    if failures:
        print("[agreement] SELF-TEST FAILED:", file=sys.stderr)
        for f in failures:
            print("  - %s" % f, file=sys.stderr)
        return False
    print(
        "[agreement] self-test OK: gate fails on an unexplained divergence, classifies an "
        "explained one, and states no rate without a run.",
        file=sys.stderr,
    )
    return True


# --------------------------------------------------------------- main


def load_json(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--cases", help="cases.json from the exporter")
    ap.add_argument("--odre", help="odre-decisions.json from the adapter")
    ap.add_argument("--out", help="where to write agreement-report.json")
    ap.add_argument(
        "--self-test",
        action="store_true",
        help="run the anti-vacuity self-test (no corpus needed) and exit",
    )
    args = ap.parse_args(argv)

    if args.self_test:
        return EXIT_OK if self_test() else EXIT_UNCLASSIFIED

    if not (args.cases and args.odre and args.out):
        ap.error("--cases, --odre and --out are required unless --self-test is given")
    for path in (args.cases, args.odre):
        if not os.path.exists(path):
            print("[agreement] ERROR: no such file: %s" % path, file=sys.stderr)
            return EXIT_USAGE

    corpus = load_json(args.cases)
    decisions = load_json(args.odre)
    ledger = load_json(LEDGER_PATH)
    capabilities = load_json(CAPABILITIES_PATH)
    schema = load_json(SCHEMA_PATH)

    bad_inputs = input_errors(corpus, decisions, ledger, capabilities)
    if bad_inputs:
        print("[agreement] ERROR: the inputs cannot be scored:", file=sys.stderr)
        for e in bad_inputs:
            print("  - %s" % e, file=sys.stderr)
        print(
            "[agreement] no report written: an agreement figure over these inputs would "
            "not be the figure it claims to be.",
            file=sys.stderr,
        )
        return EXIT_USAGE

    report, unclassified = build_report(corpus, decisions, ledger, capabilities)

    errors = validate(report, schema)
    if errors:
        print("[agreement] ERROR: the produced report is NOT schema-complete:", file=sys.stderr)
        for e in errors[:20]:
            print("  - %s" % e, file=sys.stderr)
        return EXIT_INCOMPLETE

    with open(args.out, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
        f.write("\n")
    print("[agreement] wrote %s" % args.out, file=sys.stderr)
    summarize(report)

    if unclassified:
        print(
            "\n[agreement] FAIL: %d UNCLASSIFIED divergence(s). Every difference between the "
            "two systems must be explained by a sourced entry in known-divergences.json, or "
            "this lane does not get to report agreement." % len(unclassified),
            file=sys.stderr,
        )
        for u in unclassified[:10]:
            print("  - %s [%s]: %s" % (u["case"], u["encoding"], u["why"]), file=sys.stderr)
        return EXIT_UNCLASSIFIED

    return EXIT_OK


if __name__ == "__main__":
    sys.exit(main())
