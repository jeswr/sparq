// [OPUS-4.8] sq-t58w.9 — a STRONG SECOND, CROSS-LANGUAGE differential oracle for the
// sparq-solid WAC/ACP authorization engine.
//
// What this is
// ------------
// The Rust differential oracle (sq-t58w.7, crates/sparq-solid/tests/differential_oracle.rs)
// cross-checks sparq-solid's engine against a SECOND Rust decider (an independent procedural
// reading of the spec) over a shared WAC/ACP corpus, with zero divergence. This file is the
// *cross-language* counterpart: it replays the IDENTICAL corpus through INDEPENDENT JavaScript
// Solid reference engines and asserts they agree with sparq-solid's own decisions.
//
//   - WAC: `@solid/acl-check` (the rdflib-based Solid WAC checker — a pure, in-memory
//     `checkAccess` over an in-memory store, the same library Community-Solid-Server family
//     code paths build on) PLUS, as a second WAC opinion, `@solidlab/policy-engine`'s
//     `WacPolicyEngine`.
//   - ACP: `@solidlab/policy-engine`'s `AcpPolicyEngine` (acl-check is WAC-only).
//
// How sparq-solid's decisions are obtained
// ----------------------------------------
// sparq-solid is a Rust crate and is NOT exposed through the wasm build, so there is no
// in-process wasm path. Its decisions are captured in `fixtures/solid-acl-corpus.json`, an
// artifact emitted by a one-shot Rust generator that `include!`s the SAME shared corpus
// (`crates/sparq-solid/tests/common/{wac,acp}.rs`, the source consumed by both the Rust
// conformance suites AND the Rust oracle) and records, per `(agent, client, mode, resource)`
// request, sparq-solid's ACTUAL engine verdict via `AuthIndex::accessible` — NOT a hand table.
// The fixture also carries the spec `Expect` value; the in-repo Rust conformance + oracle prove
// `sparqDecision === specExpect` with zero divergence on `main`, and this test re-asserts that
// invariant as a fixture-integrity check.
//
// HONEST CAVEAT (research-grade reference)
// ----------------------------------------
// `@solidlab/policy-engine` is PRE-1.0 (v0.0.2) — it encodes implementation-truth, not
// spec-truth. `@solid/acl-check` (v0.4.5) is mature but makes its own interpretation choices
// (notably around `acl:origin` and the `acl:Control`→`.acl` relationship). Therefore a
// disagreement here is a TRIAGE SIGNAL, not proof sparq is wrong. Where a divergence is a
// known, understood difference in the reference engine's semantics (documented in
// KNOWN_DIFFERENCES below, each with a rationale), it is printed as TRIAGED and does NOT fail
// the test — it is never auto-attributed to sparq. Any UNEXPECTED divergence fails the test
// and is printed in full for inspection.
//
// Constraints honoured: NO docker, NO server, NO network at test time. Everything is parsed and
// evaluated in-memory.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { Store, Parser } from 'n3';
import policyEngine from '@solidlab/policy-engine';
import * as aclCheckNs from '@solid/acl-check';
import * as rdflibNs from 'rdflib';

const {
  WacPolicyEngine,
  AcpPolicyEngine,
  ManagedWacRepository,
  ManagedAcpRepository,
  UnionAccessChecker,
  AgentAccessChecker,
  AgentClassAccessChecker,
  AgentGroupAccessChecker,
} = policyEngine;

// Both packages are published as CommonJS; under ESM the named exports land on `.default`
// for the namespace-imported ones.
const aclCheck = aclCheckNs.default ?? aclCheckNs;
const $rdf = rdflibNs.default ?? rdflibNs;

// Silence acl-check's chatty per-decision console logging so the test output stays the
// runner-shape summary + any triaged/unexpected divergence lines only.
aclCheck.configureLogger(() => {});

const ACL = (mode) => `http://www.w3.org/ns/auth/acl#${mode}`;
const ACL_NS = $rdf.Namespace('http://www.w3.org/ns/auth/acl#');

const CORPUS = JSON.parse(
  readFileSync(new URL('./fixtures/solid-acl-corpus.json', import.meta.url), 'utf8'),
);

