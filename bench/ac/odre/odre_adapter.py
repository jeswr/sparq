#!/usr/bin/env python3
"""ODRE side of the ODRL decision-agreement lane (bead sq-i6du2.8, epic sq-i6du2).

Reads the corpus `cases.json` produced by `exporter/` and runs each case through the
pinned ODRE reference implementation (`pyodre`, the Python implementation of
arXiv:2409.17602), emitting `odre-decisions.json`. It NEVER decides agreement — that is
`agreement.py`'s job, over this file plus the sparq decisions in `cases.json`.

ODRE is a GATHER-TIME dependency (bench/CATALOG.md convention): never vendored, never
committed. If `pyodre` is not importable the adapter emits a complete, well-formed
report with `status: "absent"` and exits 0 — a SKIP with a reason, never a fabricated
decision, and `agreement.py` then refuses to state any agreement figure.

Three input ENCODINGS are run per case, because the encoding materially changes what
ODRE evaluates and reporting only one would hide that (MAPPING.md §2):

  standard    the ODRL 2.2 graph exactly as sparq consumes it.
  odre-native standard + the two input normalisations pyodre's evaluator requires:
              constraints inlined onto the rule node, and timezone-naive xsd:dateTime
              lexical forms.
  projected   odre-native + harness-side REQUEST BINDING: rules are pre-filtered to
              those whose assignee/action match the request. This is the charitable
              encoding — it hands ODRE the request-matching step its API has no way to
              express — and is the one `agreement.py` treats as headline.

Every deviation from "feed both systems the identical bytes" is recorded per case in
`interventions`, so the report can never quietly launder a harness choice into a result.

stdlib-only apart from `pyodre` itself (and its own rdflib/jinja2/requests deps).
Exit codes: 0 = decisions written (including the honest ODRE-absent skip); 2 = usage or
input error.
"""

import argparse
import json
import os
import sys

# The three encodings, in report order. `projected` is the headline (see module docstring).
ENCODINGS = ("standard", "odre-native", "projected")

ODRL_NS = "http://www.w3.org/ns/odrl/2/"
XSD_DATETIME = "http://www.w3.org/2001/XMLSchema#dateTime"
RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"

CONSTRAINT_P = ODRL_NS + "constraint"
LEFT_P = ODRL_NS + "leftOperand"
OPERATOR_P = ODRL_NS + "operator"
RIGHT_P = ODRL_NS + "rightOperand"
ACTION_P = ODRL_NS + "action"
ASSIGNEE_P = ODRL_NS + "assignee"
RULE_PREDICATES = (ODRL_NS + "permission", ODRL_NS + "prohibition", ODRL_NS + "obligation")

# The sparq access-mode action vocabulary the acbench ODRL compiler emits. pyodre's
# interpreter REFUSES (raises) any URI outside its prefix map, so the harness registers
# this namespace through pyodre's own documented extension point (`add_prefix_mapping`).
# No `acl_Read` python function exists, so pyodre reports the action as unsupported-but-
# permitted, which is exactly the honest reading of "ODRE let the rule fire".
SPARQ_ACL_NS = "https://sparq.dev/vocab/acl#"


# --------------------------------------------------------------------- N-Triples


def parse_nt(text):
    """Parse the exporter's N-Triples into `(subject, predicate, object_token)` triples.

    Deliberately minimal: the exporter emits one canonical `<s> <p> <o> .` form per line
    (objects are IRIs or typed literals, no blank nodes, no escapes beyond the literal
    quoting rdflib itself round-trips). Anything else raises rather than being skipped —
    a silently dropped triple would change a decision.
    """
    triples = []
    for lineno, line in enumerate(text.splitlines(), 1):
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        if not line.endswith("."):
            raise ValueError("line %d: N-Triples line does not end with '.'" % lineno)
        body = line[:-1].strip()
        subj, rest = _take_term(body, lineno)
        pred, rest = _take_term(rest, lineno)
        obj = rest.strip()
        if not obj:
            raise ValueError("line %d: missing object" % lineno)
        triples.append((subj, pred, obj))
    return triples


