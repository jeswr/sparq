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