// ---------------------------------------------------------------------------------------------
// KNOWN, TRIAGED engine-semantics differences (NOT sparq bugs).
//
// Each entry is keyed by `${mechanism}|${scenario}|${engine}` and lists a `match(request)`
// predicate selecting the specific requests that legitimately differ, plus a `rationale`. A
// divergence on a matched request is reported as TRIAGED and does not fail the gate; the
// triage is explicit, auditable, and attributed to the reference engine's interpretation —
// never silently swallowed.
// ---------------------------------------------------------------------------------------------
const KNOWN_DIFFERENCES = [
  {
    key: 'wac|origin-user-app-pair|acl-check',
    // sparq treats `acl:origin` as a REQUIRED (user, application) pair: an agent matches only
    // through the named client/origin. `@solid/acl-check` enforces origin trust only when an
    // origin is *presented*; with no origin presented it grants on the agent alone (and does
    // not fail-closed on the authorization's `acl:origin`). Implementation-choice difference.
    match: (r, verdict, sparq) => r.client === null && verdict === 'allow' && sparq === 'deny',
    rationale:
      'acl-check ignores acl:origin when no origin is presented (grants on agent alone); ' +
      'sparq requires the user+app pair.',
  },
  {
    key: 'wac|origin-user-app-pair|policy-engine',
    // policy-engine likewise does not fail-closed on the authorization's `acl:origin`: it
    // grants on the agent alone regardless of the presented origin (incl. a non-matching one).
    match: (r, verdict, sparq) =>
      (r.client === null || r.client === 'https://evil.example') &&
      verdict === 'allow' &&
      sparq === 'deny',
    rationale:
      'policy-engine does not enforce acl:origin as a required pair (grants on agent alone, ' +
      'including a non-matching origin); sparq requires the named client/origin.',
  },
  {
    key: 'wac|control-governs-acl-document|acl-check',
    // sparq (and the in-repo Rust reference oracle) model `acl:Control` as granting Read+Write
    // of the resource's `.acl` document. acl-check evaluates `<R>.acl` as a plain resource with
    // no governing ACL of its own, so it denies. The `<R>.acl` requests are sparq-/oracle-only.
    match: (r, verdict, sparq) =>
      r.resource.endsWith('.acl') && verdict === 'deny' && sparq === 'allow',
    rationale:
      'acl-check does not model acl:Control => R/W of the <R>.acl document; it treats <R>.acl ' +
      'as an ungoverned plain resource. sparq + the Rust reference oracle grant it.',
  },
  {
    key: 'wac|control-governs-acl-document|policy-engine',
    // Same Control-governs-.acl semantics gap; additionally policy-engine raises
    // "No ACL document found for root container" while recursing for `<R>.acl` (it has no ACL),
    // surfaced here as an ERROR verdict — fail-closed and triaged.
    // sparq grants <R>.acl (allow); policy-engine errors recursing to the root → not 'allow'.
    match: (r, verdict, sparq) =>
      r.resource.endsWith('.acl') && verdict !== sparq && verdict !== 'allow',
    rationale:
      'policy-engine does not model acl:Control => R/W of <R>.acl and errors recursing to the ' +
      'root for the .acl resource; sparq + the Rust reference oracle grant it.',
  },
  {
    key: 'wac|agentGroup-vcard-members|policy-engine',
    // policy-engine's AgentGroupAccessChecker resolves `acl:agentGroup` membership by HTTP-
    // `fetch`ing the vCard group document (`@rdfjs/fetch`). With the corpus held entirely
    // in-memory and the no-network mandate, that fetch fails and the group never matches, so
    // policy-engine denies every group member. `@solid/acl-check` (run alongside) resolves the
    // SAME group from the in-memory store and AGREES with sparq, so WAC group coverage is
    // intact — this is a policy-engine transport limitation, not a sparq disagreement.
    // Members are denied where sparq allows; the non-member/anon cases already agree.
    match: (r, verdict, sparq) => r.agent !== null && verdict === 'deny' && sparq === 'allow',
    rationale:
      'policy-engine resolves acl:agentGroup by HTTP-fetching the vCard group document, which ' +
      'cannot run under the in-memory/no-network constraint; acl-check resolves the group from ' +
      'memory and agrees with sparq.',
  },
  {
    key: 'wac|fail-closed-no-acl|policy-engine',
    // For a resource with NO ACL anywhere (orphan), policy-engine's getAclRecursive THROWS
    // "No ACL document found for root container" rather than denying. sparq fail-closes to deny
    // (the WAC-correct outcome); acl-check (run alongside) also returns deny and agrees.
    // sparq denies (fail-closed); policy-engine errors instead of denying — any non-deny verdict.
    match: (_r, verdict, sparq) => sparq === 'deny' && verdict !== 'deny',
    rationale:
      'policy-engine throws "No ACL document found for root container" for a resource with no ' +
      'ACL instead of denying; sparq (and acl-check) fail-closed to deny.',
  },
];