def _take_term(s, lineno):
    """Split the leading `<iri>` term off `s`, returning `(iri, remainder)`."""
    s = s.strip()
    if not s.startswith("<"):
        raise ValueError("line %d: expected an IRI term, got %r" % (lineno, s[:40]))
    end = s.index(">")
    return s[1:end], s[end + 1 :]


def serialize_nt(triples):
    out = []
    for s, p, o in triples:
        out.append("<%s> <%s> %s ." % (s, p, o))
    return "\n".join(out) + "\n"


def iri_of(token):
    """The IRI inside an `<iri>` object token, or None if the token is a literal."""
    if token.startswith("<") and token.endswith(">"):
        return token[1:-1]
    return None


# --------------------------------------------------------------------- encodings


def encode_standard(policy_nt):
    """The ODRL 2.2 graph verbatim — the bytes sparq itself consumes."""
    return policy_nt, []


class Unrepresentable(Exception):
    """The case cannot be handed to ODRE without changing what the policy MEANS.

    Raised instead of encoding a policy whose meaning the harness would have to distort.
    The case is then reported out-of-scope-for-ODRE with this reason attached — the
    honest outcome, where silently encoding it would manufacture a decision (and, in the
    multi-constraint case below, a systematically wrong one).
    """


def encode_odre_native(policy_nt):
    """Apply the two input normalisations pyodre 1.0.6's evaluator requires.

    1. **Constraint inlining.** `ODRE._constraints` selects `?rule odrl:operator ?op ;
       odrl:leftOperand ?l ; odrl:rightOperand ?r` — i.e. constraint properties must sit
       DIRECTLY on the rule node. ODRL 2.2 links them through `odrl:constraint` to a
       separate node, which that query does not reach. Without this normalisation ODRE
       sees zero constraints and every rule fires unconditionally, so leaving it out
       would compare sparq against a vacuous evaluation.
    2. **Timezone-naive dateTime literals.** pyodre casts a right operand with
       `cast_dateTime`, whose format is `%Y-%m-%d %H:%M:%S` — a `Z`-suffixed
       (offset-aware) xsd:dateTime raises on cast. The instant is unchanged; only the
       lexical form is.

    Both are recorded as interventions on every case they touch.

    REFUSAL: a rule bearing more than ONE constraint is `Unrepresentable`. ODRE's rule-node
    encoding cannot express a conjunction of distinct constraints, because
    `ODRE._constraints` selects `?operator`, `?left_operand` and `?right_operand`
    independently from the rule node — a cartesian product, not a per-constraint join. A
    two-bound temporal window (`dateTime >= start` AND `dateTime < end`) inlines to the
    four-way product including `dateTime >= end` and `dateTime < start`, which is
    unsatisfiable for every instant. Encoding it anyway would hand ODRE a policy that
    denies by construction and then score that denial as a decision. Measured on
    pyodre 1.0.6, not inferred.
    """
    triples = parse_nt(policy_nt)
    interventions = []

    # 1. inline constraints
    by_subject = {}
    for s, p, o in triples:
        by_subject.setdefault(s, []).append((p, o))
    inlined = []
    constraint_nodes = set()
    per_rule_constraints = {}
    for s, p, o in triples:
        if p == CONSTRAINT_P:
            node = iri_of(o)
            if node is None:
                raise ValueError("odrl:constraint object is not an IRI: %s" % o)
            constraint_nodes.add(node)
            per_rule_constraints[s] = per_rule_constraints.get(s, 0) + 1
            for cp, co in by_subject.get(node, []):
                if cp in (LEFT_P, OPERATOR_P, RIGHT_P):
                    inlined.append((s, cp, co))
    overloaded = sorted(r for r, n in per_rule_constraints.items() if n > 1)
    if overloaded:
        raise Unrepresentable(
            "rule(s) %s carry %d constraints; ODRE's rule-node constraint encoding takes "
            "the cartesian product of operator x leftOperand x rightOperand and cannot "
            "express a conjunction of distinct constraints"
            % (", ".join(overloaded[:2]), max(per_rule_constraints.values()))
        )
    if inlined:
        interventions.append("constraint-inlining")
    out = [t for t in triples if t[0] not in constraint_nodes and t[1] != CONSTRAINT_P]
    out.extend(inlined)

    # 2. timezone-naive dateTime lexical form
    naive = []
    changed = False
    for s, p, o in out:
        if o.endswith('^^<%s>' % XSD_DATETIME) and 'Z"^^' in o:
            o = o.replace('Z"^^', '"^^')
            changed = True
        naive.append((s, p, o))
    if changed:
        interventions.append("tz-naive-datetime")

    return serialize_nt(naive), interventions


