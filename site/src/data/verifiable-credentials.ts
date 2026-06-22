// [OPUS-4.8] sq-3p0z (#822) — fixture data for the Verifiable-Credentials import-and-query
// surface (research/gui-design.md §2b "Verifiable-Credentials workspace").
//
// A W3C Verifiable Credential is an RDF document (a JSON-LD doc that maps to RDF, or already
// Turtle / N-Quads), so "load it into a graph and query it" is JUST the existing in-tab WASM
// ingest path — `live` today. These samples seed the surface so a reader sees it working
// before importing their own; the import surface (drag-drop / paste / URL) loads ANY VC.
//
// HONESTY. These credentials are *imported and queried* — NOT cryptographically verified.
// Recognising / querying a VC's claims is distinct from verifying its proof or producing a
// zero-knowledge proof about it; that is the separate research-grade ZK estate (sparq-zk),
// which is internally re-audited and NOT externally audited (sq-qhy4). The surface copy says
// so, and the privacy-claims gate enforces it.

/** The engine format string each sample / import parses as (the wasm `load`/`loadDataset`). */
export type VcFormat = "jsonld" | "turtle" | "nquads" | "trig" | "ntriples";

export interface SampleCredential {
  /** Stable id + dropdown key. */
  id: string;
  /** Human label shown in the credentials list. */
  label: string;
  /** One-line description of what the credential asserts. */
  blurb: string;
  /** The RDF/JSON-LD source as the holder would receive it. */
  source: string;
  /** The engine format the source parses as. */
  format: VcFormat;
}

// A W3C VC JSON-LD university-degree credential. JSON-LD maps to RDF via the engine's oxjsonld
// parser (the lean wasm bundle ships the `jsonld` feature), so this is queryable as triples
// with no pre-processing. IMPORTANT: the lean in-tab bundle has NO remote-context loader, so
// the `@context` is fully INLINE here (a credential that references the W3C VC context only by
// URL would need that context inlined or be supplied as Turtle/N-Quads — see the page caveat).
// The inline terms expand to the standard cred:/schema: IRIs the example queries match on.
const DEGREE_JSONLD = `{
  "@context": {
    "id": "@id",
    "type": "@type",
    "VerifiableCredential": "https://www.w3.org/2018/credentials#VerifiableCredential",
    "UniversityDegreeCredential": "https://example.org/vc#UniversityDegreeCredential",
    "issuer": { "@id": "https://www.w3.org/2018/credentials#issuer", "@type": "@id" },
    "credentialSubject": { "@id": "https://www.w3.org/2018/credentials#credentialSubject", "@type": "@id" },
    "degree": "https://example.org/vc#degree",
    "name": "https://schema.org/name",
    "alumniOf": "https://schema.org/alumniOf"
  },
  "id": "https://example.org/credentials/3732",
  "type": ["VerifiableCredential", "UniversityDegreeCredential"],
  "issuer": "https://university.example/issuers/565049",
  "credentialSubject": {
    "id": "did:example:ebfeb1f712ebc6f1c276e12ec21",
    "name": "Ada Lovelace",
    "degree": "BSc Computer Science",
    "alumniOf": "Example University"
  }
}`;

// A driving-licence VC as Turtle (the holder may receive a VC already serialised as RDF).
// It shares the same subject DID as the degree credential, so a join across the two
// imported credentials is queryable.
const LICENCE_TURTLE = `@prefix cred: <https://www.w3.org/2018/credentials#> .
@prefix sch:  <https://schema.org/> .
@prefix ex:   <https://example.org/vc#> .
@prefix xsd:  <http://www.w3.org/2001/XMLSchema#> .

<https://example.org/credentials/dl-9981> a cred:VerifiableCredential , ex:DrivingLicenceCredential ;
    cred:issuer    <https://dvla.example/issuers/1> ;
    cred:validFrom "2023-01-01T00:00:00Z"^^xsd:dateTime ;
    cred:credentialSubject [
        a sch:Person ;
        sch:identifier   "did:example:ebfeb1f712ebc6f1c276e12ec21" ;
        ex:licenceClass  "B" ;
        ex:validUntil    "2033-01-01"^^xsd:date ;
        ex:revoked       false
    ] .`;

