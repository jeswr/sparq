# Term-Index (Dictionary) Compression — design space and prioritised plan

How sparq should represent its term dictionary as it scales from the M1 toy store
to (a) a native billion-triple engine and (b) a small-bundle browser engine over
a 2–4 GB graph. This document is the dictionary-specific complement to
`ARCHITECTURE.md` §3.1 and `data-structures-discovery.md` §"Dictionary".

Sources are tagged `[measured]` (a paper/impl reports the number on real data),
`[claimed]` (authors assert, not independently reproduced), `[literature]`
(established result, not a new measurement), or `[speculation]` (our inference,
unmeasured for RDF/sparq). URLs are kept inline so claims are checkable.

---

## 0. Where sparq is today (the baseline being improved)

The current dictionary lives in `crates/sparq-core/src/dict.rs`. Two things are
already true that the prompt's framing pre-dates — they change the priority order:

1. **It is NOT double-stored.** `dict.rs` already implements the single-storage
   interner: an arena `terms: Vec<Term>` holds each non-inline term exactly once,
   and a `hashbrown::HashTable<Id>` stores **only the `u32` id**, rehashing the
   candidate back out of the arena to compare (`hash_term` is seeded-free `FxHasher`
   so it is stable). The `(Term, Id)` key duplication a `HashMap<Term, Id>` would
   keep is already gone. This is the survey's #1 "adopt now" find — **already done**.
   So "stop double-storing" is largely complete; what remains is that `Vec<Term>`
   still holds **owned `oxrdf::Term`s** (a fat enum: pointer+len+cap per string,
   plus a separate datatype `NamedNode` allocation per typed literal), not a packed
   byte arena. The *string bytes* are stored once, but with per-term `String`/`Term`
   slot overhead (~24–40 B/term of slot + allocator overhead) and **uncompressed**.

2. **Inline integers already work** (`INLINE_BASE = 1<<30`): canonical
   non-negative `xsd:integer` in `[0, 2^30)` never touches the dictionary. Ids are
   `u32`, partitioned `[1, 2^30)` dictionary · `[2^30, 2^31)` inline-int ·
   `[2^31, …)` local-vocab. **The inline range is nearly full and the id is only
   32 bits** — this is the binding constraint on extending inlining (below).

So the honest current state is: **strings stored once but as fat uncompressed
`Term`s; only `xsd:integer` inlined; ids are u32 and insertion-ordered (NOT
lexicographic / NOT order-preserving)**. The `ARCHITECTURE.md` end-state (tagged
64-bit ValueId + sorted front-coded vocab, §3.1) is the M4 target this plan
sequences toward.