def encode_projected(policy_nt, request):
    """`odre-native` + harness-side request binding.

    pyodre's `enforce(policy)` has no request parameter: it evaluates every rule in the
    policy and returns the actions of those whose constraints hold, ignoring
    `odrl:assignee`, `odrl:target` and the requested action entirely. The charitable
    comparison therefore performs the request-matching step OUTSIDE ODRE and hands it
    only the rules that bind to this request:

    - a rule whose `odrl:assignee` is neither the requesting party nor a collection the
      party belongs to (membership evidence supplied by the corpus, `party_collections`)
      is dropped;
    - a rule that does not carry the requested action is dropped;
    - a rule that carries EXTRA actions keeps only the requested one, because
      `ODRE._action` takes an arbitrary first row from an unordered result set and would
      otherwise report a different action than the one under test.

    Target binding is NOT performed: the acbench ODRL compiler attaches `odrl:target` at
    the POLICY node, not per rule, and every rule in a case's policy is already scoped to
    the requested resource by construction (the exporter selects the case's intents by
    resource). Recorded as a declared scope limit, not silently assumed.

    Binding runs BEFORE the `odre-native` normalisation, so a case whose only
    multi-constraint (hence `Unrepresentable`) rule does not bind to this request stays
    comparable rather than being discarded for a rule that never applied.
    """
    triples = parse_nt(policy_nt)
    party = request["party"]
    collections = set(request.get("party_collections") or [])
    action = request["action"]

    rules = set()
    for _s, p, o in triples:
        if p in RULE_PREDICATES:
            rid = iri_of(o)
            if rid is not None:
                rules.add(rid)

    assignees = {}
    actions = {}
    for s, p, o in triples:
        if s in rules and p == ASSIGNEE_P:
            assignees.setdefault(s, set()).add(iri_of(o))
        if s in rules and p == ACTION_P:
            actions.setdefault(s, set()).add(iri_of(o))

    kept = set()
    for rid in rules:
        rule_assignees = assignees.get(rid)
        if rule_assignees is not None:
            if not (party in rule_assignees or rule_assignees & collections):
                continue
        rule_actions = actions.get(rid)
        if rule_actions is not None and action not in rule_actions:
            continue
        kept.add(rid)

    dropped = len(rules) - len(kept)

    # Constraint nodes reachable ONLY from a dropped rule must go with it. Left behind,
    # they are inert to ODRE (nothing links them to a rule) but they would still be
    # counted as "constraints ODRE was given", overstating the non-vacuity evidence.
    all_constraint_nodes = {iri_of(o) for _s, p, o in triples if p == CONSTRAINT_P}
    kept_constraint_nodes = {
        iri_of(o) for s, p, o in triples if p == CONSTRAINT_P and s in kept
    }
    orphaned = all_constraint_nodes - kept_constraint_nodes

    out = []
    for s, p, o in triples:
        if p in RULE_PREDICATES and iri_of(o) not in kept:
            continue
        if s in rules and s not in kept:
            continue
        if s in orphaned:
            continue
        if s in kept and p == ACTION_P and iri_of(o) != action:
            continue
        out.append((s, p, o))

    # Constraint nodes belonging to dropped rules are now unreferenced; the native
    # encoder walks odrl:constraint links, so they simply fall away with their rules.
    native, interventions = encode_odre_native(serialize_nt(out))
    interventions = list(interventions)
    interventions.append("request-binding")
    if dropped:
        interventions.append("rules-dropped:%d" % dropped)
    return native, interventions