/** The seeded sample credentials a reader sees before importing their own. */
export const SAMPLE_CREDENTIALS: SampleCredential[] = [
  {
    id: "degree",
    label: "University degree (JSON-LD VC)",
    blurb: "A W3C VC 2.0 university-degree credential, as the holder receives it (JSON-LD).",
    source: DEGREE_JSONLD,
    format: "jsonld",
  },
  {
    id: "licence",
    label: "Driving licence (Turtle VC)",
    blurb: "A driving-licence credential serialised as Turtle, for the same subject DID.",
    source: LICENCE_TURTLE,
    format: "turtle",
  },
];

export interface SampleQuery {
  id: string;
  label: string;
  /** One-line description of what the query asks of the imported credentials. */
  blurb: string;
  sparql: string;
}

/** Starter SPARQL queries over the seeded credentials. The user can edit or replace them. */
export const SAMPLE_QUERIES: SampleQuery[] = [
  {
    id: "list",
    label: "List every imported credential",
    blurb: "Every subject typed as a VerifiableCredential, with its issuer.",
    sparql: `PREFIX cred: <https://www.w3.org/2018/credentials#>
SELECT ?credential ?issuer WHERE {
  ?credential a cred:VerifiableCredential ;
              cred:issuer ?issuer .
}`,
  },
  {
    id: "claims",
    label: "Read the degree credential's claims",
    blurb: "The subject's name, degree and alma mater from the JSON-LD credential.",
    sparql: `PREFIX sch: <https://schema.org/>
PREFIX ex:  <https://example.org/vc#>
SELECT ?name ?degree ?alumniOf WHERE {
  ?subject sch:name ?name ;
           ex:degree ?degree ;
           sch:alumniOf ?alumniOf .
}`,
  },
  {
    id: "join",
    label: "Same holder across two credentials (ASK)",
    blurb: "Do the degree (JSON-LD) and the licence (Turtle) describe the same subject DID?",
    sparql: `PREFIX sch: <https://schema.org/>
ASK {
  # the degree credential's subject DID
  ?degreeSubject sch:name "Ada Lovelace" .
  BIND("did:example:ebfeb1f712ebc6f1c276e12ec21" AS ?did)
  FILTER(STR(?degreeSubject) = ?did)
  # the licence credential's subject, identified by the same DID
  ?licSubject sch:identifier ?did .
}`,
  },
];

// A subset of the in-tab WASM engine's loadable RDF/JSON-LD formats, for the import format
// dropdown. JSON-LD is first because it is the canonical VC serialisation. (Kept here, beside
// the surface that consumes it, rather than re-deriving the REPL's full list.)
export const VC_FORMAT_OPTIONS: { value: VcFormat; label: string; exts: string[] }[] = [
  { value: "jsonld", label: "JSON-LD VC (.jsonld / .json)", exts: ["jsonld", "json-ld", "json"] },
  { value: "turtle", label: "Turtle (.ttl)", exts: ["ttl", "turtle"] },
  { value: "nquads", label: "N-Quads (.nq)", exts: ["nq", "nquads"] },
  { value: "trig", label: "TriG (.trig)", exts: ["trig"] },
  { value: "ntriples", label: "N-Triples (.nt)", exts: ["nt", "ntriples"] },
];

/** Best-effort engine format from a filename, defaulting to JSON-LD (the VC default). */
export function vcFormatFromName(name: string): VcFormat {
  const ext = name.split(/[?#]/)[0].split(".").pop()?.toLowerCase() ?? "";
  const hit = VC_FORMAT_OPTIONS.find((f) => f.exts.includes(ext));
  return hit?.value ?? "jsonld";
}

/**
 * Recognise a JWT-shaped VC envelope (`SD-JWT-VC` / `JWT-VC`): three base64url segments
 * separated by dots, optionally with SD-JWT disclosures appended after a `~`. These envelopes
 * are NOT plain RDF — the surface recognises them and explains they need decoding/verification
 * (the separate ZK/crypto estate) before their claims can be queried as triples. It never
 * pretends to parse or verify them.
 */
export function looksLikeJwtVc(text: string): boolean {
  const head = text.trim().split("~")[0];
  return /^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/.test(head);
}
