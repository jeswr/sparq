# Audit: does any observable / canonical output depend on the thread-count-dependent dict-id assignment order?

Status: AUDIT — CONCLUSION (sq-xom2). Code-level, read-only. [OPUS-4.8]

Surfaced by the #691 (sq-lqty) fix: `sparq-core`'s parallel sharded dictionary merge
assigns dictionary ids in a **thread-count-dependent** order, and that order leaked into
`sparq-algos` `NodeGraph` node-index assignment (since fixed by canonicalising the
node order in `graph.rs`). This note answers the follow-up: does *anything else* that is
observable or canonical depend on that internal id order?

## TL;DR verdict

**No leak into any surface that is required to be canonical / deterministic.** Every
required-canonical surface — RDF canonicalisation (`sparq-canon`), the ZK commitment
(`sparq-zk`), HDT write (`sparq-hdt`), and the W3C conformance comparator — sorts or
canonicalises on the *lexical term form*, independently of dict-id order. The
conformance harness compares unordered results as a **bag (multiset)** and uses sequence
order only for `ORDER BY`, so it is robust to id-order churn.

**But the premise is real and broader than `sparq-algos`.** Several *observable but
spec-unspecified* surfaces inherit the id order: RDF serialisation row order
(Turtle/TriG/N-Quads/N-Triples/JSON-LD), `SELECT` result row order without `ORDER BY`,
`CONSTRUCT`/`DESCRIBE` triple order, and `ORDER BY` **tie** order. None of these is a
correctness bug (all are unspecified by the SPARQL / RDF specs), and no golden file or
digest pins them — but they *do* change across thread counts, so a future golden/snapshot
test that serialises a graph or pins an unordered/tied result would be host-flaky exactly
like the #691 `label_propagation` test was. That is a latent footgun, not a present
defect.

**One narrow REAL robustness concern (not a correctness bug):** the `sparq-vectors`
graph *fingerprint* (`content_hash`) is — by deliberate design — id-ordered. It is safe
on the persisted-store serving path (ids are frozen on disk via `save_mmap` and stable on
reopen), but it can spuriously mismatch if a vector store is bound to a graph
**re-loaded from source RDF at a different thread count** instead of reopened from a
persisted dict. The mechanism *fails closed* (a descriptive error, never wrong vectors),
so it is a usability/portability sharp edge, not unsoundness.

## The mechanism (confirmed)

`crates/sparq-core/src/lib.rs` — the parallel build path sizes the merge by thread count:

```rust
fn default_shards() -> usize {
    (rayon::current_num_threads() * 2).clamp(4, 64)
}
```

`crates/sparq-core/src/dict.rs` — `ShardedDict::new(n)` makes `n` leaf shards
(`+ 1` triple shard); leaf terms are routed `hash % n_leaf`; final dense ids are assigned
in **shard order** (`bases()` accumulates shard term-counts left-to-right; a term's final
id is `base[shard] + local`). For a fixed shard count the assignment is fully
deterministic (each shard walks partials in order), but the **number of shards depends on
`rayon::current_num_threads()`**, so the same logical graph loaded at 2 vs 8 threads gets
a different (still internally consistent) id→term binding. This is exactly what the #691
commit message reproduced at `RAYON_NUM_THREADS=2/4` vs `1/3/8/16`.

> Correction to a tempting mis-reading: it is *not* true that "the sharded merge is
> deterministic, therefore dict-ids do not vary by thread count." The merge is
> deterministic *given a fixed shard count*; the shard count itself is thread-count-derived,
> so the id order is thread-count-dependent. (One sub-investigation initially asserted the
> ids were thread-count-independent — that is wrong, and the #691 fix exists precisely
> because they are not.)

CI does **not** pin `RAYON_NUM_THREADS` on the conformance / bulk test shards (only the
HNSW-recall shards pin it to `1`, per `.github/workflows/ci.yml`), and the nextest matrix
runs shards on differently-sized runners — the same environment that made #691 flaky.

## Per-surface evidence

Legend: **SAFE** = canonicalises independently of dict-id order; **OBSERVABLE (unspecified)**
= varies with thread count but spec-permitted and not pinned anywhere canonical;
**WATCH** = real sharp edge worth a follow-up.

