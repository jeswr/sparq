// [OPUS-4.8] sq-vw3ax.14 — pure, framework-free data for the ODRL usage-control
// capability row on /capabilities. `sparq-policy` is the declarative usage-control
// layer above access control: it parses a W3C ODRL 2.2 policy (permission /
// prohibition / duty, with purpose / recipient / dateTime / count constraints) into a
// typed model and evaluates an access request to a fail-closed ALLOW / DENY Decision
// { allow, matched_rules, unmet_constraints }. It is an OPT-IN native crate
// (`publish = false`, dependency-of-nothing in the workspace); `evaluate` is pure Rust
// with no I/O.
//
// =====================================================================================
// HONESTY CONTRACT — read before editing (same discipline as src/lib/federation.ts).
//
//   * TIER — captured-output WALKTHROUGH, NOT a live run. The static GitHub-Pages site
//     has no backend and the lean wasm bundles carry zero policy code, so this page does
//     NOT execute the evaluator in your tab. A live in-tab evaluator would need a
//     dedicated sparq-policy wasm bundle (a separate portability spike); until that
//     ships, this is the honest walkthrough tier.
//
//   * WHAT THE CITED TEST PINS vs WHAT THE SITE GATE PINS — every verdict below is the
//     `sparq_policy::evaluate` Decision for that exact (policy, request) pair. The named
//     test cited per variant (`crates/sparq-policy/tests/odrl_eval.rs`) remains a readable
//     provenance handle for the ODRL behaviour. In addition, `site/test/policy.test.mjs`
//     sends this module's exact Turtle and request values through the real native Rust
//     evaluator and compares allow, rule ids, and explanations verbatim. The evaluator
//     is pure and deterministic (policy in, decision out — no network, no randomness),
//     and the suite is ratcheted against the MIT-licensed SolidLab ODRL Test Suite
//     (67/68 through the real `evaluate` path).
//
//   * The rule-IRI ids and the `matched`/`unmet` strings are captured from the
//     evaluator's deterministic output (`crates/sparq-policy/src/eval.rs`): a granting
//     permission's bare rule IRI on ALLOW; the
//     overriding prohibition id (+ "prohibition <id> matches the request") or the
//     `rule <id> constraint (<left> <Op> <right>) unsatisfied` / "permission <id>
//     requires undischarged duty <iri>" caveat on DENY. The crate tests use BLANK-NODE
//     rules (which get generated ids); the site scenarios intentionally use named IRIs
//     so the displayed identifiers are also the evaluator's exact identifiers. Do NOT
//     hand-edit a verdict: the native fixture comparison will reject drift.
//     `site/test/policy.test.mjs` passes every exact Turtle/request pair through a native
//     test harness backed by `sparq_policy::evaluate` and compares the returned Decision
//     verbatim. It also pins the internal consistency (allow ⇔ a matched rule
//     and no unmet caveat) AND that every caveat matches one of eval.rs's documented
//     output shapes AND that every constraint caveat's (rule, left, operator, right)
//     anchors to the scenario's OWN Turtle constraint — eval.rs prints the POLICY's
//     operands verbatim, never the request's supplied value — so a fabricated string or
//     an operand substitution fails the unit gate.
//
//   * SCOPE — single-node only. The federated-disclosure / ODRL→MPC composition (per-node
//     ODRL driving the disclosed-vs-hidden split; a Duty → ZK proof obligation) is
//     DEFERRED and would inherit the MPC honest-majority envelope + the open ZK-soundness
//     remediation. No ZK/MPC guarantee is claimed here. <!-- privacy-claims-allow: NEGATIVE — names the deferred ODRL→MPC/ZK composition as not-yet-built; sq-qhy4 -->
// =====================================================================================

/** The ODRL 2.2 namespace — every action / leftOperand IRI on this page expands from it. */
export const ODRL_NS = "http://www.w3.org/ns/odrl/2/";

/** A single `leftOperand` value the request supplies as evidence for one dimension. */
export interface RequestContext {
  /** Short display label for the dimension (e.g. "dateTime", "purpose", "recipient"). */
  left: string;
  /** The value the request states for it (display form). */
  value: string;
}

/** The evaluator's Decision for one (policy, request) pair — verbatim shapes from
 *  `sparq_policy::Decision` (eval.rs). `matched` is non-empty iff a rule justifies the
 *  verdict; `unmet` carries the fail-closed explanation on a DENY (empty on a clean ALLOW). */