# --------------------------------------------------------------------- ODRE driver


def load_odre(pinned_instant):
    """Import pyodre and build a clock-pinned, acl-namespace-aware ODRE driver.

    Returns `(enforce_fn, meta)` or `(None, meta)` when pyodre is not importable.

    Two harness interventions, both through pyodre's own documented extension points:

    - **clock pinning.** `python_functions.odrl_dateTime()` returns `datetime.now()`, so
      an unpinned run would decide temporal constraints against the wall clock and the
      lane's verdict would change by the calendar. The interpreter's `evaluate` is
      overridden to resolve `odrl_dateTime` to the corpus's pinned evaluation instant —
      the same instant sparq is evaluated at. Without this the comparison is not a
      comparison.
    - **prefix registration.** `add_prefix_mapping` teaches the interpreter the sparq acl
      action namespace, which it would otherwise refuse with `Unknown URI`.
    """
    meta = {
        "package": "pyodre",
        "paper": "arXiv:2409.17602",
        "status": "absent",
        "version": None,
        "reason": None,
    }
    try:
        from pyodre.odre import ODRE
        from pyodre.python_interpreter import PythonInterpreter
        import pyodre.python_functions as odre_functions
    except Exception as exc:  # ImportError, or a broken install
        meta["reason"] = "%s: %s" % (type(exc).__name__, exc)
        return None, meta

    try:
        from importlib.metadata import version as _version

        meta["version"] = _version("pyodre")
    except Exception:
        meta["version"] = "unknown"
    meta["status"] = "present"

    pinned = _pinned_datetime(pinned_instant)

    class PinnedClockInterpreter(PythonInterpreter):
        """`PythonInterpreter` with `odrl:dateTime` bound to the pinned instant."""

        def evaluate(self, expression):
            env = dict(vars(odre_functions))
            env["odrl_dateTime"] = lambda: pinned
            return eval(expression, env)  # noqa: S307 — pyodre's own evaluation model

    def enforce(policy_nt):
        interpreter = PinnedClockInterpreter()
        interpreter.add_prefix_mapping("acl_", SPARQ_ACL_NS)
        return ODRE().set_interpreter(interpreter).enforce(policy_nt, format="nt")

    return enforce, meta


def _pinned_datetime(instant):
    """`YYYY-MM-DDTHH:MM:SSZ` -> a timezone-naive UTC `datetime`.

    Naive on purpose: pyodre's `cast_dateTime` produces naive datetimes, and Python
    refuses to order naive against aware ones. Both sides of every comparison are then
    naive UTC.
    """
    from datetime import datetime

    return datetime.strptime(instant, "%Y-%m-%dT%H:%M:%SZ")