### RDF canonicalisation — SAFE

`crates/sparq-canon/src/lib.rs`. `canonicalize_triples` materialises every triple to its
**lexical `oxrdf` term form** (`Dict::term(id)`), serialises to canonical N-Quads text
(`format!("{} {} {} .\n", …)` over the `Display` impl), and hands that *string* to
`rdf_canon::canonicalize_quads` (the W3C-test-suite-validated RDFC-1.0 implementation).
The blank-node labelling (`hash-first-degree` / `hash-n-degree` / issuance) runs only on
the lexical N-Quads form. No dict-id is hashed, sorted on, or iterated. The `rdf-canon`
W3C suite (`crates/sparq-canon/tests/rdf_canon_suite.rs`) pins byte-for-byte output. RDF
canonicalisation is dict-id-order-**independent**.

### ZK commitment — SAFE

`crates/sparq-zk/src/commit.rs`. The commitment pipeline RDFC-1.0-canonicalises the graph
*first*, then encodes each **canonical** triple to a leaf in canonical N-Quads order and
folds the leaf sequence with Poseidon2. It loops over `canonical.triples` (canon order),
never over dict-ids, so `C(G)` is independent of dict-id order. (Caveat per repo policy:
the v1 verifier is remediated + internally re-audited but external accredited-cryptographer
sign-off is PENDING (sq-qhy4); this audit only establishes that the *graph-to-leaves
encoding order* does not depend on dict-id order — it makes no claim about the soundness of
the ZK scheme.)

### HDT write — SAFE

`crates/sparq-hdt/src/encode.rs`. The four dictionary sections (shared / subjects /
objects / predicates) are built as `BTreeSet<&str>` over the **rendered term strings** —
i.e. lexically sorted — then Plain-Front-Coded. HDT ids are ranks within those
lexically-sorted sections, and the BitmapTriples are sorted in HDT-id (hence lexical) SPO
order. The on-disk archive is therefore a function of the term *set*, not of sparq's
internal dict-id order. (`crates/sparq-hdt/tests/write_roundtrip.rs` pins save→load→save
stability.)

### W3C conformance comparator — SAFE

`crates/sparq-conformance/src/compare.rs` does **bag (multiset)** comparison for unordered
result sets and **sequence** comparison only when the query carries `ORDER BY`
(`run.rs::is_ordered`). Blank nodes are matched by bijection. So a thread-count-driven
change in result row order does not affect conformance pass/fail for the common case. (See
the residual ORDER-BY-tie note below for the one edge it does *not* cover.)

### RDF serialisation row order — OBSERVABLE (unspecified)

`crates/sparq-engine/src/serialize.rs`. Every writer
(`write_turtle` / `write_trig` / `write_nquads`, and `construct.rs::triples_to_ntriples`)
emits triples in **input-slice order with no canonicalising sort**, and the input slice
comes from `graph_triples(graph)` → `graph.iter_ids()` → `store.scan(&[None,None,None])`,
which walks the SPO permutation index. That index is sorted by the `[s,p,o]` **dict-id**
triple (`store.rs`: `triples.sort_unstable()` on ids). So serialisation order is dict-id
order, hence thread-count-dependent.

This is spec-permitted: Turtle / N-Triples / N-Quads / TriG / JSON-LD define no canonical
triple order. The `write_turtle` doc-comment's "Output is deterministic (subjects in
first-seen order …)" is true *as a pure function of its input slice* but is misleading
when chained through `graph_to_turtle`, because the input slice's order is itself
thread-count-dependent. No golden/snapshot/bench file in the repo compares a serialised
graph dump across runs (verified: no `.expected`/`.golden`/`insta` snapshot dumps a
serialised graph; `crates/sparq-core/tests/snapshot.rs` materialises to term strings and
**sorts** before asserting; the differential tests in `sparq-engine` canonicalise rows by
sorting term strings). So today this is latent, not broken.

### SELECT result order without ORDER BY — OBSERVABLE (unspecified)

