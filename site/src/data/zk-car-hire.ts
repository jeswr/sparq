// [OPUS-4.8] sq-13rg / sq-4r4b — fixture data for the ZK car-hire flagship.
// The RDF credentials are the PRIVATE inputs; the SPARQL eligibility query is the
// public relation; the proof discloses only the boolean verdict.

/** The renter's two W3C Verifiable Credentials, as Turtle. These are the holder's
 *  PRIVATE data — they never leave the device and are never disclosed to the desk;
 *  only commitments to them and the eligibility verdict are. (Shown here purely to
 *  make the demo concrete.) */
export const CREDENTIAL_TURTLE = `@prefix cred: <https://www.w3.org/2018/credentials#> .
@prefix sch:  <https://schema.org/> .
@prefix ex:   <https://example.org/vc/> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

# ── Gov-ID credential (issued by HM Passport Office) ──────────────
ex:govId a cred:VerifiableCredential ;
    cred:issuer    <https://gov.uk/issuers/hmpo> ;
    cred:holder    ex:holder ;
    sch:birthDate  "1994-04-02"^^xsd:date ;
    ex:age         "30"^^xsd:integer .          # ← the value proven hidden

# ── Driving-licence credential (issued by the DVLA) ───────────────
ex:licence a cred:VerifiableCredential ;
    cred:issuer        <https://gov.uk/issuers/dvla> ;
    cred:holder        ex:holder ;               # ← same holder as gov-ID
    ex:licenceNumber   "MORGA753116SM9IJ" ;      # ← stays private
    ex:licenceValid    true ;
    ex:statusListIndex "48172"^^xsd:integer .    # ← revocation slot (hidden)`;

/** The eligibility query the car-hire desk asks. It is the PUBLIC relation: the
 *  desk learns only its boolean answer, proven over the committed credentials. */
export const ELIGIBILITY_QUERY = `# Car-hire desk eligibility — answered with ZERO disclosure of the data.
ASK {
  ?id      ex:age          ?age ;
           cred:holder     ?person .
  ?licence cred:holder     ?person ;          # same holder across both VCs
           ex:licenceValid true .
  FILTER( ?age >= 25 )                         # ← the only age fact revealed: ≥ 25
}`;

export interface DisclosureRow {
  /** What the verifier (the car-hire desk) learns. */
  reveals: string;
  /** What stays private on the holder's device. */
  hides: string;
}

/** The reveal-vs-hide contract, grounded in the real circuit family
 *  (filter_int / join_eq / hidden_issuer / revoke_unset). */
export const DISCLOSURE: DisclosureRow[] = [
  {
    reveals: "Eligible: true — the holder may hire a car",
    hides: "The exact date of birth and age (only “≥ 25 is true” leaks)",
  },
  {
    reveals: "The predicates asked (age ≥ 25; a valid licence; one shared holder)",
    hides: "The licence number and every other undisclosed credential field",
  },
  {
    reveals: "Commitments C(G) to both credential graphs + a freshness nonce",
    hides: "The holder’s identity (the join value hides behind a commitment)",
  },
  {
    reveals: "That both credentials were signed by issuers in the desk’s trusted set",
    hides: "Which specific issuer key signed (on the hidden-issuer path)",
  },
];

/** The circuit family that composes the full manifest (gate counts are the measured
 *  `bb gates -s ultra_honk` snapshot — non-canonical, for scale only). The age-gate
 *  member (`filter_int`) is the one this page proves LIVE in your tab. */
export interface CircuitRow {
  name: string;
  role: string;
  gates: number;
  live: boolean;
}

export const CIRCUIT_FAMILY: CircuitRow[] = [
  {
    name: "filter_int_d2",
    role: "age ≥ 25 over the hidden integer age — proven live on this page",
    gates: 17416,
    live: true,
  },
  {
    name: "scan_k2_n64_r8",
    role: "BGP scan over each committed credential graph",
    gates: 34821,
    live: false,
  },
  {
    name: "join_eq_na16_nb16",
    role: "both credentials share one holder, without disclosing who",
    gates: 7025,
    live: false,
  },
  {
    name: "hidden_issuer_d4",
    role: "signed by a trusted issuer, without revealing which key",
    gates: 16932,
    live: false,
  },
  {
    name: "revoke_unset_d10",
    role: "the licence is not revoked, at a hidden status-list index",
    gates: 899,
    live: false,
  },
  {
    name: "holder_pok",
    role: "the presenter possesses the bound holder key",
    gates: 10334,
    live: false,
  },
];

/** Human labels for the public-input vector of the age-gate circuit. The age is
 *  NOT in this list — that is the whole point. */
export const PUBLIC_INPUT_LABELS = [
  "challenge (freshness nonce)",
  "operand_enc (committed age-term anchor)",
  "op = 3 (≥)",
  "bound = 25",
  "expected (the verdict)",
];