export interface Decision {
  allow: boolean;
  /** Rule id(s) justifying the verdict: the granting permission on ALLOW; the overriding
   *  prohibition on a deny-overrides DENY; empty when no permission matched at all. */
  matched: string[];
  /** Human-readable fail-closed explanation(s) on a DENY; empty on a clean ALLOW. */
  unmet: string[];
}

/** One request the visitor can send at a scenario's policy — with its real decision. */
export interface RequestVariant {
  id: string;
  /** Short chip label (e.g. "In the window", "As Mallory", "No purpose stated"). */
  label: string;
  /** One line on what changes vs the other variants. */
  note: string;
  /** The request as ODRL evaluates it. */
  action: string;
  target?: string;
  party?: string;
  context?: RequestContext[];
  /** Discharged duty action IRIs (display form), if any. */
  discharged?: string[];
  /** The REAL `evaluate` decision (see the honesty contract above). */
  decision: Decision;
  /** The crate test function that asserts this verdict (provenance). */
  test: string;
}

/** One policy + the set of requests you can evaluate against it. */
export interface PolicyScenario {
  id: string;
  title: string;
  /** The usage-control question this policy answers, in one plain line. */
  summary: string;
  /** The ODRL 2.2 policy, in Turtle (the exact text the walkthrough shows + parses-in-spirit). */
  turtle: string;
  /** Which ODRL feature this scenario exercises (a small badge). */
  feature: string;
  variants: RequestVariant[];
}

const alice = "https://alice.ex/me";
const mallory = "https://mallory.ex/me";

/** Verdict provenance points at `crates/sparq-policy/tests/odrl_eval.rs`. */
const T = (fn: string) => `odrl_eval.rs::${fn}`;

