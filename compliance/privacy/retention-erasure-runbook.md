<!-- [OPUS-4.8] sq-toze.33 (PR-G3) — Operator runbook: data-subject erasure + retention on a
     sparq deployment. OPERATOR-FACING, NOT a certification claim. The deploying org is the
     GDPR controller; sparq is the technical means. Honest about WAL/backup/physical-erasure
     caveats. Re-review when Fable returns. -->

# Retention & erasure — operator runbook

> **What this is.** A practical runbook for an **operator** (the deploying organisation) to
> fulfil **data-subject erasure** (GDPR Art. 17 "right to be forgotten"), **rectification**
> (Art. 16), **access/portability** (Art. 15/20), and **storage-limitation / retention**
> (Art. 5(1)(e)) obligations *on a sparq deployment*.
>
> **What this is NOT.** This is **not** a certification or compliance claim, and not a
> statement that "sparq is GDPR compliant". **sparq is a data *engine*; the deploying
> organisation is the GDPR controller** (the ISO 27701 PII controller / SOC 2 entity) and owns
> the retention *policy*, the lawful basis, the data-subject-request process, and the
> attestation. See [`README.md`](./README.md) for the operator-vs-engine split and
> [`../data-flow.md`](../data-flow.md) for every place the binary can touch data.
>
> This runbook documents the engine **mechanism** the operator drives, and is **honest about
> the caveats** — notably that a logical SPARQL `DELETE` is **not** a physical, crypto-grade
> erasure (the `--persist` write-ahead log and any operator backups retain superseded data
> until rotated; sparq has **no built-in crypto-erase**). The operator must close those gaps.

## 0. Who does what (read first)

| Step | sparq provides (verified mechanism) | Operator orchestrates (responsibility) |
|---|---|---|
| **Locate** a subject's triples | `SELECT` / `CONSTRUCT` / `DESCRIBE` by identifier or named graph | Knows *which* identifier(s)/graph(s) belong to the subject (the engine has no notion of "personal data") |
| **Erase** | SPARQL `DELETE DATA` / `DELETE … WHERE` / `CLEAR` / `DROP GRAPH` | Decides scope; runs the update; authenticates the caller |
| **Rectify** | `DELETE { old } INSERT { new } WHERE { … }` (atomic) | Supplies the corrected values |
| **Export** (access/portability) | `SELECT`/`CONSTRUCT`/`DESCRIBE` → standard RDF/result serialisations | Scopes + delivers to the subject |
| **Retention enforcement** | (no built-in scheduler) — operator scripts a periodic `DELETE WHERE` | Owns the retention period + the schedule + the legal basis |
| **Durable-store erasure-completeness** | `--persist` WAL is append/replay; **must be rotated** | **Owns** WAL rotation + backup purge + at-rest encryption / crypto-erase |

**Honesty anchor.** Everything in the left column is a *mechanism* the engine genuinely
provides (cited below). Everything in the right column is the **operator's** job — the engine
cannot do it, and this runbook does not claim it does.

## 1. Locate the data subject's triples

The engine has **no notion of "personal data"** — it treats all RDF identically (see
[`README.md`](./README.md)). The operator must know which identifier(s) or named graph(s)
correspond to the data subject, then locate the triples with a read query.

By subject identifier (everything *about* the subject):

```sparql
SELECT ?p ?o WHERE { <http://example.org/person/alice> ?p ?o }
```

Also catch triples where the subject appears as an **object** (e.g. `?x :knows <alice>`):

```sparql
SELECT ?s ?p WHERE { ?s ?p <http://example.org/person/alice> }
```

If the operator partitions per-subject data into a **named graph** (a recommended deployment
pattern — see §6), enumerate it:

```sparql
SELECT ?s ?p ?o WHERE { GRAPH <http://example.org/subject/alice> { ?s ?p ?o } }
```

Run these against `/sparql` (HTTP) or `sparq_engine::{query, construct, describe}` (library).
The result is the **scope** the operator will erase. **Capability:** standard SPARQL
`SELECT`/`CONSTRUCT`/`DESCRIBE` (privacy control **P-5**; see [`controls.md`](./controls.md)).

## 2. Export (Art. 15 access / Art. 20 portability)

Before erasing, the subject may be owed a copy. RDF is inherently portable; a `CONSTRUCT` or
`DESCRIBE` produces a standard, machine-readable serialisation (Turtle / N-Triples / RDF-XML /
JSON results via HTTP `Accept` negotiation):

```sparql
CONSTRUCT { <http://example.org/person/alice> ?p ?o }
WHERE     { <http://example.org/person/alice> ?p ?o }
```