`crates/sparq-engine/src/exec.rs::scan_to_bindings` returns rows in triple-index scan
order (`store.scan`), i.e. dict-id order, when there is no sort column. SPARQL leaves the
order of a solution sequence without `ORDER BY` unspecified, so this is conformant; the
SELECT result writers (`crates/sparq-server/src/results.rs` CSV/TSV/XML, `json.rs`) emit
rows faithfully in engine order and add no sort. `GROUP BY` output and `CONSTRUCT`/
`DESCRIBE` triple order inherit the same scan order transitively. Spec-permitted; not
pinned canonically.

### ORDER BY tie order — OBSERVABLE (unspecified), inherits the dict-id order only

`crates/sparq-engine/src/exec.rs::order_bindings`. The comparator returns
`Ordering::Equal` for rows that tie on all `ORDER BY` keys (no secondary tie-breaker).
The serial path uses `sort_by` (a **stable** sort), so ties keep their *input* order —
which is the dict-id-dependent scan order, hence thread-count-dependent. SPARQL leaves the
relative order of `ORDER BY`-tied solutions unspecified, so this is conformant.

**Correction (sq-8m65 follow-up, [OPUS-4.8]):** an earlier revision of this note claimed the
large-result parallel path uses a *non-stable* sort (`par_sort_by`), making tie order
non-deterministic *even at a fixed thread count*, as a separate cause from dict-ids. **That
is wrong.** `keyed.par_sort_by(cmp)` (`exec.rs`) is rayon's **stable** parallel sort —
rayon's `par_sort_by` is documented as "stable (i.e., does not reorder equal elements)",
an adaptive parallel merge sort; the *unstable* rayon entry point is `par_sort_unstable_by`,
which this path does **not** use (it has used `par_sort_by` since the parallel-ORDER-BY
commit `1ebbf32a`). So the parallel path preserves the `b.rows` input order of tied rows
exactly like the serial `sort_by`. At a **fixed thread count** the tie order is therefore
fully deterministic; the only residual variation is that `b.rows` is itself in dict-id /
scan order, which is thread-count-dependent — i.e. the *same* umbrella dict-id-order
property as the other surfaces above, **not** a separate parallel-sort defect. There is no
fixed-thread-count non-determinism to fix here. A future internal golden over a *tied*
`ORDER BY` result is still host-flaky for the dict-id reason (covered by the
golden-determinism contributor note), not because of an unstable sort.

### Vector-store graph fingerprint — WATCH (real robustness sharp edge, fails closed)

`crates/sparq-vectors/src/fingerprint.rs`. `Fingerprint::content_hash` deliberately folds
every term **in ascending dict-id order, binding the id explicitly** (`h.write_u32(id)`),
to detect a dict-id shift. `check_against` recomputes `Fingerprint::of(graph)` at
open/query time and hard-errors on mismatch (`store.rs`/`diskann.rs`), because a `.spqv`/
`.spqg` is keyed by dictionary term id and querying it against a shifted dict would return
silently-wrong neighbours.

- On the **persisted-store serving path** this is SAFE: `Graph::save` persists the dict in
  its built id order via `dict.save_mmap`, and `Graph::open` mmaps it back with the exact
  id→term binding, so the fingerprint recomputed on reopen matches regardless of the
  reopening host's thread count.
- The in-process `IdMask` mask-cache keyed by `(Key, Fingerprint)` (`rewrite.rs`) is
  `thread_local` and in-memory — one process, one thread count, one id assignment — so its
  cache key is internally consistent. SAFE.
- The sharp edge: if a caller builds a vector store against a graph **loaded from source
  RDF**, persists only the store, and at query time **re-loads the graph from the same RDF
  at a different thread count**, the dict-ids differ and the fingerprint mismatches even
  though the graph is logically identical → a spurious "graph fingerprint mismatch" error
  (or, if the check were ever bypassed, wrong vectors). The mechanism fails closed, so this
  is a portability/usability concern, not unsoundness.

The honest fix-shape (not implemented here) would be to make the fingerprint a function of
the *term set in a dict-id-independent order* (e.g. fold terms in lexical order, or fold an
order-independent commutative combiner over term hashes) so that the same graph fingerprints
identically at any thread count — while still catching real dict shifts (which change the
term↔position binding only because the *graph* changed, not because the thread count did).
That trades the current "any id permutation changes the hash" property for "any *graph*
change changes the hash," which is the property actually wanted. This is captured as a
follow-up bead rather than decided here, because it has a design choice (lexical-fold cost
vs. commutative-combiner collision posture) that wants the maintainer.

