# scripts/bench — run-all-benchmarks orchestrator

`run-all-benchmarks.sh` (bead sq-hz0g2) runs the **whole benchmark estate** with
per-suite isolation and **streams results incrementally to a local folder as each
suite completes** — so a session that dies mid-run (usage limit, shutdown, Ctrl-C)
loses at most the in-flight suite. Everything already finished is on disk for the
next session.

## Usage

```sh
scripts/bench/run-all-benchmarks.sh --list          # print the suite catalog
scripts/bench/run-all-benchmarks.sh --dry-run       # what would run/skip right now
scripts/bench/run-all-benchmarks.sh                 # fast + standard tiers
scripts/bench/run-all-benchmarks.sh --tier heavy    # everything runnable locally
scripts/bench/run-all-benchmarks.sh --only fts,rsp-oracle
scripts/bench/run-all-benchmarks.sh --remote        # EC2 plan DRY-RUN (see below)
```

Results land in `~/sparq-bench-results/<UTC-timestamp>-<git-sha>/`:

- `manifest.json` — host, commit, toolchain, start/end, per-suite status table;
  **re-written atomically after every suite**.
- `suites/<id>.json` + `suites/<id>.md` + `suites/<id>.log` — one result per suite
  (machine-readable, human summary, full output). `suites/<id>.d/` holds any extra
  artifacts a suite emits (e.g. the ci-bench JSON).

## Catalog and skip discipline

The catalog (in the script; `--list`) maps each suite to its
[`bench/benchmarks.toml`](../../bench/benchmarks.toml) registry id(s) —
see [`bench/CATALOG.md`](../../bench/CATALOG.md) for the human guide. Suites whose
dependency is missing (QLever, EYE, nargo/bb, a GPU, olympics data, an LLM agent,
an EC2 budget) are **skipped with the reason recorded in the manifest**, never
silently dropped. One red suite never kills the run; each suite has its own
timeout, a `df` disk-floor gate, and `/tmp` scratch that is cleaned afterwards.

## Honesty

Every result file is stamped **NON-CANONICAL**: this work box is shared and
frequently busy, so wall-clock numbers are trend-only (QUIET-BOX convention,
`bench/CATALOG.md`). The deterministic gates (counts, gate counts, bytes,
pass-rates) are load-robust and remain meaningful.

## EC2 mode — prepared, not launched

`--remote` prints the launch plan and exits (the EC2 quota is currently not
fixed). A real launch additionally requires `EXECUTE=1` and follows the repo's
EC2 bench protocol: `purpose=sparq-bench` tag, orphan-proof
`--instance-initiated-shutdown-behavior terminate` + a user-data shutdown
watchdog + remote self-shutdown on completion, ephemeral keypair/security-group,
and it only ever operates on the instance ids it creates (never prod/dev boxes).
Results are rsync-streamed back into the same local folder **per suite** while
the remote run progresses. The launch path is prepared but has NOT been executed
yet — validate it on first use when the quota returns.
