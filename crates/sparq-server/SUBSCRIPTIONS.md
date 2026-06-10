# SPARQL Subscriptions (`/subscriptions`) — protocol v1 (T23)

`sparq-server` can push **live diffs of a SELECT query's result** to clients over a
WebSocket. A client registers a query once; after every committed SPARQL Update that
*changes* that query's result, the server sends the **added** and **removed** bindings —
each as a complete [SPARQL 1.1 Query Results JSON](https://www.w3.org/TR/sparql11-results-json/)
object.

## Lineage: SEPA, and where we diverge

The design follows the **SEPA** (SPARQL Event Processing Architecture) W3C member
submission, specifically the
[SPARQL 1.1 Subscribe Language](https://www.w3.org/submissions/2018/SUBM-sparql11-subscribe-20181016/)
(Blazegraph's competing subscription design was never shipped). From SEPA we keep:

* the **subscribe / unsubscribe** verb pair with an optional client-chosen **alias**
  echoed back in every related message;
* the **notification shape**: `addedResults` / `removedResults`, *both* full SPARQL JSON
  results objects, plus a per-subscription monotonically increasing **sequence** number
  whose first value (`0`) carries the complete initial result as `addedResults`;
* the *re-evaluate and diff on committed update* processing model.

Divergences (deliberate, for v1 simplicity):

| SEPA | sparq-server |
| --- | --- |
| Dedicated `sparql-se:` URI scheme + a separate subscribe HTTP endpoint | A plain WebSocket at `ws://host:port/subscriptions`; updates arrive through the ordinary `POST /sparql` (`application/sparql-update`) endpoint |
| `spuid` string subscription ids | Numeric `id`s, unique for the server's lifetime |
| Secure variant (OAuth, `wss`-only profile) | None — deploy behind your own TLS/auth layer |
| Subscriptions to any query form | **SELECT only** (the diff is defined over solution bindings) |
| Bag (multiset) result semantics | Diffs are over **distinct** bindings (set semantics; duplicate rows collapse) |

## Transport

* Endpoint: `GET /subscriptions` with a standard WebSocket upgrade.
* All protocol messages are **text frames containing one JSON object**. Binary frames are
  answered with an `error` message. Pings are answered automatically.
* Incoming frames are capped at the server's `--max-body-bytes` (default 1 MiB), the same
  guard as HTTP request bodies.
* One socket can hold **multiple subscriptions** (up to `--max-subscriptions-per-conn`).

## Client → server messages

### subscribe

```json
{"subscribe": {"query": "SELECT ?s ?age WHERE { ?s <http://ex/age> ?age }", "alias": "ages"}}
```

* `query` (required, string) — a SPARQL **SELECT** query.
* `alias` (optional, string) — echoed in `subscribed`, every `notification` and any
  `error` related to this request, so clients can correlate without tracking ids.

On success the server answers with **two** messages, in order:

```json
{"subscribed": {"id": 1, "alias": "ages"}}
{"notification": {"id": 1, "alias": "ages", "sequence": 0,
  "addedResults":   {"head": {"vars": ["s", "age"]}, "results": {"bindings": [ /* full current result */ ]}},
  "removedResults": {"head": {"vars": ["s", "age"]}, "results": {"bindings": []}}}}
```

The subscription is **refused** with an `error` message (and no slot is consumed) when:

* `query` is missing, malformed, or not a SELECT;
* the connection already holds `--max-subscriptions-per-conn` subscriptions (default 16);
* the server already holds `--max-subscriptions` subscriptions in total (default 256);
* the initial evaluation violates the server budget — it exceeds `--max-results` rows
  (mirroring the HTTP 413 refusal) or the `--query-timeout` deadline (mirroring the 503).

### unsubscribe

```json
{"unsubscribe": {"id": 1}}
```

Answered with `{"unsubscribed": {"id": 1}}`; an unknown id yields an `error`. Closing the
socket (cleanly or by vanishing) implicitly unsubscribes everything it held.

## Server → client messages

| Message | Shape | When |
| --- | --- | --- |
| `subscribed` | `{"subscribed": {"id": n, "alias"?: s}}` | Subscription accepted |
| `notification` | `{"notification": {"id": n, "alias"?: s, "sequence": k, "addedResults": R, "removedResults": R}}` | Sequence 0 immediately after `subscribed` (full result as added); then once per re-evaluation whose diff is non-empty |
| `unsubscribed` | `{"unsubscribed": {"id": n}}` | Unsubscribe acknowledged |
| `error` | `{"error": {"message": s, "id"?: n, "alias"?: s}}` | Refused subscribe, unknown unsubscribe id, unparseable frame, or a failed re-evaluation (which **terminates** that subscription — see below) |

`R` above is always a complete SPARQL JSON results object with the query's `head.vars`.
Term encoding matches the HTTP endpoint's `application/sparql-results+json`: `uri` /
`bnode` / `literal` (with `xml:lang`, and `datatype` for anything but plain `xsd:string`),
plus the SPARQL 1.2 `{"type": "triple"}` encoding for RDF 1.2 triple terms.

## Processing model: re-evaluate + diff, with coalescing

* **Commit hook.** A successful `POST /sparql` update atomically swaps the server's graph
  snapshot, *then* bumps a commit generation on a `tokio::sync::watch` channel that every
  subscription connection observes.
* **Re-evaluation.** A woken connection re-runs each of its SELECTs against a fresh
  snapshot on the blocking pool, under the server's query budget (`--query-timeout`
  deadline + `--max-results` row cap), diffs against the stored previous result, and sends
  a `notification` only when the diff is non-empty.
* **Diffing.** Each solution row is canonicalised to its SPARQL-JSON binding object; the
  serialised object is the row's identity. Added = rows only in the new result; removed =
  rows only in the old. Distinct-bindings (set) semantics.
* **Coalescing (the dirty-flag pattern).** The watch channel stores only the *latest*
  generation; `changed()` resolves once when the connection's seen value is stale,
  regardless of how many commits made it stale. If updates land while a re-evaluation
  batch is running, the generation moves again and exactly **one** further re-evaluation —
  against the then-latest snapshot — covers the whole burst. Consequences clients must
  accept:
  * notifications are per **re-evaluation**, not per commit — several rapid commits may
    arrive as one combined diff (one `sequence` step);
  * a commit whose effects are cancelled before the re-evaluation runs (insert+delete)
    may produce no notification at all;
  * `sequence` counts *delivered* notifications per subscription and increases by exactly
    1 each time.
* **Failure mid-life.** If a re-evaluation violates the budget (the data grew past
  `--max-results`, or it timed out), the server sends an `error` naming the limit and
  **terminates that subscription** (its slot is freed; other subscriptions on the socket
  are unaffected). Re-subscribe with a narrower query.

## Limits & flags

| Flag | Env var | Default | Meaning |
| --- | --- | --- | --- |
| `--max-subscriptions N` | `SPARQ_MAX_SUBSCRIPTIONS` | 256 | Active subscriptions, server-wide |
| `--max-subscriptions-per-conn N` | `SPARQ_MAX_SUBSCRIPTIONS_PER_CONN` | 16 | Active subscriptions per socket |

Subscription evaluations additionally run under the server's existing `--query-timeout`
and `--max-results` guards. Global slots are released on unsubscribe, on subscription
termination, and — via a drop guard — whenever the connection ends, however it ends.

## Known limitations (v1)

* **Blank nodes:** the engine relabels blank nodes between evaluations, so a row
  containing a bnode can appear as a remove+add pair across commits even when it is
  semantically unchanged.
* **SELECT only**; set-semantics diffs (duplicate rows collapse).
* While a connection's re-evaluation batch is running, its incoming frames (e.g.
  `unsubscribe`) queue on the socket and are processed right after the batch.
* No persistence: subscriptions die with the connection; reconnecting clients
  re-subscribe and receive a fresh sequence-0 full result.

## Example session

```text
client:  {"subscribe": {"query": "SELECT ?s ?age WHERE { ?s <http://ex/age> ?age }"}}
server:  {"subscribed": {"id": 1}}
server:  {"notification": {"id": 1, "sequence": 0, "addedResults": { ... 2 bindings ... },
          "removedResults": {"head": {"vars": ["s", "age"]}, "results": {"bindings": []}}}}

  (elsewhere)  POST /sparql  Content-Type: application/sparql-update
               INSERT DATA { <http://ex/carol> <http://ex/age> 35 }

server:  {"notification": {"id": 1, "sequence": 1,
          "addedResults": {"head": {"vars": ["s", "age"]}, "results": {"bindings": [
            {"s": {"type": "uri", "value": "http://ex/carol"},
             "age": {"type": "literal", "value": "35",
                     "datatype": "http://www.w3.org/2001/XMLSchema#integer"}}]}},
          "removedResults": {"head": {"vars": ["s", "age"]}, "results": {"bindings": []}}}}

client:  {"unsubscribe": {"id": 1}}
server:  {"unsubscribed": {"id": 1}}
```
