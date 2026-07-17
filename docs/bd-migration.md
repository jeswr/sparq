# Beads to issues migration (2026-07-17)

This repo's task tracker was migrated from **beads** (`bd`, the former
`.beads/issues.jsonl` source-of-record) to **GitHub issues** on 2026-07-17.
GitHub issues are now the sole tracker; `.beads/` has been removed from the tree
(its full history is preserved in git). See `AGENTS.md` → *Task tracking* for the
live workflow.

## Resolving a `sq-…` id after the cutover

Historical PR titles, commit messages, `research/` records, and code comments
reference beads by their `sq-XXXX` (or dotted `sq-XXXX.NN`) ids. Each migrated
bead became a GitHub issue that carries a `<!-- bd-id:sq-XXXX -->` marker in its
body, and the mapping is committed durably here:

- **[`bd-migration-map.json`](./bd-migration-map.json)** — a flat
  `{"sq-XXXX": <issue-number>}` object (254 migrated beads).

To resolve an id:

```sh
# sq-id -> issue number, from the committed map
jq -r '."sq-7d3dj"' docs/bd-migration-map.json        # -> e.g. 2600

# or straight from GitHub (authoritative; the map is a snapshot of this):
gh issue list -R sparq-org/sparq --state all --limit 2000 --json number,body \
  | jq -r '.[] | select((.body//"") | test("<!-- bd-id:sq-7d3dj -->")) | .number'
```

A `sq-id` **not** present in the map was either closed before the migration or
never migrated (the migration moved open/in-progress beads only); consult
`git log` for its resolution.

## The migration tool

`scripts/bd-to-issues.py` (retained as the historical migration tool, with a
header note dating the run) parsed `bd export`, mapped bd labels to the issue
label taxonomy, and created the issues idempotently keyed on the `bd-id` marker.
It is no longer part of any live workflow.