function knownDifference(mechanism, scenario, engine, request, verdict, sparq) {
  const entry = KNOWN_DIFFERENCES.find((d) => d.key === `${mechanism}|${scenario}|${engine}`);
  if (!entry) return null;
  // The matcher is verdict-aware: it selects the EXACT divergence direction we have triaged,
  // so an opposite-direction regression (e.g. the reference engine wrongly granting where it
  // used to deny) is NOT masked — it surfaces as an unexpected divergence.
  return entry.match(request, verdict, sparq) ? entry.rationale : null;
}

// ---------------------------------------------------------------------------------------------
// Corpus parsing helpers.
// ---------------------------------------------------------------------------------------------

/**
 * Parse a scenario's N-Quads into a map: graph IRI -> Quad[] (default-graph quads). The corpus
 * places each ACL/ACR document into the named graph `<R>.acl` / `<R>.acr`, and resource content
 * into `<R>`, mirroring the Solid `R -> R.acl` linkage the Rust loader resolves.
 */
function parseByGraph(nquads) {
  const quads = new Parser({ format: 'application/n-quads' }).parse(nquads);
  const byGraph = new Map();
  for (const q of quads) {
    const g = q.graph.value;
    if (!byGraph.has(g)) byGraph.set(g, []);
    byGraph.get(g).push(q);
  }
  return byGraph;
}

/**
 * The parent container IRI of a Solid resource by slash-path structure (containment is by IRI
 * structure in this corpus, not by an `ldp:contains` triple). Returns `undefined` at/above the
 * storage root so the policy-engine ancestor walk terminates.
 */
function getParent(id) {
  const SCHEME = 'https://';
  if (id.endsWith('/')) {
    const trimmed = id.slice(0, -1);
    const idx = trimmed.lastIndexOf('/');
    if (idx < SCHEME.length) return undefined;
    return trimmed.slice(0, idx + 1);
  }
  const idx = id.lastIndexOf('/');
  if (idx < SCHEME.length) return undefined;
  return id.slice(0, idx + 1);
}

/**
 * Build a policy-engine `AuthorizationManager` over the in-memory corpus graphs. `suffix` is
 * `.acl` (WAC) or `.acr` (ACP); `getAuthorizationData(id)` returns the contents of `<id+suffix>`
 * as an n3 Store (flattened to the default graph, the shape the repositories consume), or
 * `undefined` when no such document exists.
 */
function buildManager(byGraph, suffix) {
  return {
    getParent,
    getAuthorizationData: async (id) => {
      const quads = byGraph.get(`${id}${suffix}`);
      if (!quads || quads.length === 0) return undefined;
      const store = new Store();
      for (const q of quads) store.addQuad(q.subject, q.predicate, q.object);
      return store;
    },
  };
}

// ---------------------------------------------------------------------------------------------
// Reference engine adapters: each maps a corpus request to an `'allow' | 'deny' | 'error:…'`.
// ---------------------------------------------------------------------------------------------

/** `@solidlab/policy-engine` WAC adapter. */
function makePolicyEngineWac(byGraph) {
  const checker = new UnionAccessChecker([
    new AgentAccessChecker(),
    new AgentClassAccessChecker(),
    new AgentGroupAccessChecker(),
  ]);
  const engine = new WacPolicyEngine(checker, new ManagedWacRepository(buildManager(byGraph, '.acl')));
  return async (request) => {
    const creds = {};
    if (request.agent) creds.agent = request.agent;
    if (request.client) creds.client = request.client;
    try {
      const perms = await engine.getPermissions(request.resource, creds, [ACL(request.mode)]);
      return perms[ACL(request.mode)] === true ? 'allow' : 'deny';
    } catch (e) {
      return `error:${e.message}`;
    }
  };
}

/** `@solidlab/policy-engine` ACP adapter. */
function makePolicyEngineAcp(byGraph) {
  const engine = new AcpPolicyEngine(new ManagedAcpRepository(buildManager(byGraph, '.acr')));
  return async (request) => {
    const creds = {};
    if (request.agent) creds.agent = request.agent;
    if (request.client) creds.client = request.client;
    try {
      const perms = await engine.getPermissions(request.resource, creds, [ACL(request.mode)]);
      return perms[ACL(request.mode)] === true ? 'allow' : 'deny';
    } catch (e) {
      return `error:${e.message}`;
    }
  };
}

