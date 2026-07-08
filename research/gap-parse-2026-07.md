<!-- [SONNET-4.6] sq-hmd7l.6 first-read gap record. All rows are NON-CANONICAL
(measured on the shared work box, not a quiet EC2 instance). Canonical numbers
ride sq-hmd7l.26. -->

# RDF parse competitor gap — first-read record (2026-07-07)

**Status:** NON-CANONICAL first-read. Canonical execution: sq-hmd7l.26 (quiet EC2).
**Bead:** sq-hmd7l.6. **Epic:** sq-hmd7l.
**Box:** shared work box (EC2 work instance, not a dedicated bench box — numbers are
load-dependent and not suitable for publication).

## Harness

`bench/parse/src/main.rs` subcommand `bench-ext <file>`:

- Invokes **serdi** (serd 0.32.2), **rapper** (raptor2 2.0.16), and Jena **riot**
  (not installed on this box) as subprocesses over the same corpus file used by
  `bench-nt` / `bench-ttl`.
- Regime: **subprocess** — wall time includes process spawn, kernel file-I/O,
  parse, and process exit. This is NOT equivalent to the in-process `oxttl
  parse-only` rows, which exclude all three costs. Every task label contains
  the word "subprocess" to make this explicit.
- Mode selection:
  - **rapper** `-c` count-only: parses without serializing output; closest
    available approximation to parse-only for an external tool.
  - **serdi** pipes stdout to `/dev/null` for timing (full parse + NT serialization
    to sink); count verified separately via `serdi <file> | wc -l`.
  - **Jena riot** `--count` (not available on this box; column absent).
- Count guard: the tool's reported or observable triple count is cross-checked
  against the authoritative oxttl count before any MB/s row is printed. A mismatch
  suppresses the row (no fabricated numbers).
- Absent tool: column absent, no placeholder.
- ITERS=3, min-of-3 wall time reported (not median, to reduce process-spawn noise).

## Tool availability on this box

| Tool | Version | Status |
|---|---|---|
| serdi | 0.32.2 | installed (`/usr/bin/serdi`) |
| rapper (raptor2) | 2.0.16 | installed (`/usr/bin/rapper`) |
| Jena riot | — | not installed — column absent |

## NON-CANONICAL first-read rows (2026-07-07, shared work box)

Dataset: synthetic 50 000-entity graph, 400 000 triples.
NT file: 25 944 790 bytes. TTL file: 9 072 587 bytes.

**All numbers below are NON-CANONICAL — do not cite; canonical rows from sq-hmd7l.26.**

### N-Triples (400 000 triples, 25.9 MB)

| dataset | task | threads | s (min-of-3) | MB/s | Mtriples/s | count-ok |
|---|---|---|---|---|---|---|
| bench-smoke.nt | rapper/raptor2 N-Triples (subprocess, count-only, no store) | 1 (subprocess) | 0.500 | 52 | 0.80 | yes |
| bench-smoke.nt | serdi/serd N-Triples (subprocess, parse+serialize-to-sink) | 1 (subprocess) | 0.304 | 85 | 1.31 | yes |

In-process reference (from `bench-nt` on the same box, same file):

| task | MB/s | Mtriples/s |
|---|---|---|
| memscan ceiling (1 core) | 28 683 | 442 |
| oxttl NT parse-only | 97 | 1.49 |
| custom NT parse+intern (incumbent, 1T) | ~170 | ~2.6 |

### Turtle (400 000 triples, 9.1 MB)

| dataset | task | threads | s (min-of-3) | MB/s | Mtriples/s | count-ok |
|---|---|---|---|---|---|---|
| bench-smoke.ttl | rapper/raptor2 Turtle (subprocess, count-only, no store) | 1 (subprocess) | 0.458 | 20 | 0.87 | yes |
| bench-smoke.ttl | serdi/serd Turtle (subprocess, parse+serialize-to-sink) | 1 (subprocess) | 0.263 | 35 | 1.52 | yes |

## Regime differences — honest comparison notes

1. **subprocess vs in-process**: process spawn + OS scheduling adds ~2–10 ms on
   this box even for a no-op. At 26 MB the spawn overhead is ~1–2% of total wall
   time; at smaller files it dominates. Canonical rows must use a large corpus
   (≥100 MB) to make spawn cost negligible, or report spawn overhead separately.

2. **rapper count-only vs serdi parse+serialize-to-sink**: rapper `-c` avoids all
   serialization; serdi parses AND writes NT to a sink. The serdi column includes
   serialization cost and will appear faster in MB/s only if its parse throughput
   strongly outweighs the added I/O — the current first-read numbers suggest serdi
   is faster on both NT and TTL even with the serialize cost. This warrants
   investigation on the canonical box.

3. **Jena riot**: not available on this box. The canonical run (sq-hmd7l.26) must
   have riot installed and should use `riot --count` mode.

## Pending

- Canonical EC2 run with riot installed: sq-hmd7l.26.
- suite-id registration (`parse-competitors`): sq-hmd7l.1 (dep, not yet landed).
- Regime investigation: serdi faster than rapper despite serialize step — profile
  on canonical box.
