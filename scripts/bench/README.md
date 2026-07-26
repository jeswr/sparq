# scripts/bench — benchmark orchestrators + same-box gathers

## multi-axis-box.sh — bin-packed multi-axis canonical runner (sq-hmd7l.25)

ONE dedicated quiet box instead of one box per small axis (the cost discipline of
`research/comparative-benchmarking-everything.md` §4 point 5, design record PR
#1768): provisions a single `purpose=sparq-bench` c6i.4xlarge, runs the
ordered wave-1 axis list (`fts geo hdt update parse`; override/reorder with
`AXES=`) **strictly serially** — one axis, and therefore one engine, active at a
time — then the box self-terminates.

- **Modes:** `--dry-run` prints the packed execution plan (axes + harness
  presence, caps, budget arithmetic, orphan-proofing summary) with **no AWS
  call**; `AWS_PROFILE=pss ... --launch [<branch>]` provisions for real;
  `--instance` is the on-box entrypoint (contains **no** shutdown/terminate
  call — self-termination lives only in the launcher-generated user-data).
- **Orphan-proof** (per the standing EC2 rules):
  `--instance-initiated-shutdown-behavior terminate`, a FIRST-LINE user-data
  watchdog (3h hard cap default) + `systemd-run` backup, launcher poll deadline
  **below** the watchdog, EXIT-trap terminate + ephemeral keypair/SG teardown,
  the prod/dev instance ids refused by id, and a post-launch
  `scripts/orphan-check-bench.sh` dry-run that must come back clean.
- **Results, both channels:** per-axis `=== SPARQ_BENCH_RESULT <axis> ===`
  console blocks (compact envelope JSON under a per-envelope byte cap for the
  ~64KB serial buffer, plus a provenance line: instance id/type, commit, UTC,
  canonical flag) inside one outer marker range, **and** an incremental SSH pull
  of the full envelopes from `/root/axis-results/` into `RESULTS_LOCAL`.
- **Discipline:** `df` floor check + scratch cleanup between axes; dataset caps
  (`PARSE_GEN_N`, per-axis `AXIS_ENV_<axis>` env passthrough); an axis whose
  harness has not landed on the checked-out branch is skipped with an honest
  `absent` status (bead reference printed), never a fabricated row. The parse
  axis emits raw harness rows until its envelope wrapper lands (sq-hmd7l.6).

## shacl-same-box.sh — SHACL competitor comparison (sq-7d3dj.33)

Same-box SHACL validation comparison — **sparq-shacl vs pySHACL vs Apache Jena
SHACL** — over the shared `bench/shacl/` workloads (the 5 committed gate shapes
+ the SPARQL-constraint-heavy `bench/shacl/shapes-sparql/` set) at LUBM scales
(default `univ=1` ~103k and `univ=10` ~1.3M triples). All three engines are
timed **in-process, validate-only, best-of-N on a loaded graph** (drivers:
`pyshacl-shacl-bench.py`, `JenaShaclBench.java`; sparq uses
`examples/bench_shacl`), with per-workload timeouts recorded as honest `ERROR`
rows and per-workload `#violations`/`conforms` cross-checked engine-vs-engine.
Emits one `bench/canonical-competitor-results/`-shaped envelope JSON per scale;
`canonical:false` unless `CANONICAL=1` (dedicated quiet box). Engine deps are
gather-only `/tmp` scratch (pip venv + Jena tarball) — clean with
`rm -rf /tmp/jena-shacl /tmp/shacl-bench-venv`. First-read + root-cause:
[`research/shacl-baseline-2026-07.md`](../../research/shacl-baseline-2026-07.md).

## materialize-same-box.sh — reasoning/materialization competitor comparison (sq-hmd7l.7)

Same-box **deductive-closure (materialization)** comparison — **sparq `reason`
(OWL-RL / RDFS) vs Apache Jena rule reasoners vs VLog vs Nemo** — computing the
SAME closure over the SAME LUBM `(ABox + TBox)` N-Triples (`bench/lubm/gen.sh`,
READ-ONLY; `bench/lubm/run.sh` is not touched) at LUBM scales (default `univ=1`
~103k and `univ=10` ~1.3M input triples). The `univ≥100` canonical EC2 run
belongs to the canonical wave (sq-hmd7l.26) — this harness does **not** launch
EC2.

**Oracle = closure-size cross-check.** sparq's `reason` self-reports its closure
count; the harness asserts it against a pinned per-scale/per-profile expected at
`univ=1` (`owl=150589`, `rdfs=126732`) and records every engine's closure size
in the envelope's `count_crosscheck`. INVARIANT: no throughput row without a
closure-count agreement **or** an explicitly-recorded profile-difference caveat.

**Profile / rule-set fidelity is recorded per column, never absorbed.** The
compared "closure" is only meaningful if the rule set matches, and it does not
across engines: sparq `reason owl` is the **full W3C OWL 2 RL/RDF** rule table
(`crates/sparq-reason`); Jena has **no** full OWL 2 RL reasoner (its
`OWL_MICRO`/`OWL_MINI`/RDFS rule reasoners are OWL-subset + add axiomatic triples
+ de-dup the ABox on load, so the closure size differs **by construction**); VLog
and Nemo are **general Datalog** engines that need a separately-validated OWL-RL
Datalog encoding (`.dlog`/`.rls`) reproducing sparq's closure. Absent a validated
encoding (or a binary on PATH), the VLog/Nemo columns emit an honest
`NOT-RUN-LOCALLY` with the exact blocker — never a fabricated number.

Timed figure: sparq's self-reported materialize time (parse excluded); Jena's
in-process `InfModel` materialize best-of-N (JVM start-up + parse outside the
timed section; drivers `scripts/bench-adapters/jena_reason_adapter.java`,
`vlog_adapter.py`, `nemo_adapter.py`). Emits one
`bench/canonical-competitor-results/`-shaped envelope per scale;
`canonical:false` unless `CANONICAL=1` (dedicated quiet box). Gather-only Jena
tarball lives in `/tmp/jena-reason` — clean with `rm -rf /tmp/jena-reason`.
Acceptance: `ONLY=sparq LUBM_UNIVS=1 scripts/bench/materialize-same-box.sh` exits
0, asserts the pinned closure counts, emits a well-formed envelope.

## run-all-benchmarks.sh — whole-estate orchestrator

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

## Canonical competitor gather (dedicated quiet EC2 box)

The canonical 5-engine competitor matrices under
`bench/canonical-competitor-results/<date>/` are produced by a **dedicated quiet
c6i.4xlarge** (one engine active at a time, same corpus + query files, counts
cross-checked before any timing is trusted). The committed harness
([FABLE-5] sq-7d3dj.34):

- `canonical-competitor-bench.sh` — the orphan-proof EC2 **launcher**
  (`AWS_PROFILE=pss scripts/bench/canonical-competitor-bench.sh <branch>`):
  ephemeral keypair/SG, `--instance-initiated-shutdown-behavior terminate`, a
  user-data self-shutdown watchdog **below** which the sentinel-gated poll
  deadline sits, incremental result pull, explicit terminate + teardown on exit.
- `canonical-http-gather-instance.sh` — the **instance-side** HTTP/TTFB panel:
  all five engines in the SAME HTTP regime — **sparq-server** itself,
  `oxigraph serve-read-only`, Fuseki via the **offline `tdb2.tdbloader` → 
  `fuseki-server --tdb2`** intended bulk path (the fix for the 2026-07-07
  docker-image load hang), Virtuoso, QLever — measuring **full-request latency
  AND TTFB** in **both keep-alive and fresh-connect** regimes
  (`http_sparql_adapter.py --profile`, 6-col TSVs).
- `canonical-beir-bench.sh` / `canonical-beir-gather-instance.sh` — the **BEIR
  IR-quality gather pair** ([FABLE-5] sq-tvzyi), same orphan-proof launcher rails:
  the instance side carries the heavy pip `pyserini`+`beir` provisioning (venv,
  JDK 21) that the wave-1 bin-packed box did not, builds the sparq-text
  `beir_text` example, and runs `scripts/gather-competitors.sh --run --only
  lucene-anserini` per BEIR cut so sparq-text and the Lucene/Anserini kernel BM25
  oracle are scored by the ONE shared scorer on the SAME cut + qrels
  (Recall@100/nDCG@10 deficits). Envelopes land in `bench/competitor-results/` +
  a `gather-meta.json` provenance record; reviewed deficits are vendored into
  `research/gap-fts-2026-07.md` §4 deliberately (the BEIR corpus is not
  redistributable in-repo, so this stays gather-only).
- `emit_envelope.py` — folds per-engine TSVs + `meta.json` into one canonical
  envelope per suite (3-col and 6-col aware).
- `ingest-canonical-competitors.mjs` — envelopes → the dashboard's
  `same_box_comparisons` (asserts cross-gather count stability; carries the
  keep-alive-vs-fresh `connection` note + `values_ttfb`/`values_fresh` columns).

## bench-s3-results.sh + s3-results-uploader.sh — durable results channel (sq-ffaa9)

A **third, termination-independent** way to get results off a canonical bench box.
The two pre-existing channels can both fail on the same run, and on the x86_64
`c6i` (Nitro) class they routinely do:

- `aws ec2 get-console-output` — the Nitro serial console does not reliably expose
  userspace `/dev/console` writes, the ring buffer wraps under a chatty build, and
  it is wiped on terminate. Recovered text comes back truncated, garbled or empty.
- SSH/scp pull — dies exactly when a heavy gather saturates the box, and is gone
  the moment the watchdog self-terminates it.

When both fail the run is **unretrievable** and the compute spend is wasted. With
this channel the box streams envelopes + logs to S3 as it runs, and the launcher
reads them back **after** the instance is gone.

**Off by default.** Nothing changes until `SPARQ_BENCH_S3_BUCKET` is exported: the
`--iam-instance-profile` argument is empty, the user-data uploader line renders to
the literal `true`, and `multi-axis-box.sh` / `canonical-{beir,competitor,materialize}-bench.sh`
behave byte-for-byte as before.

**One-time maintainer step (needs IAM + S3 admin rights).** Preview it with no AWS
call at all, then apply:

```sh
scripts/bench/bench-s3-results.sh plan            # policy JSON + the exact API calls
AWS_PROFILE=<admin> scripts/bench/bench-s3-results.sh provision
AWS_PROFILE=pss     scripts/bench/bench-s3-results.sh check
export SPARQ_BENCH_S3_BUCKET=sparq-bench-results-<account-id>
```

`provision` is idempotent, and verifies that the instance profile really does carry
the upload role before reporting success — an `add-role-to-instance-profile` failure
is benign when the role is already attached but fatal when the profile holds a
different one, so it is checked rather than assumed. It is also the **sole
convergence authority for the role's trust policy**: `create-role` is skipped when a
same-named role already exists, so provision issues `update-assume-role-policy` on
that existing role rather than inheriting whatever trust document it came with. A
role trusting the wrong service (or nothing) is attachable and looks healthy in
every existence check, yet EC2 cannot assume it — the box boots with no role
credentials and every upload fails. `check` reports READY only on actual role
attachment **and** on a trust policy that lets `ec2.amazonaws.com` call
`sts:AssumeRole`, never on the profile or role merely existing. It also prints
the **`iam:PassRole` policy the launcher principal separately needs** — attaching an instance profile requires it, and
without it `run-instances` fails `AccessDenied`.

**The instance role is write-only, deliberately:** `s3:PutObject` +
`s3:AbortMultipartUpload` scoped to `arn:aws:s3:::<bucket>/*`, with no
`GetObject`/`ListBucket`/`DeleteObject` and no bucket-level action. A compromised
bench box can append its own objects and nothing else. The uploader is written
against exactly that grant — it uses `aws s3 cp`, never `aws s3 sync` (which needs
`ListBucket`+`GetObject` to diff and would fail closed).
`scripts/tests/test_bench_s3_results.sh` pins that coupling, plus the
off-by-default behaviour and the dollar-free user-data line the launchers'
unquoted heredocs require.

Results land under `s3://<bucket>/<run-id>/` alongside a `_provenance.txt`
(instance id/type, arch, commit) and, on a clean finish, `_UPLOAD_COMPLETE.txt` —
so a partial upload is distinguishable from a completed one. `bench_s3_fetch`
pulls them into `RESULTS_LOCAL/s3/`, and the launchers call it **before** the SSH
tar so a dead sshd cannot skip it.

**Degradation is always toward launching, never toward blocking:** an instance
profile that is missing, unreadable, or that does not carry the upload role logs a
warning and launches without the channel rather than failing an expensive run over
an IAM read grant. The uploader is a best-effort
side-car — it does not run under `set -e` and always exits 0, so a telemetry
failure can never abort a gather.