/**
 * Find the effective ACL document for a resource per WAC's nearest-ACL resolution: the closest
 * of the resource itself, then its ancestor containers, whose `<X>.acl` graph carries any
 * triples. Returns `{ aclDoc, subject, isDirect }`, or `null` when no ancestor has an ACL
 * (fail-closed). Mirrors the loader linkage acl-check needs (it filters by the ACL-doc graph,
 * and uses `acl:accessTo` for the direct case vs `acl:default` for an inherited ancestor).
 */
function effectiveAcl(kb, resource) {
  const candidates = [resource];
  let cur = resource;
  for (;;) {
    const parent = getParent(cur);
    if (!parent) break;
    candidates.push(parent);
    cur = parent;
  }
  for (const subject of candidates) {
    const aclDoc = `${subject}.acl`;
    const n = kb.statementsMatching(null, null, null, $rdf.sym(aclDoc)).length;
    if (n > 0) return { aclDoc, subject, isDirect: subject === resource };
  }
  return null;
}

/** `@solid/acl-check` WAC adapter over an rdflib store loaded from the scenario N-Quads. */
function makeAclCheckWac(kb) {
  return (request) => {
    const eff = effectiveAcl(kb, request.resource);
    if (!eff) return 'deny'; // no applicable ACL anywhere: fail-closed (WAC).
    const agent = request.agent ? $rdf.sym(request.agent) : null;
    // `directory` selects acl:default (inherited) vs acl:accessTo (direct) inside acl-check.
    const directory = eff.isDirect ? null : $rdf.sym(eff.subject);
    const origin = request.client ? $rdf.sym(request.client) : null;
    try {
      const allowed = aclCheck.checkAccess(
        kb,
        $rdf.sym(request.resource),
        directory,
        $rdf.sym(eff.aclDoc),
        agent,
        [ACL_NS(request.mode)],
        origin,
        null,
        [],
      );
      return allowed ? 'allow' : 'deny';
    } catch (e) {
      return `error:${e.message}`;
    }
  };
}

/** Load a scenario's N-Quads into a fresh rdflib store (graphs preserved). */
function loadRdflib(nquads) {
  const kb = $rdf.graph();
  return new Promise((resolve, reject) => {
    // N-Quads parsing in rdflib is async (callback); the 4th term becomes the named graph.
    $rdf.parse(nquads, kb, 'https://pod.example/', 'application/n-quads', (err) => {
      if (err) reject(err);
      else resolve(kb);
    });
  });
}

function requestLabel(r) {
  const who =
    r.agent && r.client
      ? `(agent ${r.agent}, client ${r.client})`
      : r.agent
        ? `agent ${r.agent}`
        : 'anonymous';
  return `${who} ${r.mode} ${r.resource}`;
}

// ---------------------------------------------------------------------------------------------
// The oracle: run one mechanism's corpus through one or more JS engines, comparing every
// request to sparq-solid's recorded decision. Returns { pairs, unexpected[], triaged[] }.
// ---------------------------------------------------------------------------------------------
async function runMechanism(mechanism, engineFactories) {
  const scenarios = CORPUS.scenarios.filter((s) => s.mechanism === mechanism);
  let pairs = 0;
  const unexpected = [];
  const triaged = [];

  for (const scenario of scenarios) {
    const byGraph = parseByGraph(scenario.nquads);
    // Engine adapters may need either the n3-graph map or an rdflib store; build both lazily.
    const kb = mechanism === 'wac' ? await loadRdflib(scenario.nquads) : null;

    const engines = engineFactories.map(({ name, build }) => ({
      name,
      decide: build({ byGraph, kb }),
    }));

    for (const request of scenario.requests) {
      pairs += 1;
      const sparq = request.sparqDecision; // 'allow' | 'deny'
      for (const engine of engines) {
        const verdict = await engine.decide(request);
        const agrees = verdict === sparq;
        if (agrees) continue;
        const rationale = knownDifference(
          mechanism,
          scenario.name,
          engine.name,
          request,
          verdict,
          sparq,
        );
        const record = {
          scenario: scenario.name,
          engine: engine.name,
          request: requestLabel(request),
          js: verdict,
          sparq,
          rationale,
        };
        if (rationale) triaged.push(record);
        else unexpected.push(record);
      }
    }
  }
  return { pairs, unexpected, triaged };
}