```sh
curl -G http://127.0.0.1:3030/sparql \
  -H 'Accept: text/turtle' \
  --data-urlencode 'query=CONSTRUCT { <http://example.org/person/alice> ?p ?o } WHERE { <http://example.org/person/alice> ?p ?o }'
```

**Capability:** standard SPARQL `CONSTRUCT`/`DESCRIBE` + content negotiation
(`crates/sparq-engine/src/lib.rs` construct/describe entry points; result serialisations in
`sparq-server`). The operator scopes, runs, and delivers — sparq provides the export
*mechanism*, not the delivery process.

## 3. Erase (Art. 17 right to erasure)

sparq supports the full SPARQL 1.1 Update erasure surface. Pick the narrowest operation that
covers the located scope.

### 3a. Pattern-scoped deletion — `DELETE WHERE` / `DELETE … WHERE`

Remove every triple with the subject as subject or object:

```sparql
DELETE WHERE { <http://example.org/person/alice> ?p ?o } ;
DELETE WHERE { ?s ?p <http://example.org/person/alice> }
```

```sh
curl -i http://127.0.0.1:3030/sparql \
  -H 'Content-Type: application/sparql-update' \
  -H 'Authorization: Bearer <TOKEN>' \
  --data 'DELETE WHERE { <http://example.org/person/alice> ?p ?o } ; DELETE WHERE { ?s ?p <http://example.org/person/alice> }'
```

### 3b. Ground deletion — `DELETE DATA`

When the exact triples are known:

```sparql
DELETE DATA { <http://example.org/person/alice> <http://schema.org/email> "alice@example.org" }
```

### 3c. Whole-document deletion — `DROP GRAPH` / `CLEAR GRAPH`

If per-subject data lives in one named graph (§6), erase the document in one operation. `DROP
GRAPH <g>` removes the named-graph entry entirely; `CLEAR GRAPH <g>` empties it but keeps the
(now-empty) entry:

```sparql
DROP GRAPH <http://example.org/subject/alice>
```

**Capabilities (verified):** `DeleteData`, `DeleteWhere`/`DeleteInsert`, `Clear`, `Drop` all
route through the durable `Graph` in `crates/sparq-engine/src/update.rs:420-465`. SPARQL 1.1
Update is W3C-conformance-gated. Erasure is an **IMPL mechanism / OPERATOR process** control
(**P-3 / P-4**; see [`controls.md`](./controls.md)).

### 3d. Atomicity

A multi-operation update body is **all-or-nothing** — there is no partial apply, so a
rectify-and-erase body either fully commits or fully rolls back (regression test
`multi_op_one_unauthorized_denies_whole_body_no_partial_apply`,
`crates/sparq-solid/tests/update.rs:443`; control **P-6**). Use this to combine the
subject-as-subject and subject-as-object deletes into one durable transaction.

### 3e. Authenticate the eraser

An erasure is a **write**. By default the bare server has **no authentication** (documented
boundary **B3** — see [`README.md`](./README.md) and `crates/sparq-server/README.md`). Before
accepting data-subject-request traffic, the operator MUST gate writes — either:

- `--auth-token <TOKEN>` (env `SPARQ_AUTH_TOKEN`): constant-time Bearer gate on every write
  (UPDATE + Graph-Store `PUT`/`POST`/`DELETE`); or
- front the server with a gateway / `sparq-solid` graph-level WAC/ACP authorisation
  (fail-closed) — controls **P-10 / P-11**.

## 4. Rectify (Art. 16 right to rectification)

Correct inaccurate data atomically:

```sparql
DELETE { <http://example.org/person/alice> <http://schema.org/email> ?old }
INSERT { <http://example.org/person/alice> <http://schema.org/email> "new@example.org" }
WHERE  { <http://example.org/person/alice> <http://schema.org/email> ?old }
```

**Capability:** `DeleteInsert` (`crates/sparq-engine/src/update.rs:481` onward); atomic per §3d
(control **P-6**).

## 5. Verify the erasure

After erasing, re-run the locate queries from §1 and confirm an **empty** result:

```sh
curl -G http://127.0.0.1:3030/sparql \
  --data-urlencode 'query=ASK { <http://example.org/person/alice> ?p ?o }'
# expect: {"boolean":false}

curl -G http://127.0.0.1:3030/sparql \
  --data-urlencode 'query=ASK { ?s ?p <http://example.org/person/alice> }'
# expect: {"boolean":false}
```

`ASK` returning `false` (or `SELECT` returning zero rows) confirms the triples are no longer
**logically present** in the live store. **This does not by itself confirm physical erasure** —
see §7.

