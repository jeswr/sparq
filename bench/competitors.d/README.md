<!-- [OPUS-5] sq-jfrp0 (issue #2679) -->
# `bench/competitors.d/` — per-competitor registry fragments

**Register a new competitor engine HERE, not by appending to the `competitors`
array in [`../competitors.json`](../competitors.json).**

The competitor registry consumers read is the *assembled* view:

```text
bench/competitors.json  +  bench/competitors.d/*.json   (sorted by filename)
```

produced by [`scripts/bench_registry.py`](../../scripts/bench_registry.py).
`scripts/gather-competitors.sh` assembles it automatically when this directory
has fragments.

## Why

Two bench PRs each adding a competitor appended to the same JSON array and
conflicted. One file per competitor removes the shared append point — see
[`../registry.d/README.md`](../registry.d/README.md) for the full rationale.

## How

Create `bench/competitors.d/<id>.json` containing a single `competitors` array
whose entries use exactly the trunk's entry schema (so an entry can be moved
between trunk and fragment verbatim):

```json
{
  "competitors": [
    {
      "id": "example-engine",
      "name": "Example Engine",
      "kind": "http-sparql",
      "role": "what it is a reference for",
      "pinned_version": "1.2.3",
      "version_source": "where that pin comes from",
      "install_recipe": "how to install it",
      "run_recipe": "how to run it",
      "comparable_suites": [{ "sparq_suite": "sp2b", "note": "…" }],
      "version_env_metadata": "what the gather records",
      "quiet_box_sensitive": true,
      "dashboard_engine_id": "example-engine"
    }
  ]
}
```

`schema_version`, `results_layout`, `engines`, and `values` stay in the trunk —
a fragment carrying any key other than `competitors` is rejected, as is a
duplicate competitor `id`.

## Checking your work

```sh
python3 scripts/bench_registry.py --check         # validate all three bench registries
python3 scripts/bench_registry.py --competitors   # print the assembled competitors.json
```