def run_case(enforce, case):
    """Run one case through every encoding; never raises."""
    out = {"id": case["id"], "encodings": {}}
    for enc in ENCODINGS:
        try:
            if enc == "standard":
                policy, interventions = encode_standard(case["policy_ntriples"])
            elif enc == "odre-native":
                policy, interventions = encode_odre_native(case["policy_ntriples"])
            else:
                policy, interventions = encode_projected(
                    case["policy_ntriples"], case["request"]
                )
        except Unrepresentable as exc:
            # NOT an error and NOT a decision: the case is out of ODRE's expressible
            # range under this encoding, and says so with its reason.
            out["encodings"][enc] = {
                "status": "encoding-unrepresentable",
                "decision": None,
                "actions": [],
                "rules": 0,
                "constraints": 0,
                "interventions": [],
                "error": str(exc),
            }
            continue
        except Exception as exc:
            out["encodings"][enc] = {
                "status": "encoding-error",
                "decision": None,
                "actions": [],
                "rules": 0,
                "constraints": 0,
                "interventions": [],
                "error": "%s: %s" % (type(exc).__name__, exc),
            }
            continue

        shape = policy_shape(policy)
        try:
            usage_decision = enforce(policy)
            actions = sorted(str(k) for k in usage_decision.keys())
            out["encodings"][enc] = {
                "status": "ok",
                # NON-VACUITY EVIDENCE: how much ODRE was actually given to evaluate. A
                # headline agreement figure over cases where ODRE saw zero constraints
                # (or zero rules) says far less than the same figure over constrained
                # ones, so the shape travels with the decision instead of being inferred.
                "rules": shape["rules"],
                "constraints": shape["constraints"],
                # ODRE's contract: an action key present in the usage decision means the
                # rule bearing it fired (its constraints held). The requested action being
                # present is therefore ODRE's ALLOW for this request.
                "decision": "allow" if case["request"]["action"] in actions else "deny",
                "actions": actions,
                "interventions": interventions,
                "error": None,
            }
        except Exception as exc:
            # A refusal (e.g. pyodre's `Unknown URI` guard) is DATA, not a crash: it is
            # recorded verbatim and classified downstream, never coerced into a decision.
            out["encodings"][enc] = {
                "status": "odre-error",
                "rules": shape["rules"],
                "constraints": shape["constraints"],
                "decision": None,
                "actions": [],
                "interventions": interventions,
                "error": "%s: %s" % (type(exc).__name__, exc),
            }
    return out


def policy_shape(policy_nt):
    """How many rules, and how many constraints, the policy handed to ODRE carries.

    Counted on the ENCODED policy, so it reflects what ODRE was actually given after
    inlining and request binding — not what the original ODRL graph contained.
    """
    try:
        triples = parse_nt(policy_nt)
    except Exception:
        return {"rules": 0, "constraints": 0}
    rules = {iri_of(o) for _s, p, o in triples if p in RULE_PREDICATES}
    rules.discard(None)
    constraints = 0
    for s, p, _o in triples:
        # After `encode_odre_native` a constraint is a leftOperand on the rule node; in
        # the `standard` encoding it is a separate node linked by odrl:constraint.
        if p == LEFT_P:
            constraints += 1
        del s
    return {"rules": len(rules), "constraints": constraints}


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--cases", required=True, help="cases.json from the exporter")
    ap.add_argument("--out", required=True, help="where to write odre-decisions.json")
    args = ap.parse_args(argv)

    if not os.path.exists(args.cases):
        print("[odre] ERROR: no such cases file: %s" % args.cases, file=sys.stderr)
        return 2
    with open(args.cases, encoding="utf-8") as f:
        corpus = json.load(f)

    enforce, meta = load_odre(corpus["eval_instant"])
    report = {
        "schema": "sparq.acbench.odre.decisions/1",
        "odre": meta,
        "encodings": list(ENCODINGS),
        "headline_encoding": "projected",
        "pinned_instant": corpus["eval_instant"],
        "cases": [],
    }

    if enforce is None:
        print(
            "[odre] SKIP: pyodre is not importable (%s).\n"
            "       Install the pinned reference implementation once, with network:\n"
            "         bash bench/ac/odre/run.sh --setup\n"
            "       No decision is fabricated; the agreement report will state SKIPPED."
            % meta["reason"],
            file=sys.stderr,
        )
        for case in corpus["cases"]:
            report["cases"].append(
                {
                    "id": case["id"],
                    "encodings": {
                        enc: {
                            "status": "skipped-no-odre",
                            "decision": None,
                            "actions": [],
                            "interventions": [],
                            "error": None,
                        }
                        for enc in ENCODINGS
                    },
                }
            )
    else:
        print(
            "[odre] running %d cases x %d encodings through pyodre %s"
            % (len(corpus["cases"]), len(ENCODINGS), meta["version"]),
            file=sys.stderr,
        )
        for case in corpus["cases"]:
            report["cases"].append(run_case(enforce, case))

    with open(args.out, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2, sort_keys=False)
        f.write("\n")
    print("[odre] wrote %s" % args.out, file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