export const SCENARIOS: PolicyScenario[] = [
  {
    id: "time-window",
    title: "Time-boxed read",
    summary: "Anyone may read asset-x — but only on or before 2026-12-31. After that the grant lapses; with no time stated the request cannot prove it is in-window, so it fails closed.",
    feature: "odrl:dateTime constraint",
    turtle: `@prefix odrl: <http://www.w3.org/ns/odrl/2/> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

<urn:pol/win> a odrl:Set ; odrl:permission <urn:rule/read-until-2026> .

<urn:rule/read-until-2026>
    odrl:action odrl:read ;
    odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:dateTime ;
                      odrl:operator    odrl:lteq ;
                      odrl:rightOperand "2026-12-31T23:59:59Z"^^xsd:dateTime ] .`,
    variants: [
      {
        id: "in-window",
        label: "In the window",
        note: "Reads at 2026-06-16 — before the upper bound.",
        action: "read",
        target: "urn:asset/x",
        context: [{ left: "dateTime", value: "2026-06-16T09:00:00Z" }],
        decision: { allow: true, matched: ["urn:rule/read-until-2026"], unmet: [] },
        test: T("datetime_window_gates"),
      },
      {
        id: "after-window",
        label: "After the window",
        note: "Reads at 2027-03-01 — the grant has lapsed.",
        action: "read",
        target: "urn:asset/x",
        context: [{ left: "dateTime", value: "2027-03-01T00:00:00Z" }],
        decision: {
          allow: false,
          matched: [],
          unmet: [
            'rule urn:rule/read-until-2026 constraint (http://www.w3.org/ns/odrl/2/dateTime Lteq "2026-12-31T23:59:59Z") unsatisfied',
          ],
        },
        test: T("datetime_window_gates"),
      },
      {
        id: "no-time",
        label: "No time stated",
        note: "Supplies no dateTime evidence — cannot prove it is in-window.",
        action: "read",
        target: "urn:asset/x",
        decision: {
          allow: false,
          matched: [],
          unmet: [
            'rule urn:rule/read-until-2026 constraint (http://www.w3.org/ns/odrl/2/dateTime Lteq "2026-12-31T23:59:59Z") unsatisfied',
          ],
        },
        test: T("datetime_window_gates"),
      },
    ],
  },
  {
    id: "deny-overrides",
    title: "Deny overrides permission",
    summary: "Everyone may read asset-x, EXCEPT Mallory, who is prohibited. A matching prohibition overrides any permission — the fail-closed deny-overrides carve-out.",
    feature: "odrl:prohibition (deny-overrides)",
    turtle: `@prefix odrl: <http://www.w3.org/ns/odrl/2/> .

<urn:pol/po> a odrl:Set ;
    odrl:permission  <urn:rule/anyone-read> ;
    odrl:prohibition <urn:rule/deny-mallory> .

<urn:rule/anyone-read>
    odrl:action odrl:read ; odrl:target <urn:asset/x> .

<urn:rule/deny-mallory>
    odrl:action odrl:read ; odrl:target <urn:asset/x> ;
    odrl:assignee <https://mallory.ex/me> .`,
    variants: [
      {
        id: "as-alice",
        label: "As Alice",
        note: "The permission applies; no prohibition names her.",
        action: "read",
        target: "urn:asset/x",
        party: alice,
        decision: { allow: true, matched: ["urn:rule/anyone-read"], unmet: [] },
        test: T("prohibition_overrides_permission"),
      },
      {
        id: "as-mallory",
        label: "As Mallory",
        note: "The prohibition carves her out — deny wins over the permission.",
        action: "read",
        target: "urn:asset/x",
        party: mallory,
        decision: {
          allow: false,
          matched: ["urn:rule/deny-mallory"],
          unmet: ["prohibition urn:rule/deny-mallory matches the request"],
        },
        test: T("prohibition_overrides_permission"),
      },
    ],
  },
  {
    id: "purpose",
    title: "Purpose-gated use",
    summary: "Alice may use asset-x only for research. A different purpose is denied; stating no purpose at all is unprovable, not a wildcard — fail-closed.",
    feature: "odrl:purpose constraint",
    turtle: `@prefix odrl: <http://www.w3.org/ns/odrl/2/> .

<urn:pol/purp> a odrl:Set ; odrl:permission <urn:rule/use-research> .

<urn:rule/use-research>
    odrl:action odrl:use ;
    odrl:target <urn:asset/x> ;
    odrl:assignee <https://alice.ex/me> ;
    odrl:constraint [ odrl:leftOperand odrl:purpose ;
                      odrl:operator    odrl:eq ;
                      odrl:rightOperand <urn:purpose/research> ] .`,
    variants: [
      {
        id: "research",
        label: "For research",
        note: "The stated purpose matches — and odrl:use subsumes the read/use subtree.",
        action: "use",
        target: "urn:asset/x",
        party: alice,
        context: [{ left: "purpose", value: "urn:purpose/research" }],
        decision: { allow: true, matched: ["urn:rule/use-research"], unmet: [] },
        test: T("purpose_match_grants"),
      },
      {
        id: "marketing",
        label: "For marketing",
        note: "A different purpose — denied.",
        action: "use",
        target: "urn:asset/x",
        party: alice,
        context: [{ left: "purpose", value: "urn:purpose/marketing" }],
        decision: {
          allow: false,
          matched: [],
          unmet: [
            // The evaluator formats the POLICY constraint's right operand (`c.right`,
            // eval.rs), not the request's supplied value — so a purpose-mismatch deny
            // still names the policy's bound <urn:purpose/research>.
            "rule urn:rule/use-research constraint (http://www.w3.org/ns/odrl/2/purpose Eq <urn:purpose/research>) unsatisfied",
          ],
        },
        test: T("purpose_mismatch_denies"),
      },
      {
        id: "no-purpose",
        label: "No purpose stated",
        note: "Missing purpose is unprovable — never treated as 'any purpose allowed'.",
        action: "use",
        target: "urn:asset/x",
        party: alice,
        decision: {
          allow: false,
          matched: [],
          unmet: [
            "rule urn:rule/use-research constraint (http://www.w3.org/ns/odrl/2/purpose Eq <urn:purpose/research>) unsatisfied",
          ],
        },
        test: T("missing_purpose_fails_closed"),
      },
    ],
  },
  {
    id: "recipient",
    title: "Disclose only to a recipient set",
    summary: "asset-x may be distributed only to nodeB or nodeC. A recipient outside that set is denied — the 'disclosing only to recipient R' dimension.",
    feature: "odrl:recipient isPartOf",
    turtle: `@prefix odrl: <http://www.w3.org/ns/odrl/2/> .

<urn:pol/r> a odrl:Set ; odrl:permission <urn:rule/distribute-nodes> .

<urn:rule/distribute-nodes>
    odrl:action odrl:distribute ;
    odrl:target <urn:asset/x> ;
    odrl:constraint [ odrl:leftOperand odrl:recipient ;
                      odrl:operator    odrl:isPartOf ;
                      odrl:rightOperand "nodeB|nodeC" ] .`,
    variants: [
      {
        id: "to-nodeb",
        label: "To nodeB",
        note: "A permitted recipient.",
        action: "distribute",
        target: "urn:asset/x",
        context: [{ left: "recipient", value: "nodeB" }],
        decision: { allow: true, matched: ["urn:rule/distribute-nodes"], unmet: [] },
        test: T("recipient_is_part_of_set"),
      },
      {
        id: "to-noded",
        label: "To nodeD",
        note: "Outside the permitted set — denied.",
        action: "distribute",
        target: "urn:asset/x",
        context: [{ left: "recipient", value: "nodeD" }],
        decision: {
          allow: false,
          matched: [],
          unmet: [
            'rule urn:rule/distribute-nodes constraint (http://www.w3.org/ns/odrl/2/recipient IsPartOf "nodeB|nodeC") unsatisfied',
          ],
        },
        test: T("recipient_is_part_of_set"),
      },
    ],
  },
  {
    id: "duty",
    title: "Read with a duty",
    summary: "Reading asset-x carries a duty: the data must be anonymized. Until that duty is discharged the permission does not grant — obligation, not just permission.",
    feature: "odrl:duty obligation",
    turtle: `@prefix odrl: <http://www.w3.org/ns/odrl/2/> .

<urn:pol/d> a odrl:Set ; odrl:permission <urn:rule/read-if-anonymized> .

<urn:rule/read-if-anonymized>
    odrl:action odrl:read ;
    odrl:target <urn:asset/x> ;
    odrl:duty [ odrl:action odrl:anonymize ] .`,
    variants: [
      {
        id: "not-discharged",
        label: "Duty not discharged",
        note: "No anonymize duty discharged — the permission is blocked.",
        action: "read",
        target: "urn:asset/x",
        decision: {
          allow: false,
          matched: [],
          unmet: [
            "permission urn:rule/read-if-anonymized requires undischarged duty http://www.w3.org/ns/odrl/2/anonymize",
          ],
        },
        test: T("duty_must_be_discharged"),
      },
      {
        id: "discharged",
        label: "Duty discharged",
        note: "The anonymize duty is discharged — the permission grants.",
        action: "read",
        target: "urn:asset/x",
        discharged: ["odrl:anonymize"],
        decision: { allow: true, matched: ["urn:rule/read-if-anonymized"], unmet: [] },
        test: T("duty_must_be_discharged"),
      },
    ],
  },
];