## 6. Deployment pattern that makes erasure cheap and auditable

Erasure is dramatically simpler if the operator **partitions per-data-subject data into a
dedicated named graph** at ingest time (e.g. one graph IRI per subject). Then:

- **Locate** is `GRAPH <subject-iri> { ?s ?p ?o }` (no cross-graph scan).
- **Erase** is a single `DROP GRAPH <subject-iri>` (one atomic, scoped operation).
- The operator's records-of-processing (RoPA) can map subject → graph IRI directly.

This is an **operator deployment decision**, not an engine feature — sparq supports named
graphs and the Graph-Store Protocol; the operator chooses to use them this way.

## 7. Caveats — logical delete is NOT physical erasure (read before claiming Art. 17 done)

> **The single most important honesty item in this runbook.** A successful SPARQL `DELETE` /
> `DROP` makes data **logically absent from the live query surface**. It is **not**, on its
> own, a complete physical or cryptographic erasure of the *persisted* store or of operator
> backups. sparq has **no built-in crypto-erase**. The operator MUST address the following.

### 7a. The `--persist` write-ahead log retains superseded data (PR-G3)

When the server runs with `--persist DIR` (env `SPARQ_PERSIST_DIR`), every committed update is
**appended to a per-graph write-ahead log and fsync'd before the ack**, and on restart the WAL
is **replayed** (`crates/sparq-server/src/main.rs`; `crates/sparq-server/README.md` →
"Durable persistence"). The WAL is **append/replay**: a triple's earlier `INSERT` may remain in
prior WAL segments after a later `DELETE` removes it from the live store. Specifically:

- A pattern `DELETE` is recorded as a delete delta — the *value* is still recoverable from the
  earlier insert segment until the log is **compacted/rotated**.
- Per `crates/sparq-server/README.md` ("Deferred hardening"), WAL-durable `CLEAR` / `DROP GRAPH
  <g>` of an **existing** named graph are today applied in memory and persisted only at the
  **next compaction** — so until then, the dropped graph's data can still be on disk in the WAL.

**Consequence:** a complete Art. 17 erasure of the *persisted* store requires the SPARQL
`DELETE`/`DROP` **AND** a durable-store purge. Until sparq ships an explicit
`compact`/`vacuum` admin command (deferred — see gap below), the operator's purge procedure is:

1. **Quiesce writes** (stop accepting data-subject-request and ingest traffic, or take the
   instance out of rotation).
2. **Re-seed from a clean snapshot.** Export the *current* (already-erased) live store with a
   `CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }` (per named graph), then start a **fresh**
   `--persist` directory seeded from that export. The fresh WAL contains no superseded inserts.
3. **Destroy the old persist directory** (see §7c for at-rest/physical destruction).

This is operationally heavy; partitioning per subject into a named graph (§6) plus periodic
re-seeding minimises how often it is needed. Tracked as **PR-G3** in
[`gap-register.md`](./gap-register.md); the deferred `compact`/`vacuum` admin command that would
make this a one-step operation is bead **sq-x32t**.

### 7b. Operator backups retain data until rotated

