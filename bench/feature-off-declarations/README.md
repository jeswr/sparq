<!-- [OPUS-4.8] sq-v3nel-v2 (2026-07-07): per-PR feature-OFF change declarations. -->

# feature-OFF wasm-bundle change declarations (mechanism V2)

The `vectorized-feature-off` gate (leg 2,
`.github/workflows/vectorized-feature-off.yml`) builds the feature-OFF `sparq-wasm`
bundle from the **base** tree and the **head** tree in the same CI run and compares
them **byte-for-byte**. Identical bytes pass automatically. If the bytes differ, the
change must be **declared** — otherwise the gate fails (the whole point: an accidental
`vectorized`/default-path leak into the feature-OFF build is caught).

## How to declare

If your PR *intentionally* changes always-compiled engine/core code in a way that moves
the feature-OFF bundle bytes, add **one file** named after your PR number:

```
bench/feature-off-declarations/<PR-number>.json
```

with the shape:

```json
{
  "pr": 1234,
  "date": "2026-07-07",
  "reason": "one line on what always-compiled change moves the feature-OFF bytes"
}
```

The gate is satisfied when the head tree's declarations directory contains at least one
`<digits>.json` (or `.md`) file the base tree's does **not** — a set difference on the
directory listing. Only names matching `<digits>.json|md` count as declarations, so this
`README.md` (and any `.gitkeep`) is ignored.

## Why per-PR files (V2)

The previous mechanism stored one scalar `change_token` in
`bench/feature-off-declaration.json`. Every declaring PR edited the **same line**, so the
first declared PR to merge made every other declared PR textually **CONFLICTING** in git
(`#1720` and `#1718` both went `DIRTY` after `#1726`'s declaration merged). The gate check
was order-independent but the file was not. Per-PR files remove the shared line: different
PRs add different files, so git never conflicts regardless of merge order.

The legacy scalar file `bench/feature-off-declaration.json` is **retired** (frozen, kept
parseable). During the transition window the gate still accepts a scalar-token inequality
so in-flight pre-V2 branches keep working; new PRs must use a per-PR file here instead.

## Scope

This declaration governs **intent** (did you mean to change the feature-OFF bytes at all?).
The **size** of an accepted change is governed separately, unchanged, by the
`metrics.wasm_bundle_bytes` floor ratchet (±2% band) in `bench/perf-baseline.json` /
`bench.yml`. A change moving the bundle > ±2% must also raise that floor.