/** Expand a display action / leftOperand / duty token to the full IRI the Rust API
 *  uses: the `odrl:` prefix and bare local names both resolve against {@link ODRL_NS};
 *  an already-absolute IRI (e.g. `urn:…`, `https:…`) is returned unchanged. */
function expandIri(a: string): string {
  if (a.startsWith("odrl:")) return `${ODRL_NS}${a.slice("odrl:".length)}`;
  return a.includes(":") ? a : `${ODRL_NS}${a}`;
}

/** The `sparq_policy::Value` constructor the Rust API wraps a context value in, keyed by
 *  its leftOperand — faithful to the crate tests (dateTime → `Value::DateTime`, purpose →
 *  `Value::Iri`, recipient → `Value::Str`, count → `Value::Num`). */
function valueExpr(left: string, value: string): string {
  switch (left) {
    case "dateTime":
      return `Value::DateTime(${JSON.stringify(value)}.into())`;
    case "purpose":
      return `Value::Iri(${JSON.stringify(value)}.into())`;
    case "count":
      return `Value::Num(${value})`;
    default:
      return `Value::Str(${JSON.stringify(value)}.into())`;
  }
}

/** Render a request variant as the API line `sparq-policy` builds it — a FAITHFUL,
 *  compilable-shape `Request::new(action).on(target).by(party).with(leftIri, Value::…)…`
 *  where every operand is a quoted full IRI and every context value is wrapped in its
 *  `Value` constructor (matching `crates/sparq-policy/tests/odrl_eval.rs`). Display only. */
export function requestLine(v: RequestVariant): string {
  let line = `Request::new("${expandIri(v.action)}")`;
  if (v.target) line += `.on("${v.target}")`;
  if (v.party) line += `.by("${v.party}")`;
  for (const c of v.context ?? []) {
    line += `.with("${expandIri(c.left)}", ${valueExpr(c.left, c.value)})`;
  }
  for (const d of v.discharged ?? []) line += `.discharge("${expandIri(d)}")`;
  return line;
}
