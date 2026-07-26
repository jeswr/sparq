<!-- [OPUS-5] sq-jfrp0 (issue #2679) -->
# `bench/registry.d/` — per-suite benchmark registry fragments

**Add a new benchmark HERE, not by appending to
[`../benchmarks.toml`](../benchmarks.toml).**

The registry that every gate reads is the *assembled* view:

```text
bench/benchmarks.toml  +  bench/registry.d/*.toml   (sorted by filename)
```

produced by [`scripts/bench_registry.py`](../../scripts/bench_registry.py).
`benchmarks.toml` is the historical trunk; this directory is the append point.

## Why

Every bench PR used to append its `[[benchmark]]` entry to the same spot in the
same file, so two sibling bench PRs conflicted **by construction** and had to be
merged one-at-a-time. One file per suite means two PRs for different suites never
touch the same path. Same fix, same shape as
[`.github/feature-matrix.d/`](../../.github/feature-matrix.d) for the opt-in
feature matrix.

## How

1. Create `bench/registry.d/<suite>.toml` — name it after the `bench/<suite>/`
   directory (or the crate/family) the entry covers.
2. Put one or more `[[benchmark]]` entries in it, using exactly the schema
   documented in the header of [`../benchmarks.toml`](../benchmarks.toml)
   (`id` / `name` / `category` / `measures` / `invoke` / `source` / `dataset` /
   `quiet_box_sensitive` / `pinning` / `records_to` / `status`, plus the optional
   `featured` disposition).
3. Nothing else. No index to update, no generated file to re-commit.

A fragment may contain **only** `[[benchmark]]` entries (a stray `[table]` header
is rejected), every entry needs an `id`, and an `id` already claimed by the trunk
or another fragment is rejected.

## Checking your work

```sh
python3 scripts/bench_registry.py --check      # validate all three bench registries
python3 scripts/bench_registry.py --registry   # print the assembled benchmarks.toml
python3 scripts/bench_registry.py --ids        # every registered benchmark id
```

## Sibling fragment directories

| trunk | fragments | contents |
|---|---|---|
| `bench/benchmarks.toml` | `bench/registry.d/*.toml` | `[[benchmark]]` entries |
| `bench/competitors.json` | `bench/competitors.d/*.json` | `{"competitors": [...]}` |
| `bench/CATALOG.md` | `bench/catalog.d/*.md` | per-suite human notes |