Any backups, replicas, or filesystem snapshots the operator takes of the in-memory store or the
`--persist` directory will **retain the subject's data until those backups expire / are
rotated**. sparq has no visibility into the operator's backup regime. The operator's retention
policy MUST define a backup-retention window and ensure erased data ages out (or is purged on
request) — and MUST document, per Art. 17(1)/(2), how erasure propagates to backups (commonly:
"erased from live immediately; purged from backups on the next rotation cycle, max
`<FILL-IN: backup retention period>`").

### 7c. No engine-side at-rest encryption / crypto-erase

sparq holds data in process memory by default and writes a **plaintext** WAL with `--persist`
(control **P-9**; `crates/sparq-server/src/main.rs`). There is **no engine-side at-rest
encryption** and **no crypto-erase** (deleting a key to render ciphertext unrecoverable). At-rest
confidentiality and any crypto-erase strategy are the **operator's** responsibility:

- full-disk / volume encryption on the host and on backup media;
- if crypto-erase is the chosen erasure-completeness mechanism, the operator implements it at
  the storage layer (e.g. per-tenant volume keys destroyed on erasure) — **outside** sparq;
- secure physical destruction / overwrite of decommissioned persist directories and backup
  media per the operator's media-sanitisation policy.

### 7d. Request logs and metrics

Request logging is **off by default**; `--verbose` enables `TraceLayer` lines that can include
SPARQL query text (which may embed an identifier, e.g. `FILTER(?email = "alice@…")`) — gap
**PR-G4** (bead **sq-toze.34**). Prometheus `/metrics` are **aggregate-only** (counts, no
content). If `--verbose` is enabled, the operator must include those logs in the erasure scope
and apply the operator's log-retention/redaction policy. See [`controls.md`](./controls.md)
P-12 / P-2 and [`../data-flow.md`](../data-flow.md).

## 8. Retention-period enforcement (Art. 5(1)(e) storage limitation)

The retention **policy** (how long each category of personal data may be kept, and the legal
basis) is the **operator's** — sparq has no notion of categories, ages, or policy. sparq has
**no built-in retention scheduler**; the operator enforces retention by **scheduling periodic
erasure** of expired data.

If the operator records an ingest/observation timestamp on each subject record (e.g.
`prov:generatedAtTime` or a deployment-specific predicate), a scheduled `DELETE WHERE` with a
date `FILTER` enforces a retention window:

```sparql
# Run on a schedule (operator's cron / job runner). Deletes records older than the cutoff.
DELETE { ?s ?p ?o }
WHERE  {
  ?s ?p ?o ;
     <http://www.w3.org/ns/prov#generatedAtTime> ?t .
  FILTER( ?t < "<FILL-IN: retention cutoff, e.g. NOW() - P2Y>"^^<http://www.w3.org/2001/XMLSchema#dateTime> )
}
```

Operationalising:

- The operator runs this from its **own scheduler** (cron, systemd timer, k8s CronJob) against
  the authenticated write endpoint (§3e). sparq does not schedule anything itself.
- Treat each scheduled deletion as an erasure for §7 purposes: if `--persist` is on, the deleted
  values persist in the WAL until the next re-seed/compaction; fold a periodic re-seed (§7a)
  into the retention job cadence.
- **`<FILL-IN>`** the retention period(s), the per-category cutoff(s), the legal basis, and the
  schedule — these are deployment/policy values this runbook deliberately does not assume.

## 9. End-to-end checklist (operator)

1. **Authenticate** the write surface (`--auth-token` or gateway / `sparq-solid`) — §3e.
2. **Locate** the subject's triples (subject-position AND object-position; per-subject graph if
   used) — §1.
3. **Export** a copy if access/portability is owed — §2.
4. **Erase** with the narrowest operation (`DELETE WHERE` / `DROP GRAPH`), atomically — §3.
5. **Verify** logical absence with `ASK`/`SELECT` returning empty — §5.
6. **Purge the durable store** if `--persist` is on: re-seed from the erased snapshot + destroy
   the old WAL directory — §7a.
7. **Propagate to backups**: ensure the subject's data ages out of backups within the documented
   window, or purge on request — §7b.
8. **Address at-rest residue**: rely on the operator's full-disk encryption / crypto-erase /
   media sanitisation — §7c.
9. **Scrub logs** if `--verbose` was on — §7d.
10. **Record** the request, scope, actions, and completion in the operator's data-subject-request
    register (Art. 5(2) accountability) — operator responsibility.

## Engine gaps that would make this easier (tracked, not papered)

These are **engine-side capability gaps** that would reduce operator burden — recorded honestly
in [`gap-register.md`](./gap-register.md), not claimed as present:

- **PR-G3** — `--persist` WAL is not erasure-complete; no built-in `compact`/`vacuum` admin
  command. This runbook (bead **sq-toze.33**) documents the manual purge procedure in §7a; the
  optional `compact`/`vacuum` command is the **deferred** code fix, bead **sq-x32t**.
- **PR-G4** — no request-log redaction control (bead **sq-toze.34**) — affects §7d.
- A built-in **crypto-erase** / per-tenant key-destruction primitive is a deferred,
  out-of-scope engine feature (bead **sq-du24**); today crypto-erase is the operator's
  storage-layer concern (§7c).

## References

- [`README.md`](./README.md) — operator-vs-engine split, status legend, B3 no-auth boundary.
- [`controls.md`](./controls.md) — P-3/4/5/6 (erasure/rectification/access), P-9 (at-rest),
  P-10/11 (authz), P-12 (error/log hygiene).
- [`gap-register.md`](./gap-register.md) — PR-G3 (WAL erasure-completeness), PR-G4 (log
  redaction).
- [`../data-flow.md`](../data-flow.md) — the persist-WAL erasure caveat + the data-touch map.
- `crates/sparq-engine/src/update.rs` — the `DELETE`/`DROP`/`CLEAR` implementation.
- `crates/sparq-server/README.md` → "Durable persistence" — the WAL semantics + deferred
  hardening note.