/** [OPUS-4.8] sq-1s2 — the plain-language "what each circuit-family member proves"
 *  explainer. Mirrors the live-vs-designed table in research/zk-verification-and-costs.md
 *  (the public-inputs / hidden columns are lifted from §"Live vs designed"). The
 *  `proves` / `primitive` text and the gate counts come from the snapshot JSON; this
 *  table adds only the PUBLIC-vs-HIDDEN framing + status, never a fabricated number.
 *
 *  STATUS values are deliberately honest (privacy-claims gate, sq-qhy4):
 *   - "live"     → proven in your browser tab today (only filter_int_d2 / the age-gate).
 *   - "wired"    → compiled + gate-counted + native prove/verify tests, NOT run in-browser.
 *   - "designed" → gadget + binding complete but with a documented open seam / deferral. */
export type FamilyStatus = "live" | "wired" | "designed";

export interface CircuitFamilyExplainer {
  /** The operator-family label shown to a reader. */
  family: string;
  /** A representative compiled member id in the snapshot JSON (binds the gate count). */
  representative: string;
  /** One-line statement of what this family verifies. */
  proves: string;
  /** What the verifier (the car-hire desk) learns — the PUBLIC side. */
  publicInputs: string;
  /** What stays the holder's secret — the HIDDEN / private witness. */
  hidden: string;
  /** The cryptographic primitive underneath. */
  primitive: string;
  /** Honest deployment status — see FamilyStatus. */
  status: FamilyStatus;
  /** A short honest qualifier shown next to the status. */
  statusNote: string;
}

export const CIRCUIT_FAMILY_EXPLAINER: CircuitFamilyExplainer[] = [
  {
    family: "filter",
    representative: "filter_int_d2",
    proves:
      "A hidden value satisfies a predicate — here, that the renter’s age is ≥ 25.",
    publicInputs: "challenge, operand_enc, op, bound, expected (the verdict)",
    hidden: "the value’s exact decimal digits (the age)",
    primitive: "range / comparison over a hidden integer (in-circuit blake3 token binding)",
    status: "live",
    statusNote:
      "the age-gate is the ONLY statement proven live in your browser tab (filter_int_d2); d1/d3/d4 are wired",
  },
  {
    family: "scan",
    representative: "scan_k2_n64_r8",
    proves:
      "Every disclosed row genuinely came from a committed credential graph — and no matching row was hidden (completeness).",
    publicInputs: "challenge, per-graph commitments, the BGP pattern, the disclosed rows, row_count, attribution",
    hidden: "the per-graph row counts and the term encodings",
    primitive: "set-membership + completeness (BGP triple-pattern scan over committed graphs)",
    status: "wired",
    statusNote: "all 8 scan members compile with native prove / verify / tamper tests; not run in-browser",
  },
  {
    family: "join",
    representative: "join_eq_na16_nb16",
    proves:
      "Two credentials share one value at the join slots (one holder across both VCs) — without disclosing who.",
    publicInputs: "challenge, the two graph commitments, a hiding join_commitment, the join slots",
    hidden: "both graphs’ encodings, the two joined rows, the shared join value, and a blinder",
    primitive: "equality-join over committed graphs (single-prover; the joined entity never appears in a public input)",
    status: "wired",
    statusNote: "all 4 join members compile + tested natively; the cross-credential join is not run in-browser",
  },
  {
    family: "issuer",
    representative: "hidden_issuer_d4",
    proves:
      "A trusted issuer signed the credential — without revealing which authority’s key signed it.",
    publicInputs: "challenge, the signed message m, the trusted-issuer key-set root",
    hidden: "the issuer public key, the signature, and which key-set member it is",
    primitive: "in-circuit signature verification + set-membership (Schnorr over Baby-JubJub + hidden-key Merkle membership)",
    status: "designed",
    statusNote: "gadget + binding complete; ADDITIVE only and inherits the open soundness audit — clear-key attestation is still mandatory",
  },
  {
    family: "holder",
    representative: "holder_pok",
    proves:
      "The presenter possesses the holder key the credential was issued to (proof of possession).",
    publicInputs: "challenge, the issuer-attested holder_pk_digest",
    hidden: "the holder secret key and the holder public-key point",
    primitive: "proof-of-possession / proof-of-knowledge (Baby-JubJub key-pair PoK)",
    status: "designed",
    statusNote: "compiled; the in-circuit hidden-key tier is DEFERRED at the manifest seam — the landed binding is the clear-key tier that discloses the holder point",
  },
  {
    family: "revoke",
    representative: "revoke_unset_d10",
    proves:
      "The licence is NOT revoked, at a hidden status-list index (non-revocation).",
    publicInputs: "challenge, the status-list root, the index_commitment",
    hidden: "the status-list index, the status bit (= 0), the blinding, and the Merkle siblings",
    primitive: "non-membership / hidden-index status (Merkle inclusion + bit-unset + index cross-binding)",
    status: "designed",
    statusNote: "compiled; the clear-index path still leaks the index unless the committed-index path is used",
  },
];