## Conclusion

(a) **No observable/canonical leak that matters today.** Every surface that is *required*
to be deterministic across runs/hosts — RDF canonicalisation, the ZK graph→leaves encoding,
HDT write, and conformance bag-comparison — canonicalises on the lexical term form,
independently of dict-id order. The #691 `sparq-algos` leak was the only place an
order-sensitive *algorithm output* consumed raw id order, and it is fixed.

(b) **The id order is genuinely thread-count-dependent and does surface in several
observable-but-spec-unspecified places** (serialisation order, unordered SELECT/CONSTRUCT
order, ORDER-BY-tie order). These are conformant and unpinned, so they are a latent
golden/snapshot footgun rather than a present bug. The mitigation is a *discipline*: any
future test that pins a serialised graph or an unordered/tied result must canonicalise
(sort by term) or pin `RAYON_NUM_THREADS`, exactly as the existing differential/snapshot
tests already do.

(c) **One real robustness sharp edge** in the `sparq-vectors` fingerprint, which is
id-ordered by design and can spuriously mismatch across thread counts on the
non-persisted (re-load-from-RDF) binding path. It fails closed, so it is not unsoundness,
but it is worth a fix so the same graph fingerprints identically at any thread count.

## Phased plan (future beads)

1. **Bead — make the `sparq-vectors` graph fingerprint dict-id-order-independent.**
   Replace the ascending-id fold with a dict-id-order-independent fold over the term set
   (lexical-order fold, or a commutative term-hash combiner) so the same graph fingerprints
   identically at any thread count, while still detecting a genuine graph change. Includes a
   regression test that builds the same graph at `RAYON_NUM_THREADS` 1/2/4/8 and asserts an
   identical `content_hash`, plus a test that a real dict shift (added/removed term) still
   changes it. Decide the lexical-fold-cost vs. commutative-collision trade-off with the
   maintainer. (WATCH item above.)
2. **Bead — golden/snapshot determinism guard** (sq-8qzz). **RESOLVED ([OPUS-4.8])** as the
   contributor-note option: `CONTRIBUTING.md` now carries "A golden over serialised RDF or an
   unordered/tied result must canonicalise or pin threads", codifying the discipline. No
   heavy harness/CI-lint was built — the audit verified no current test is at risk, so a
   forward-looking convention is the honest minimum.
3. **Bead (separate cause?) — stabilise large-result `ORDER BY` tie order** (sq-8m65).
   **CLOSED as not-an-issue ([OPUS-4.8]):** the premise was a misreading — `par_sort_by` is
   already rayon's **stable** sort (see the corrected ORDER-BY-tie section above), so the
   parallel path's tie order is *already* deterministic at a fixed thread count. There is no
   separate intra-host non-determinism to remove; adding a total-order tie-breaker would cost
   a term materialisation per tied row for an order SPARQL leaves unspecified. The sort site
   now carries a comment recording this.

(Phased item 1 — the `sparq-vectors` fingerprint fix — tracks as sq-xhiv (P2), handled
separately from this determinism-follow-up batch.)
4. **Bead (optional, hygiene) — fix the misleading `write_turtle` doc-comment.** Reword
   "Output is deterministic …" to scope the claim to the function's input slice and note that
   `graph_*` serialisers inherit the store's (thread-count-dependent) row order; cross-reference
   this audit.

## Open questions for the maintainer

- For bead 1: lexical-order fold (deterministic, O(dict_len log dict_len) sort cost on
  fingerprint) vs. an order-independent commutative combiner (cheaper, but needs a combiner
  with an acceptable accidental-collision posture for an integrity — not security — check).
  Which trade-off do you prefer?
- Is the non-persisted "bind a vector store to a freshly-RDF-loaded graph" path actually a
  supported usage, or is the persisted-store path the only blessed one? If the latter, bead 1
  drops to documentation-only (state the constraint and keep the cheap id-ordered fingerprint).