Two findings from `data-structures-discovery.md` are load-bearing here and recur
below:
- **PtrHash / RecSplit MPHF** (~37 ns/key, 2.1–3.0 bits/key `[measured]`,
  https://arxiv.org/html/2502.15539) — a keyless string→id map, but **not
  order-preserving**, so it cannot *be* the rank without an **MMPHF** (rank in
  ~O(log log u) bits/key `[literature]`,
  https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ESA.2023.46), which
  in turn needs a membership guard (fuse/ribbon filter) because it returns garbage
  on absent keys.
- **SuRF / range-filter family** (https://db.cs.cmu.edu/papers/2019/20_srf-zhang.pdf)
  — a succinct *filter*, **lossy** (false positives, cannot materialise id→term),
  so it is an auxiliary block-range pruner, never the dictionary itself.

---

## 1. The design space

Axes: **mem** = bytes/term (lower better), **intern** = string→id hot-path speed
(bulk-load), **id→term** = materialisation speed (projection), **order** =
order-preserving ids (range FILTER → block pruning, ORDER BY), **build** =
build/freeze cost, **WASM** = bundle + no-disk + no-bit-twiddling fit.

Scores are relative (●●●●● best … ● worst) and synthesise the five surveys; the
space column is the measured ratio vs raw UTF-8 where available.

| Technique | mem (vs raw) | intern | id→term | order | build | WASM | Evidence |
|---|---|---|---|---|---|---|---|
| **Single-storage interner** (arena + ids-only table) — *current* | ~50% of double-stored; no entropy gain | ●●●●● O(1) hash | ●●●●● O(1) slice | ✗ insertion-order | ●●●●● incremental | ●●●●● pure Rust, in dep tree | `[measured]` lasso ~13 GiB/s; matklad |
| **IRI prefix/namespace factoring** | ~55% of vocab (Wikidata −45%) | ●●●● +split | ●●●● +concat | ✗ (sorts within prefix) | ●●●● 1-pass greedy or fixed split | ●●●●● table+concat, no bitops | `[measured]` QLever −45% (20→11 GB) |
| **Extend tagged ValueIds** (double/decimal/date/bool) | entry **eliminated** for inlinable literals | ●●●●● parse+pack | ●●●●● arith decode | ✓ value-order in id | ●●●●● none | ●●●● needs u64 id (index width) | `[measured]` QLever ValueId.h |
| **Lang-tag + datatype-IRI interning** | −(10..40 B)/literal | ●●●● +tiny lookup | ●●●● reassemble | ✗ | ●●●●● incremental | ●●●●● | `[literature]` Oxigraph/oxrdf |
| **Typed sub-dictionaries** (S/O-shared, P, IRI/lit/bnode split) | enables per-class codec; SO dedup | ●●●● smaller maps | ●●●● range dispatch | partial | ●●● needs shared-set pass / freeze | ●●●● | `[measured]` HDT four-section |
| **FSST** (per-string symbol table) | **~50%** (URLs 1.68×) | ●●●●● hash on compressed bytes | ●●●●● 1–3 GB/s | ✗ gain-ordered | ●●●● train pass | ●●●●● pure-Rust spiraldb/fsst, 4 KB tbl | `[measured]` VLDB'20 |
| **OnPair / OnPair16** | **~36%** (URLs 2.8×) | ●●●●● | ●●●●● ~5 GiB/s | ✗ | ●●● slower train | ●●● no mature Rust crate (2025) | `[measured]` arXiv 2508.02280 |
| **Plain Front-Coding (PFC/HDT)** | **3–25% on IRIs**; >80% on literals | ●● binary-search locate | ●●●● 0.2–0.4 µs extract | ✓ lexicographic | ●● sorted freeze | ●●●●● byte-aligned, no rank/select | `[measured]` Martínez-Prieto IS'15 |
| **HTFC** (PFC + Hu-Tucker/Re-Pair) | **4–5% on IRIs**, ~10% lit | ●● 1–6 µs locate | ●●● 0.4–2 µs | ✓ | ● Hu-Tucker+grammar build | ●● bit-level decode tables | `[measured]` IS'15 |
| **MARISA / Xcdat (succinct/DA trie)** | ~24% (MARISA) / ~49% (Xcdat) | ●●● | ●●●● | ✗ MARISA (weight/label order) ✓ Xcdat | ●● | ● C++ only, FFI/WASM friction | `[measured]` s-yata; Xcdat KAIS'17 |
| **BurntSushi `fst` (acyclic FST)** | ~20% on URLs (incl. value) | ●●● O(len) | ✗ **no id→term** | ✓ | ●● build-from-sorted, one-shot | ●●●● pure Rust, mmap | `[measured]` 1.6 B URLs 134→27 GB |
| **PtrHash / RecSplit (MPHF)** | 1.56–3.0 bits/key *index side only* | ●●●●● ~37 ns/key | n/a (needs side array) | ✗ not order-preserving | ●●● build | ●●●● no_std Rust | `[measured]` arXiv 2502.15539 |
| **MMPHF (monotone)** | ~O(log log u) bits/key | ●●●● | n/a | ✓ **rank-preserving** | ●●● | ●●● needs membership guard | `[literature]` ESA'23 |
| **Oxigraph inline-or-128-bit-hash** | strings once, no str→id map | ●●●●● hash, lock-free | ●●●● probe | ✗ random hash order | ●●●●● | ●●●● but 16 B ids bloat indices | `[claimed]` Oxigraph wiki |
| **HashDAC-rp / compressed-hash dict** | 12–65% | ●●●● O(1) hash | ●●● Re-Pair decode | ✗ | ●● Re-Pair build | ●● bit-level | `[measured]` IS'15 |
| **PDT / CoCo-trie** | near-XBW, order-preserving | ●●● | ●●● | ✓ | ● data-aware optimiser | ● C++/GPL, no Rust | `[claimed]` CoCo-trie |
| **XBW / FM-index self-index** | **3–20%** (smallest) | ✗ 20–200 µs | ✗ 50–500 µs | ✓ | ● could not build on Literals | ● heavy rank/select | `[measured]` IS'15 |
| **zstd block + trained dict** | ~49% (2.05×) | n/a (needs side index) | ✗ whole-block decode | ✓ if blocks ordered | ●●● | ●● zstd C build / ruzstd | `[measured]` |

**Reading the table.** Four clusters emerge:

- **Composable shrinks of *what the dictionary must hold*** (prefix factoring,
  ValueId inlining, lang/datatype interning, typed split). Order-neutral-to-friendly,
  incremental, WASM-trivial, keep the O(1) hash hot path. These *multiply* with
  everything below and carry the least risk.
- **Per-string payload compressors that keep the hash hot path** (FSST, OnPair,
  HashDAC-rp). Halve-or-better the string bytes, **not order-preserving**, random
  access intact. FSST is the pure-Rust, lowest-risk member.
- **Order-preserving sorted/sealed structures** (PFC, HTFC, FST, tries, MMPHF).
  Best space *and* the range-scan win sparq's `ARCHITECTURE.md` wants — but they
  trade O(1) intern for log-n locate and demand a **freeze** (lexicographic id
  assignment after the full term set is known). Right home: a post-load sealed tier.
- **Research endgame** (XBW, CoCo-trie, RDFCSA-as-vocab): smallest, slowest,
  C++-only or unported. Shelf.

The crucial tension for sparq is **order-preservation vs O(1) intern**. The
current insertion-order ids are O(1) to intern but give **no** range pruning on
strings; the `ARCHITECTURE.md` plan is to make ids the **lexicographic rank** so
range FILTER → block metadata pruning works (the same lever inline integers already
give). Everything order-preserving requires a freeze; everything hash-fast is not
order-preserving. **This is resolved by the bulk-load-then-freeze model QLever uses
and `ARCHITECTURE.md` §3.1 already commits to**: a build-time hash map for dedup,
then sort + assign ranks + drop the map.

---

## 2. Prioritised plan for sparq's dictionary

Bytes/term figures are *expected* and tagged with their basis. sparq has **no
measured dictionary bytes/term or load-throughput baseline yet** (BENCHMARKS.md has
none) — establishing that baseline is itself the first action (§3).

### (a) Adopt now — low risk, compose cleanly, keep the hot path

**A0. Single-storage interner — already shipped.** `dict.rs` already stores each
term once + an ids-only `HashTable`. *Remaining gap:* `Vec<Term>` holds fat
`oxrdf::Term`s, not a packed `(offset,len)` byte arena. Migrating to a real byte
arena (`Vec<u8>` blob + `Vec<u32>` offsets, hand back `&str`/`TermRef`) removes
the per-term `Term` slot (~24–40 B) and the second datatype-`NamedNode` allocation
on typed literals.
*Expected:* slot overhead from ~24–40 B/term → ~4–8 B/term (one offset + a small
kind/datatype tag). `[speculation]` for RDF magnitude; the storage-dedup logic is
`[measured]` (lasso ~13 GiB/s resolve, https://github.com/Kixiron/lasso).
*Cost:* moderate — touches `term()`/`intern()` signatures (borrow vs owned) and
the `oxrdf::Term` round-trip. *Order:* unchanged (insertion).

**A1. IRI prefix / namespace factoring — the highest-leverage adopt-now (see §3).**
Split each IRI at the last `/` or `#` (cheap, streaming) or via a one-shot greedy
longest-common-prefix pass over the sorted vocab at freeze (better ratio, QLever's
method). Store IRIs as `(prefix_id: u16/u32, suffix)`; intern the few-hundred
distinct prefixes once.
*Expected:* whole-vocab **−45%** on Wikidata `[measured]` (QLever 20→11 GB,
https://ad-publications.cs.uni-freiburg.de/theses/Bachelor_Johannes_Kalmbach_2018.pdf);
per Wikidata-style ~50-byte IRI, drops to suffix (~10–20 B) + ~4–6 B overhead, so
roughly **halves NamedNode heap**. On the most prefix-rich class (Uniprot URIs) the
lower bound is ~7% of raw `[measured]` (Martínez-Prieto et al. IS'15,
https://users.dcc.uchile.cl/~gnavarro/ps/is15.2.pdf).
*Speed:* intern +1 `memchr` split + tiny prefix-table probe (negligible); id→term
+1 concat at projection only. Hash hot path stays O(1).
*Order:* not order-preserving by itself (sorts within a prefix) — acceptable because
range FILTER rides on inline numeric ValueIds, not IRI string order.
*Cost:* low — localised to NamedNode storage; no new index structure; works during
streaming load with the fixed-split heuristic. **Three independent corroborations**
(QLever prefix codes, HDT/PFC front-coding, Virtuoso `RDF_PREFIX`).

**A2. Extend tagged ValueIds to double / decimal / date / dateTime / boolean.**
Mirror the existing `try_inline`: canonical-form-only, value packed in the id,
so these literals never enter the dictionary and sort by value.
*Expected:* **whole entry eliminated** per inlinable literal — −1 arena slot, −1
table entry, −(value + datatype-IRI + overhead) ≈ **40–80 B/term saved** on
date/decimal-heavy data `[measured]` QLever inlines exactly these
(https://github.com/ad-freiburg/qlever/blob/master/src/global/ValueId.h,
.../util/Date.h). Multi-percent whole-store win on sensor/Wikidata-statement data.
*Speed:* strictly faster (skip intern + lookup); numeric/date FILTER reads value
from id, no string parse — and **inline ids stay order-preserving**, extending the
range-pruning sparq already gets from integers to dates and doubles.
*Cost:* **the one real piece of work is the id-width decision.** sparq's u32 inline
range `[2^30, 2^31)` is nearly full and integers already consume it, so you cannot
add four more datatypes in u32 without shrinking dictionary capacity. **Widen `Id`
to u64 with a QLever-style high-bit datatype tag** (the `ARCHITECTURE.md` §3.1
ValueId end-state). This is an M4 commitment but the *design* should be locked now
because it constrains every index column width (u32→u64 doubles permutation index
bytes — a real cost to weigh against the dictionary savings).

**A3. Language-tag table + interned datatype IRIs.** Store `(value, datatype_id:
u16, lang_id: u16)` instead of repeating `xsd:string`/`rdf:langString`/lang strings
per literal. oxrdf already separates these fields, so it is a representation swap.
*Expected:* −(10–40 B)/literal on `xsd:string`/langString (the residual after A2
inlines numerics) `[literature]` (Oxigraph term kinds,
https://github.com/oxigraph/oxigraph/wiki/Architecture). Bounded but universal and
near-free. *Cost:* low; compose with A0.

### (b) Prototype — measure before committing; each needs a benchmark gate

**B1. FSST over the string arena (the payload compressor).** Train one shared
255-symbol table (or per-class: one for IRI suffixes, one for literals), compress
each arena entry independently, key the hash table on **compressed bytes** (FSST
equality is preserved). Pure-Rust `spiraldb/fsst` (no_std-able, 4 KB L1-resident
table, https://github.com/spiraldb/fsst; paper
https://www.vldb.org/pvldb/vol13/p2649-boncz.pdf).
*Expected:* **~2× on text, ~1.68× on URLs** `[measured]` → roughly **halves the
residual string bytes** *on top of* A1's prefix factoring (multiplicative, since
FSST captures local substrings A1 leaves). On literals (where PFC fails) FSST still
gets ~2×.
*Speed:* id→term 1–3 GB/s (sub-µs/term), intern unchanged (hash on compressed or
raw bytes). **Order:* NOT order-preserving** — acceptable for v1 because range
relies on inline ValueIds.
*Cost:* low–moderate; the spiraldb crate is self-described as not production-ready,
so budget hardening. **A/B against OnPair16** (2.8× on URLs `[measured]`, arXiv
2508.02280) only if FSST under-compresses literal-heavy data — OnPair has no mature
Rust crate, so it is a phase-2-of-this-prototype, not a starting point.
*Gate:* ship only if it beats A1-alone on bytes/term by a margin worth the decode
cost, with **no** id→term regression at projection.

**B2. Typed sub-dictionaries with the HDT shared-S/O section.** Split the vocab by
role (SO-shared / S / O / P) and by kind (IRI / literal / bnode), assign the shared
section the lowest ids. sparq already range-partitions ids and **already knows each
term's S/P/O role at intern time** (it builds six permutations), so the role is free.
*Expected:* removes one full copy of every term used as **both** S and O (HDT: SO is
a large fraction of entities, up to ~60% on linked data `[measured]`,
https://www.rdfhdt.org/hdt-internals/). Also the scaffolding that lets each class
use its best codec (prefix-factoring on IRIs, A2 on numerics, FSST/Re-Pair on
literals, a tiny cache-resident table on the ~tens-of-thousands of predicates).
*Speed:* neutral-to-positive (smaller homogeneous maps). *Order:* enables the
contiguous SO id range that shrinks join domains (`ARCHITECTURE.md` §3.1).
*Cost:* moderate — requires the shared-set pass and id-range bookkeeping; **best
done at the freeze step, bundled with the sorted-rank assignment** (it is the same
two-phase commit). *Gate:* the SO-overlap fraction on the target dataset justifies
the bulk-load complexity.

**B3. The freeze → sorted lexicographic-rank vocab + PFC (the order-preserving
endgame's first half).** This is the structural pivot the `ARCHITECTURE.md` M3/M4
plan already commits to: keep the FxHashMap **only during bulk load** for dedup;
at end-of-load, sort terms by Unicode codepoint, assign dense ids = rank (now
**order-preserving** → string range FILTER maps to block pruning, ORDER BY is free),
and **Plain Front-Code** each section (buckets of `b=16`, VByte shared-prefix-len +
suffix; `id→term` jumps to bucket `id/b`, `string→id` binary-searches headers).
Drop the hash map; post-freeze inserts go to the local-vocab overlay (`[2^31,…)`)
that already exists.
*Expected:* **3–25% of raw on IRIs** (PFC), and PFC *also* captures shared bytes
beyond the namespace that A1 misses (shared path segments, adjacent local-name
prefixes), so it strictly dominates A1 on space `[measured]` IS'15. extract
0.2–0.4 µs (fastest compressed extract measured). Combined with A1+FSST this is
the path to single-digit bytes/term on the IRI-heavy part.
*Speed cost — the catch to MEASURE:* `string→id` becomes binary-search + bucket
scan, **not O(1) hash**. sparq's bulk loader interns *every* triple term, so a naive
swap regresses load throughput. *Mitigations:* (i) the build-time hash map already
absorbs the load-path interning (you only locate at *query* parse time, which
resolves few tokens); (ii) optionally front a **PtrHash MPHF** (~37 ns/key
`[measured]`) for O(1) hot lookups, or an **MMPHF** (rank in ~log log u bits/key
`[literature]`, ESA'23) — the rare order-preserving keyless map — with a fuse/ribbon
membership guard for absent keys.
*WASM:* PFC is byte-aligned, no rank/select, decodes without allocation — the most
WASM-friendly compressed option.
*Cost:* moderate–high (freeze machinery, sorted external merge, overlay for
inserts). This is M3/M4 work, not now. *Gate:* the freeze must not regress query
parse latency, and the range-FILTER → block-pruning win must show up on q06-style
queries (`bit-level-encoding.md` reports 4.8→1.6 ms from inline-id range pruning;
string-range pruning should show an analogous drop).

### (c) Shelf — wrong tradeoff for an in-RAM, intern-heavy, WASM store

- **HTFC / Re-Pair grammar over buckets.** Best space (4–5% on IRIs `[measured]`)
  but adds Hu-Tucker construction + Huffman/Re-Pair decode tables and bit-level
  per-access work that taxes WASM and the hot path. **Phase-2 space upgrade *after*
  PFC is proven** and only if a profile shows PFC leaves too much on literal-heavy
  vocab. Not adopt-now.
- **Oxigraph 128-bit-hash ids.** Solves str→id-map removal but is **not
  order-preserving** (kills range scans) and **inflates every permutation column
  from u32/u64 to u128** — and sparq's whole storage win over Oxigraph (8 GB vs
  67 GB on DBLP-390M, `ARCHITECTURE.md` §1) *depends* on narrow ids. Mine it only
  as a guide for widening inline-value tags (A2), not as the dictionary.
- **MARISA / Xcdat / PDT / CoCo-trie.** Each fails a hard requirement: MARISA is
  C++-only and not order-preserving by default; Xcdat only ~2× and C++; PDT/CoCo
  are C++/GPL with no Rust port for an uncertain marginal win over HTFC on IRIs.
  Research stretch only if HTFC's space proves insufficient.
- **XBW / FM-index self-index.** Smallest (3–20% `[measured]`) but 20–500 µs/op
  (100–1000× the hot path) and could not even be built on the Literals dataset.
  Only as a cold archival tier if compressed *substring search* over terms ever
  becomes a goal — it is not.
- **`fst` (BurntSushi) as the primary dictionary.** Pure-Rust and order-preserving
  but **term→value only — no built-in id→term**, so it needs a separate front-coded
  store for materialisation; PFC gives both directions in one structure. Keep as a
  *candidate backend* for the str→id half if PFC's locate proves too slow.
- **zstd block + trained dict / rANS.** Whole-block decode for one term is
  cache-hostile for the random single-term access a query engine does, and
  under-compresses IRIs vs PFC; the zstd C build is a WASM bundle cost. Cold/overflow
  tier only.
- **DAC for the offset array.** Complementary, not primary — adopt only *after* a
  payload compressor lands and the offset array is a *measured* native-scale
  bottleneck; a flat u32 offset array is simpler and fine for the WASM budget.

---

## 3. The single highest-leverage change to make FIRST — and what to measure

**Do FIRST: IRI prefix/namespace factoring (A1).**

Why it, over the others:
- **The interner (A0) is already shipped**, so the survey's nominal #1 is largely
  done; the remaining arena migration is a refactor, not a new capability.
- **A1 is the biggest single bytes/term win for the least risk and least
  architectural disruption.** It attacks the *dominant* redundancy in RDF (the
  repeated `http://…/` namespace bytes, the same redundancy SPARQL `PREFIX` targets),
  it is order-neutral so it does not force the freeze, it keeps the O(1) hash hot
  path, it ships incrementally during streaming load, and it shrinks **live RAM**
  (so it helps the browser target as much as native). It is corroborated by three
  independent production systems (QLever −45%, HDT, Virtuoso).
- A2 (ValueId widening) is higher-certainty *per inlinable literal* but its real
  work is the u64 id-width decision that ripples into every index column — a larger
  commit better sequenced as the deliberate M4 ValueId step. A1 has no such
  ripple.

**What to measure to validate it (and to establish the missing baseline):**

There is currently **no measured dictionary baseline in BENCHMARKS.md** — so the
first job is to add three metrics to the bench harness and capture them *before*
and *after* A1, on the same datasets the architecture already targets (DBLP-390M
first; a Wikidata sample for IRI-heavy validation).

1. **Bytes/term (the headline).** Extend `Dict::heap_bytes()` (already present) to
   report `dictionary_heap_bytes / dict.len()` split by class (IRI / literal /
   bnode). Target: A1 roughly halves the IRI-class figure; report the whole-vocab
   delta against QLever's −45% Wikidata datapoint as the sanity check. Also report
   **total dictionary heap as a % of total store RAM** — the architecture's claim
   is the dictionary is ~half of memory; confirm it, then watch it fall.
2. **Load throughput (intern hot path must not regress).** triples/s end-to-end on
   the bulk loader (bar: QLever ~1.7M triples/s on DBLP-390M, `ARCHITECTURE.md` §2).
   A1 adds a `memchr` split + a tiny prefix probe per IRI; confirm the regression is
   in the noise. This is the guard that A1 did not quietly slow ingest.
3. **Query latency (the order/materialisation side).** Per-query mean/median on the
   M1 query set, watching specifically (a) projection latency (id→term now does a
   prefix concat — confirm it stays sub-µs/term and does not dominate
   serialization) and (b) that nothing regressed because the term representation
   changed. When the freeze + sorted ranks land later (B3), the *new* metric to add
   is **range-FILTER block-pruning effectiveness** — blocks skipped on a string
   range predicate, analogous to the inline-id range pruning that already takes
   q06 from 4.8→1.6 ms (`bit-level-encoding.md` §2.4).

Acceptance for A1: measurable bytes/term drop on the IRI class (target ~2× on
Wikidata-shaped IRIs), **no** statistically significant load-throughput or
query-latency regression. If A1 passes, sequence A2/A3 next, then prototype the
freeze (B3) where the order-preserving range-scan win is independently justified by
metric (3).

---

## 4. Native-billion-scale goal vs browser goal — they diverge

The two deployment targets pull the dictionary design in different directions; the
adopt-now layer (A1–A3) serves both, but the prototype/endgame choices split.

### Native, billion-triple (Wikidata ≈20B, Uniprot 100B+)

- **Binding constraint: total RAM and build throughput**, not bundle size. The
  dictionary is a dominant component (QLever: cold Wikidata vocab ~206 GB,
  `ARCHITECTURE.md` §3.1).
- **u64 ValueId is mandatory** (A2): 2^30 inline + 2^30 dictionary is nowhere near
  20B terms; widen and accept the index-column cost.
- **Internal/external vocab split is available here and ONLY here.** Push the rare/
  long/cold tail (long literals, minority-language strings, abstract intermediate
  IRIs) to an mmap'd on-disk sorted array; QLever measured Wikidata in-RAM vocab
  **80→20 GB (~4×)** from externalisation alone, before any string compression
  `[measured]` (Kalmbach thesis Ch.5). This **requires durable random-access
  storage** and breaks strict global order across the in/out boundary — fine
  natively, impossible in the browser.
- **Endgame: freeze → sorted ranks + PFC (→ HTFC) + FSST hot tier**, exactly the
  `ARCHITECTURE.md` §3.1 M3/M4 plan (PFC M1–M3, FSST upgrade M4). At this scale the
  **offset array itself is large** → DAC/Elias-Fano on offsets becomes worth it.
- **MPHF (PtrHash) for the hot str→id half** is most justified here, where 37 ns/key
  + batch prefetch matches the scan/join hot loops and the keyless map saves the most
  absolute bytes.
- Build throughput bar: QLever vocab 0.4 h + global-id 0.2 h on Wikidata
  (`ARCHITECTURE.md` §2); the freeze + sort + front-code must fit that envelope via
  the external merge-sort partial-vocab pipeline already designed (§3.1).

### Browser / WASM (Solid/RDFJS, 2–4 GB graph, small bundle)

- **Binding constraints: bundle size, no mmap / no disk (everything in an
  `ArrayBuffer`), no cross-origin-isolation by default (single-thread), and
  bit-twiddling is comparatively costly.**
- **The external/cold-vocab split is OFF the table** (no durable random-access
  store) — the native engine's single biggest lever (4× from externalisation) does
  not exist here, which is exactly why **in-RAM compression matters more** in the
  browser: A1 (prefix factoring) + FSST + PFC must carry the whole load.
- **Favour byte-aligned, no-rank/select structures:** A1 (table+concat), **FSST**
  (pure-Rust spiraldb, 4 KB table, no_std → tiny bundle), and **PFC** (byte-aligned,
  no succinct machinery). **Avoid** HTFC/Re-Pair/XBW/wavelet-tree/LOUDS here — their
  bit-level decode tables cost both bundle size and per-access latency on WASM.
- **Ship a prebuilt sealed `.sparq` index** (sorted front-coded dict + permutations,
  zero-copy `bytemuck`-castable from the fetched `ArrayBuffer`) rather than parsing
  client-side — so the *freeze* runs offline on the server and the browser only ever
  does the cheap id→term decode (`ARCHITECTURE.md` §6). This neatly sidesteps the
  freeze's intern-regression problem in the one place it would hurt most.
- **u64 vs u32 id width is a real bundle/RAM tradeoff at 2–4 GB:** a few-hundred-
  million-term browser graph may fit u32 dictionary ids + a 32-bit inline scheme,
  keeping the permutation indexes half the size. Consider a **u32 browser profile /
  u64 native profile** behind the existing feature gating (`ARCHITECTURE.md` §6
  already feature-gates the wasm build). The order-preserving range win (B3) is *more*
  valuable in the browser because there is no cold tier to fall back on.

**Net:** A1–A3 are unconditional and serve both. Natively, layer external-vocab
split + u64 + PFC/HTFC + DAC offsets + PtrHash. In the browser, layer FSST + PFC on
a prebuilt sealed index, keep ids as narrow as the graph allows, and never reach for
bit-level succinct structures.

---

## 5. One-screen summary

- **Already done:** single-storage interner + inline `xsd:integer` (`dict.rs`).
- **Do first:** **IRI prefix/namespace factoring (A1)** — ~−45% vocab `[measured]`,
  order-neutral, O(1) hot path preserved, helps native + browser equally. Measure
  **bytes/term (by class) + load throughput + query/projection latency**; gate on
  bytes/term down, no throughput/latency regression.
- **Then adopt:** widen `Id`→u64 + extend inline ValueIds to double/decimal/date/
  bool (A2); intern lang-tags + datatype IRIs (A3); finish the arena migration (A0).
- **Prototype (gated):** FSST over the arena (B1); typed sub-dicts + HDT shared-SO
  (B2); the **freeze → sorted-rank + PFC** order-preserving endgame (B3, = the
  `ARCHITECTURE.md` M3/M4 vocab plan), fronted by a build-time hash map or
  PtrHash/MMPHF so intern does not regress.
- **Shelf:** HTFC/Re-Pair, Oxigraph 128-bit hash, MARISA/Xcdat/PDT/CoCo (C++/no
  Rust or not order-preserving), XBW/FM-index, zstd-block, `fst`-as-primary.
- **Native vs browser:** native adds the external/cold-vocab split (4× `[measured]`,
  needs disk) + u64 + DAC offsets + PtrHash; browser ships a **prebuilt sealed
  index**, FSST+PFC only, narrowest ids the graph allows, no bit-level succinct
  structures.
