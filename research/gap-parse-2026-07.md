<!-- [SONNET-4.6] sq-hmd7l.6 first-read gap record. The 2026-07-07 rows are
NON-CANONICAL (shared work box). CANONICAL wave-1 rows (sq-hmd7l.26) are in the
canonical section below and supersede them. -->

# RDF parse competitor gap — first-read record (2026-07-07)

**Status:** CANONICAL wave-1 rows recorded (sq-hmd7l.26, quiet EC2 — see the
canonical section below). Headline: NT CLEARLY-AHEAD only with chunk-parallelism;
**Turtle single-thread is BEHIND serd (~2×)** — dominance gap row for `sq-hmd7l.27`.
**Bead:** sq-hmd7l.6. **Epic:** sq-hmd7l.
**Box (first-read sections):** shared work box (EC2 work instance, not a dedicated
bench box — numbers are load-dependent and not suitable for publication).

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

## CANONICAL wave-1 rows (`sq-hmd7l.26`)

Provenance: dedicated quiet box `i-06efda08048633f55` (c6i.4xlarge, eu-west-2, tag
`sparq-bench`, self-terminated, orphan-check clean), commit `a1bf9b48`
(= `origin/main` `343aee547` + wave-1 runner/harness-only fixes), UTC
`2026-07-10T00:58:43Z`, `CANONICAL=1`. Corpus: deterministic synthetic
320 000-entity graph, **2 560 000 triples**; NT 169 964 508 B (~170 MB — clears the
≥100 MB canonical floor from note 1 above), TTL 60 022 305 B. All external columns
min-of-3, count crosschecked against the authoritative oxttl count (**count-ok yes
on every row**; riot required the grouping-separator fix, see below). Raw rows:
`axis-results/parse/parse-rows.txt`. Cross-read: a first canonical read on box
`i-01a735e27b1764317` (00:53Z, same commit base; riot rows suppressed there by the
comma bug, which is why box-2 is transcribed) reproduces the subprocess competitor
rows to within ~0.5 % and the 16-thread rows to within ~5 %, but the single-thread
in-process rows ran up to ~28 % FASTER on box-1 (custom NT 1T 0.921 s / 2.78 Mt/s
vs 1.278 s / 2.00 Mt/s here) — real between-box single-thread variance on the same
instance type. The verdicts below hold under either reading (NT 1T lead vs serd is
1.2–1.7×, still not OOM; Turtle 1T stays BEHIND serd ~1.8–2.0×).

### N-Triples (170 MB, 2.56 M triples)

| task | threads | s | MB/s | Mtriples/s |
|---|---|---|---|---|
| sparq custom NT parse+intern (in-process) | 1 | 1.278 | 133 | 2.00 |
| sparq custom NT parse+intern (in-process) | 16 | 0.164 | 1033 | 15.56 |
| oxttl NT parse-only (in-process reference, not sparq's parser) | 1 | 1.435 | 118 | 1.78 |
| serdi/serd (subprocess, parse+serialize-to-sink) | 1 | 1.520 | 112 | 1.68 |
| rapper/raptor2 (subprocess, count-only) | 1 | 2.551 | 67 | 1.00 |
| Jena riot (subprocess, `--count`) | 1 | 4.896 | 35 | 0.52 |

### Turtle (60 MB, 2.56 M triples)

| task | threads | s | MB/s | Mtriples/s |
|---|---|---|---|---|
| sparq Turtle parse+intern (incumbent chunked, in-process) | 1 | 3.153 | 19 | 0.81 |
| sparq Turtle parse+intern (incumbent chunked, in-process) | 16 | 0.490 | 122 | 5.22 |
| serdi/serd (subprocess, parse+serialize-to-sink) | 1 | 1.547 | 39 | 1.65 |
| rapper/raptor2 (subprocess, count-only) | 1 | 2.194 | 27 | 1.17 |
| Jena riot (subprocess, `--count`) | 1 | 4.913 | 12 | 0.52 |

### Verdicts (fixed vocabulary; regime caveats from the section above apply)

| axis | verdict |
|---|---|
| NT, 1 thread | AHEAD-BUT-NOT-OOM vs serd (~1.2× Mt/s; and serd is doing extra serialize work); ~2× vs rapper; ~3.8× vs riot |
| NT, 16 threads | CLEARLY-AHEAD (~9.3× vs serd — chunk-parallelism is the lever; the external tools are single-threaded by design) |
| Turtle, 1 thread | **BEHIND serd (~2.0×) and rapper (~1.4×)** — sparq 0.81 Mt/s vs serd 1.65 / rapper 1.17, and serd additionally serializes. Honest dominance-gap row for `sq-hmd7l.27`: single-thread Turtle parse throughput (the incumbent chunked parser sits at oxttl speed, ~0.8 Mt/s) needs a profiling-first fix bead |
| Turtle, 16 threads | AHEAD-BUT-NOT-OOM (~3.2× vs serd) — parallelism recovers the lead but not by an order of magnitude |

Wave-1 execution notes: (a) riot 5.4.0 prints `Triples = 2,560,000` — the
grouping-separator count-parse fix (this wave's PR) un-suppressed the riot rows;
(b) subprocess spawn overhead is ~1–2 % at this corpus size (note 1 satisfied);
(c) the serdi-faster-than-rapper regime question from the first read reproduces
canonically on both formats.

## Pending

- suite-id registration (`parse-competitors`): sq-hmd7l.1 (dep, not yet landed).
- Turtle single-thread BEHIND row → root-cause fix bead via `sq-hmd7l.27`.
