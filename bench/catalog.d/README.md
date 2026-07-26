<!-- [OPUS-5] sq-jfrp0 (issue #2679) -->
# `bench/catalog.d/` — per-suite catalog notes

**Write a new suite's "needs care" note HERE, not by appending a bullet to
[`../CATALOG.md`](../CATALOG.md).**

The full human catalog is the *assembled* view:

```text
bench/CATALOG.md  +  bench/catalog.d/*.md   (sorted by filename)
```

produced by `python3 scripts/bench_registry.py --catalog`. `CATALOG.md` keeps the
conventions, the category map, and the replicate-everything quickstart; this
directory holds the per-suite prose that used to pile into its "Notes on a few
that need care" list — the shared append point that made sibling bench PRs
conflict (see [`../registry.d/README.md`](../registry.d/README.md)).

## How

Create `bench/catalog.d/<suite>.md`. It must lead with a markdown heading (use
`## <suite>`) so the assembled document stays navigable and two fragments can
never merge into one section. Everything else is ordinary prose:

```markdown
## example-suite

What it measures, which regime it runs in, what is pinned, and any honesty
caveat (quiet-box sensitivity, external-cost tier, gather-only competitor).
Link the harness README: [`../example/README.md`](../example/README.md).
```

Keep the machine-readable facts (exact invocation, dataset, pinning) in the
registry fragment under [`../registry.d/`](../registry.d) — this file is the
human explanation, not a second source of truth.

## Checking your work

```sh
python3 scripts/bench_registry.py --check     # validate all three bench registries
python3 scripts/bench_registry.py --catalog   # print the assembled catalog
```