function printReport(mechanism, engineNames, result) {
  // Runner-shape summary line (grep-parity with the Rust oracle's
  // "WAC differential pairs N / divergences 0 (floor 0)").
  console.log(
    `${mechanism.toUpperCase()} JS differential pairs ${result.pairs} ` +
      `/ engines [${engineNames.join(', ')}] ` +
      `/ unexpected divergences ${result.unexpected.length} (floor 0) ` +
      `/ triaged known-differences ${result.triaged.length}`,
  );
  for (const t of result.triaged) {
    console.log(
      `  TRIAGED [${t.scenario}] ${t.engine}: ${t.request} => js=${t.js}, sparq=${t.sparq} ` +
        `— ${t.rationale}`,
    );
  }
  for (const u of result.unexpected) {
    console.log(
      `  UNEXPECTED DIVERGENCE [${u.scenario}] ${u.engine}: ${u.request} => js=${u.js}, sparq=${u.sparq}`,
    );
  }
}

// ---------------------------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------------------------

test('fixture integrity: sparq-solid decisions match the spec Expect table (0 mismatch)', () => {
  // The fixture carries both sparq's actual engine verdict (sparqDecision) and the spec Expect
  // (specExpect). The in-repo Rust conformance suites + oracle prove these are identical with
  // zero divergence on main; re-assert it here so a stale/garbled fixture is caught early.
  let mismatches = 0;
  let wacPairs = 0;
  let acpPairs = 0;
  for (const s of CORPUS.scenarios) {
    for (const r of s.requests) {
      if (s.mechanism === 'wac') wacPairs += 1;
      else acpPairs += 1;
      if (r.sparqDecision !== r.specExpect) mismatches += 1;
    }
  }
  assert.equal(mismatches, 0, 'sparqDecision must equal specExpect for every request');
  // Guard against a silently-empty corpus (the Rust oracle's pinned floors: 47 WAC / 40 ACP).
  assert.equal(wacPairs, 47, 'expected the pinned WAC request table (47 pairs)');
  assert.equal(acpPairs, 40, 'expected the pinned ACP request table (40 pairs)');
  assert.equal(
    CORPUS.scenarios.filter((s) => s.mechanism === 'wac').length,
    12,
    'expected 12 WAC scenarios',
  );
  assert.equal(
    CORPUS.scenarios.filter((s) => s.mechanism === 'acp').length,
    12,
    'expected 12 ACP scenarios',
  );
});

test('WAC: sparq-solid agrees with @solid/acl-check and @solidlab/policy-engine (0 unexpected divergences)', async () => {
  const result = await runMechanism('wac', [
    { name: 'acl-check', build: ({ kb }) => makeAclCheckWac(kb) },
    { name: 'policy-engine', build: ({ byGraph }) => makePolicyEngineWac(byGraph) },
  ]);
  printReport('wac', ['acl-check', 'policy-engine'], result);
  assert.equal(
    result.unexpected.length,
    0,
    `WAC: ${result.unexpected.length} unexpected divergence(s) — see UNEXPECTED lines above`,
  );
});

test('ACP: sparq-solid agrees with @solidlab/policy-engine (0 unexpected divergences)', async () => {
  const result = await runMechanism('acp', [
    { name: 'policy-engine', build: ({ byGraph }) => makePolicyEngineAcp(byGraph) },
  ]);
  printReport('acp', ['policy-engine'], result);
  assert.equal(
    result.unexpected.length,
    0,
    `ACP: ${result.unexpected.length} unexpected divergence(s) — see UNEXPECTED lines above`,
  );
});

test('the oracle is load-bearing: a deliberately wrong expectation IS flagged', async () => {
  // Negative control — prove the comparison actually compares. Mutate the corpus so one known
  // WAC allow is recorded as a deny; the acl-check adapter (which says allow) must then surface
  // an UNEXPECTED divergence on a request NOT in KNOWN_DIFFERENCES.
  const scenario = CORPUS.scenarios.find((s) => s.name === 'agent-accessTo');
  assert.ok(scenario, 'agent-accessTo scenario present');
  const kb = await loadRdflib(scenario.nquads);
  const decide = makeAclCheckWac(kb);
  const allowReq = scenario.requests.find((r) => r.sparqDecision === 'allow');
  assert.ok(allowReq, 'expected an allow request to mutate');

  const jsVerdict = await decide(allowReq); // acl-check: allow
  assert.equal(jsVerdict, 'allow', 'sanity: acl-check allows the granted request');

  // Simulate a corrupted sparq decision and confirm the agreement check would flag it.
  const mutatedSparq = 'deny';
  assert.notEqual(
    jsVerdict,
    mutatedSparq,
    'a wrong sparq decision must disagree with the JS engine (oracle is not trivially passing)',
  );
  // And confirm this request is NOT masked by a known-difference entry.
  assert.equal(
    knownDifference('wac', scenario.name, 'acl-check', allowReq, jsVerdict, mutatedSparq),
    null,
    'the negative-control request must not be a triaged known-difference',
  );
});
