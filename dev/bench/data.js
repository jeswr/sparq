window.BENCHMARK_DATA = {
  "lastUpdate": 1781453904533,
  "repoUrl": "https://github.com/jeswr/sparq",
  "entries": {
    "sparq engine": [
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "885e3f4bbe03a0c88874e7f25f2c8935ecceb626",
          "message": "merge: fix CI (conformance --bin) + Benchmarks (benchmark-data bootstrap) [OPUS-4.8]\n\n(A) ci.yml conformance step 'cargo run -p sparq-conformance' was AMBIGUOUS since commit 058bdbf5 added a 2nd bin -> exit 101 before any test (NOT a conformance regression). Fix: --bin sparq-conformance + default-run in sparq-conformance/Cargo.toml. Verified locally: 1225 pass + 4 div = 1229 = ratchet PASS. (B) bench.yml/bench-ec2.yml github-action-benchmark failed on missing benchmark-data branch -> idempotent self-healing orphan-branch bootstrap (no manual setup). js/python untouched. codex retroactive (rate-limited).\nModel: claude-opus-4-8\nProvenance: Opus 4.8 (Fable unavailable) — re-review/upgrade candidate\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-13T15:47:55Z",
          "tree_id": "71e2a0c8d3fcd9c3855e653bb57536284d80eed0",
          "url": "https://github.com/jeswr/sparq/commit/885e3f4bbe03a0c88874e7f25f2c8935ecceb626"
        },
        "date": 1781365821505,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.543,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3251.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4883.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 824.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13063.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 61168.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 167769.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 5289.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42160.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7362.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 59354.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 152461.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4626,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37484.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.148,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1519238,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "c30c40ad654261642fb02c7b5a48d73e2a21105c",
          "message": "docs: codify parsing plan + 4 skills + benchmark catalog [OPUS-4.8]\n\n- research/parsing-optimization-plan.md: measure-first HDT + Turtle parse plan (honest: Turtle cannot exactly match NT; HDT H1/H3/H4 levers).\n- .claude/skills/{rust-parallel-parsing,fused-decompress-parse,hdt-format,mpc-protocols}: 4 skills grounded in this project's measured research, mapped to active crates.\n- bench/{benchmarks.toml,CATALOG.md}: codified registry (41 benchmarks / 9 categories) + reproduction conventions. Gaps flagged: no SHACL/MPC/vector perf bench, GenAI planning-only, no public SP2Bench/WatDiv/WDBench.\ncodex rate-limited -> retroactive review.\nModel: claude-opus-4-8\nProvenance: Opus 4.8 (Fable unavailable) — re-review/upgrade candidate\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-13T15:57:01Z",
          "tree_id": "275c9610683727737c62da12cf55ceb111b2b4c5",
          "url": "https://github.com/jeswr/sparq/commit/c30c40ad654261642fb02c7b5a48d73e2a21105c"
        },
        "date": 1781366307551,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.43,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 2531.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 3764,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 3.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 637.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 10238.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 49641.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 129977.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2158,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 34617.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 6163.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 49604.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 125164.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 1914.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 32467.6,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.119,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1519238,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "2bb733b4945cfae285323bb2fc3fcc557134e7b6",
          "message": "merge: HDT direct SPO decoder — skip thrown-away TriplesBitmap structures [OPUS-4.8]\n\nParsing goal (HDT): new sparq-side graph_from_reader does a one-shot SPO scan reading bitmap_y/z + sequence_y/z directly, NEVER calling hdt::TriplesBitmap::new -> avoids the wavelet matrix, per-object Vec mega-alloc, cache-hostile sort, OP-index, and Rank9Sel-over-bitmaps (H1/H2). Block-sequential PFC dict decode reusing one buffer (H3); intern from borrowed slices (H4). .hdt.gz works; internal PFC/Log64 decoded; CRCs verified. Public API behaviorally identical (upstream path kept as in-process oracle). DIFFERENTIAL CORRECTNESS: identical BTreeSet<[String;3]> + store.len + dict.len vs upstream on real (snikmeta 328) + multiblock (802) + gzip + edge archives; 6 pre-existing tests pass; CLI --features hdt green. Provisional CONTENDED throughput 2.6x@1M / 3.6x@2M (gap widens with scale) -> re-measure headline on a quiet EC2 box. Deferred (Wave B/C): zstd/bzip2 sniffing, parallel dict decode, CRC-skip. codex retroactive (rate-limited).\nModel: claude-opus-4-8\nProvenance: Opus 4.8 (Fable unavailable) — re-review/upgrade candidate\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-13T16:08:00Z",
          "tree_id": "6e6487f2daafcd75f94976d98fa2a50be00eef3a",
          "url": "https://github.com/jeswr/sparq/commit/2bb733b4945cfae285323bb2fc3fcc557134e7b6"
        },
        "date": 1781366979114,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.539,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3022.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4453.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 762.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12521.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56169.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 155600,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4329.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 43853.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8027.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58660.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 151979.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3688.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38577.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1519238,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "71052bf9a3ab07fa9b8700aa1e1fc62a12b1247d",
          "message": "merge: Turtle parse incremental wins — memchr pre-scan + per-chunk directive snapshot [OPUS-4.8]\n\nParsing goal (Turtle): T2 memchr3 terminator pre-scan (6 top-level bytes; structured handlers byte-identical, #1398 fix preserved; memchr gated behind parallel, no wasm growth). T3 per-chunk directive snapshot kills the interspersed @prefix/@base serial-fallback cliff. T1 (slice-interning) MEASURE-FIRST REJECTED: only 1.05x serial-leg (oxttl allocates its own Term; structural win needs a non-existent borrowing API / rejected custom grammar). Differential chunked==serial incl malformed-rejection parity; 38 sparq-core tests pass; conformance UNCHANGED (SPARQL 1229, inference 1654, n3/turtle 297/0). Provisional CONTENDED 3.11x scaling -> re-measure on quiet EC2. HONEST: exact NT parity infeasible (prefixed-name expansion + multi-line + oxttl Term alloc). codex retroactive.\nModel: claude-opus-4-8\nProvenance: Opus 4.8 (Fable unavailable) — re-review/upgrade candidate\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-13T16:20:04Z",
          "tree_id": "242bd7f3f2e338816e84c6b6a86b15d9af5002b3",
          "url": "https://github.com/jeswr/sparq/commit/71052bf9a3ab07fa9b8700aa1e1fc62a12b1247d"
        },
        "date": 1781367714113,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.54,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3255.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4711.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 818.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13194.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 62857.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 167097.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 5255.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 43847.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7545.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 61805,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 161433.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3323.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39295.1,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1519238,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "fbbf0eb9a71dffdb786453ab3670594162f8bdba",
          "message": "merge: ZK revocation #12 — issuer-bound status ref + external authoritative snapshot [OPUS-4.8]\n\nAudit #12 CLOSED — COMPLETES ALL 12 AUDIT ISSUES. Issuer signature binds the status-list reference (list IRI, index, version); the liveness BIT is read from an EXTERNAL relying-party authoritative snapshot (RevocationPolicy, mirroring the #3 external-K precedent) — never the prover's bytes. Rejects forged all-zero un-revoke (CredentialRevoked), omitted/mismatched reference, stale (StatusListStale), missing authoritative snapshot (StatusSnapshotMissing, fail-closed); prover-snapshot tamper is an isolated tripwire. Two interim Claude re-audits: round1 found a real hole (unauthenticated bits) -> fixed; round2 HOLD (5 lenses, no bypass). Real bb prove+verify forge tests pass. codex DEFERRED (cooldown; retroactive, task #11). Remaining = documented PRIVACY upgrades only (#3 in-circuit undisclosed-key, #12 in-circuit hidden-index), NOT soundness gaps.\nModel: claude-opus-4-8\nProvenance: Opus 4.8 (Fable unavailable) — re-review/upgrade candidate\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-13T16:29:12Z",
          "tree_id": "8c1394fe3478973fac2237d933a0ee957a46a7ea",
          "url": "https://github.com/jeswr/sparq/commit/fbbf0eb9a71dffdb786453ab3670594162f8bdba"
        },
        "date": 1781368257019,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.545,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3014.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4403.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 749.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12855.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56325.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 151259,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3869,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 39458.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8112.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57467.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 152515,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2766,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39247.4,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1519238,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "ba9c281a3e39b2ed6fda47ef6d25c5b6b4d3c06b",
          "message": "feat(bench): orphan-safe EC2 parallel benchmark harness [OPUS-4.8]\n\nbench/ec2-bench.sh: SSH-less console-output-based dev-box harness for running benchmarks on parallel quiet EC2 instances. Orphan-proof (watchdog first line, default 45-min cap, instance-initiated-shutdown=terminate, tag purpose=sparq-bench). Reads invoke from bench/benchmarks.toml by id. Subcommands launch/wait-result/result/terminate/orphan-check/sweep. Validated (3 throwaways self-terminated, $0.03). Console snapshots only after ~8min uptime -> writes both /dev/console+ttyS0, dwells 600s, wait-result polls 18min. Re-login pss for sustained campaigns. See memory feedback-ec2-benchmarks.\nModel: claude-opus-4-8\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-13T16:31:58Z",
          "tree_id": "0286ca51900a55e5d8662fc3c2d0d3e2e96886d1",
          "url": "https://github.com/jeswr/sparq/commit/ba9c281a3e39b2ed6fda47ef6d25c5b6b4d3c06b"
        },
        "date": 1781368425572,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.534,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3017.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4413.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 760.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12661.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 57156,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 155363,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4586.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 39504.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7980.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57094.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 151769.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3981.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38299.5,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.14,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1519238,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "dde4543788bce62153cec780e0311f350c72084c",
          "message": "merge: MPC M2 — disclosed-key global-IRI join [OPUS-4.8]\n\nMPC build M2 (fork-independent): DisclosedKeyJoin implements GlobalJoin::join for key_disclosed — holders evaluate_local over their OWN graphs (data stays local beyond the disclosed projection), joined crypto-free on the shared global-IRI key OUTSIDE the crypto core (arch conv #4); invariant to Q1/Q2. Faithful PAG compatible-mappings inner join over all shared columns; planner-UNTRUSTED (verifies join var projected + compatibility on every shared var). DIFFERENTIAL: federated join == single-store union eval (2/3-holder chains, fanout, empty/single/error); 13/0; wasm excludes sparq-mpc. Hidden-value/PSI join DEFERRED (Q2/M3); M3 backend (Q2), M4 collaborative proof (Q1 + the now-complete M1 ZK foundation) follow. codex retroactive.\nModel: claude-opus-4-8\nProvenance: Opus 4.8 (Fable unavailable) — re-review/upgrade candidate\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-13T16:43:40Z",
          "tree_id": "dc34c7b9bc642ac7dfd3fa805434c3580f72e52e",
          "url": "https://github.com/jeswr/sparq/commit/dde4543788bce62153cec780e0311f350c72084c"
        },
        "date": 1781369130516,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.533,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3019.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4419,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 759.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12386.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55821.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 152078.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2602.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 39949.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8066.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58888,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 152283.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2851.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37816.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1519238,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "735ed6c77a215cf3b2585ffdcb97c30ab0280370",
          "message": "docs(research): M4 distributed-sig feasibility spike — verdict + M4 v1 path [OPUS-4.8]\n\nQ1 spike: in-circuit distributed-signature-over-secret-shared-witness is genuinely novel/UNSOLVED (no published instance composes sig-verify over a secret-shared message under a hidden key in a collaborative proof). M4 v1 = verifier-side-attestation interim (ZK-#3 lifted to N sources; correctness + attested-to-K; gives up source-unlinkability + single-proof binding; honest-majority). Smallest-first future in-circuit step = federate only the correctness relation over secret-shared data. CRITICAL caveat: eprint 2025/1026 (coZK leaks honest inputs on invalid witness; validate extended witness before proving; honest-majority-only). M4 build gated on M3.\nModel: claude-opus-4-8\nCo-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
          "timestamp": "2026-06-13T17:24:18Z",
          "tree_id": "f766306aa13ba17d8aea169cf7d715acd7b3f27c",
          "url": "https://github.com/jeswr/sparq/commit/735ed6c77a215cf3b2585ffdcb97c30ab0280370"
        },
        "date": 1781371563120,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.54,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3258,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4731.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 812.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12968.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 60595.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 158270.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4008.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40462,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7336.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58070.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 153939.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2949.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 36737.7,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1519238,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "4aea49afae09eff9feb80cfff7b30b92e531bb41",
          "message": "chore(serve): remove serve-wave-b agent STATUS scratch (merged) [OPUS-4.8]\n\nserve-wave-b is merged (ad10bfc): no-HoL SRPT+aging read scheduler with\nreserved-cheap-worker. Quiet-box numbers confirmed (HoL ~207x improvement,\n~628ns/job overhead) and the signal-before-count completion race is fixed\n(fc245fb) — proven gone over 350 stress iterations. STATUS.md was agent\nscratch; removing now that the work landed.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T17:39:45Z",
          "tree_id": "32e4c24c7da42e50a071c2a759624f8cbbdfb4c1",
          "url": "https://github.com/jeswr/sparq/commit/4aea49afae09eff9feb80cfff7b30b92e531bb41"
        },
        "date": 1781375979938,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.587,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3114.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4912.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 811.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 16139.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 72334,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 193617.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3157.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 52061.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9427.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 70229.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 169693.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2950.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 45340.5,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.183,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1519238,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "f708c47b55ff537323313c32e81d729b99d1fb77",
          "message": "merge: MPC M3 — honest-majority Shamir backend + hidden-value PSI join [OPUS-4.8]\n\nM3 completes the secret-sharing backend the disclosed-key M2 deferred:\n- honest-majority Shamir t-of-n MpcBackend (F_{2^61-1}) + secure cumulative sum\n- hidden-value join via secret-shared equality-to-zero (circuit-PSI core)\n- Q2 resolved: trust model selected by the MpcBackend trait (honest-majority v1,\n  dishonest-majority a future backend), configurability documented in PLAN.md\n- randomized stress test for secret-shared equality\n\nRe-audited HOLD verdict = MERGE: YES (no bypass on all four lenses). Two minor\nnon-blocking follow-ups noted for later: reject n<3 explicitly; document the\nsecure-sum range assumption. Only touches crates/sparq-mpc (no overlap with the\nserve-wave-b scheduler work already on main).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T18:39:32Z",
          "tree_id": "d9cab7c035bf0b790a79322f9488bce7eac38286",
          "url": "https://github.com/jeswr/sparq/commit/f708c47b55ff537323313c32e81d729b99d1fb77"
        },
        "date": 1781377000485,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.595,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3101.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4875.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 854,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 16279.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 71749.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 190308,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2897.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 51817,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9860.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 69173.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 169898.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2810.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 46030.2,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.183,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1519238,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "0b9e785e2784f9db4c4c75cccba834ace06b8736",
          "message": "merge: clippy hard-gate + zero-warnings onto main (serve-wave-b + M3) [OPUS-4.8]\n\nBrings the reviewed lint work (3 clippy-fix commits + CI hard-gate + rustfmt.toml,\ncodex jobs 2254/2255 clean) onto main, which now also carries MPC M3. The lint\nbranch was cut before M3 landed, so M3's new code (sparq-mpc shamir.rs/field.rs/\njoin.rs) is linted for the first time by this merge — verified clippy-clean under\n-D warnings as a follow-up before push.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T18:55:58Z",
          "tree_id": "8eb0baa505d6d014567133096be1e2c49dcd7a6e",
          "url": "https://github.com/jeswr/sparq/commit/0b9e785e2784f9db4c4c75cccba834ace06b8736"
        },
        "date": 1781377204949,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.525,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3139.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4390.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 753.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12939.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56011.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 152135.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4545.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40186.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7490.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55699.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 146404.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3397.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39779.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.145,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "810f783778e52483cd6294a40a56692df4157f9a",
          "message": "feat(tracking): adopt beads (bd) dependency-aware task tracking + populate backlog [OPUS-4.8]\n\nStands up beads (`bd`, git-native dependency-graph issue tracker) as the durable\ncross-agent backlog, per research/task-tracking-best-practices.md (empirically\nverified install + scheme). Populated 75 issues from three sources:\n- design docs (roadmap, PLAN.md, ~58 research/*.md, per-crate TODO.md): 40 tasks\n- code TODO/FIXME/deferred markers: 11 tasks\n- roborev backlog: 1 still-present fixable finding + 10 quota-failed re-runs + a\n  batch-verify task for 13 findings already addressed by the ZK soundness work\nPlus 5 epics (MPC, ZK build-out, HDT+Turtle parse perf, roborev, CI), 3 closed\nmilestones (serve-wave-b, M3, lint-gate), and known follow-ups (fmt reformat,\nflaky tokens test, neon-intersect).\n\nDependency edges model the merge queue + prerequisites: M4 spike ← in-circuit sig\ngadget; HDT PFC decode ← direct SPO decoder; rustfmt hard-gate ← one-time\nreformat; Turtle interspersed-directive ← W3C TurtleTests rejection oracle. So\n`bd ready` (66 unblocked) computes the parallelisable work-set offline.\nembeddeddolt/ is gitignored; .beads/issues.jsonl is the committed source-of-record.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T19:07:00Z",
          "tree_id": "6b4fc437b2768a9a8d553c8730f2390523f2ddd5",
          "url": "https://github.com/jeswr/sparq/commit/810f783778e52483cd6294a40a56692df4157f9a"
        },
        "date": 1781377727495,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.54,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3348.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4893.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 811,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13578.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 63431.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 164014.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2761.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42629.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7172.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57914.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 157891.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2421.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37119.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.147,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "d132eb6e8c67ccc28e074f31410803b018ca4621",
          "message": "docs(mpc): replace machine-local absolute paths with repo-relative + external refs [OPUS-4.8]\n\nroborev finding (job 2188-era): the architecture doc's Key reference files listed\n/home/ubuntu/sparq/... and /home/ubuntu/refs/zkp-sparql-workspace/... and\n/tmp/transfer-report.pdf — machine-local paths meaningless in a clone. Relativise\nthe in-repo ref; mark the zkp-sparql-workspace + transfer-report ones as external\n(Jesse's private workspace / blog), not committable paths.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T19:07:52Z",
          "tree_id": "1981e9c36fab31d394f00b6bc09ee71ada2610c8",
          "url": "https://github.com/jeswr/sparq/commit/d132eb6e8c67ccc28e074f31410803b018ca4621"
        },
        "date": 1781377836723,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.536,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3327.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4874.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 825.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13036.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 60483.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 163108.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4819,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42179,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7311.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57850.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 155853.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4113.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38876.1,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.15,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "1dc62b5515df3331c4927db8fe21bbac833d2931",
          "message": "chore(tracking): sync beads issues.jsonl (close sq-s3r doc-fix, sq-moo.4 tokens de-flake) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T19:10:17Z",
          "tree_id": "35997d5c6677c06ed745d4f5af4ef24f3e095a70",
          "url": "https://github.com/jeswr/sparq/commit/1dc62b5515df3331c4927db8fe21bbac833d2931"
        },
        "date": 1781377943438,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.534,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3409.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4928,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 817.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13068.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 58046.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 154704.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3843.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40865.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7136.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 54845.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 151463.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3308.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37129.2,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.145,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "6494493c2c9c7a9e3629d595921ebb786b21c038",
          "message": "merge: HDT direct-decoder benchmark (2.1x/1.44x) + zstd/bzip2 compressed-HDT [OPUS-4.8]\n\nWave A/B of the parsing-opt plan, MEASURED:\n- bench-hdt A/B proves the sparq-side direct SPO decoder (skips upstream\n  TriplesBitmap wavelet+OP-index build) is ~2.1-2.25x faster on triple-walk-\n  dominated archives and ~1.44x on dict-dominated ones (RSS 1.07x lower); the\n  win-source shifts H1/H2 -> H3/H4 across regimes exactly as planned.\n- H5: zstd (.hdt.zst) + bzip2 (.hdt.bz2) added to the magic-byte content-sniffer,\n  streaming (never fully materialized); differential test asserts gzip/zstd/bzip2\n  all decode to the identical triple set as plain .hdt.\nGates: cargo test -p sparq-hdt 13/13 green; clippy -p sparq-hdt -D warnings clean.\nOnly touches sparq-hdt + the standalone bench/parse harness.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T19:12:25Z",
          "tree_id": "54a4b63d68870f82f8207db69be4b755ce84969c",
          "url": "https://github.com/jeswr/sparq/commit/6494493c2c9c7a9e3629d595921ebb786b21c038"
        },
        "date": 1781378144535,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.585,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3198.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4894.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 817.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 16548.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 72915.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 183977.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3561.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 51248.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8961.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 68356.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 168547.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2671.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 46223.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.181,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "114e745eee070b87ee789b2fa59e8c8460afdf81",
          "message": "chore(tracking): sync beads — HDT decoder proven + compressed codecs landed (6494493) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T19:14:34Z",
          "tree_id": "5d87d4665be780c587341c35e1203ca3e2e4d3cb",
          "url": "https://github.com/jeswr/sparq/commit/114e745eee070b87ee789b2fa59e8c8460afdf81"
        },
        "date": 1781378255338,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.534,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3148.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4408.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 762.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12338.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55437.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 151916.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4339.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 39873.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7394.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 53915.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 143371.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2182.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 36629.4,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.142,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "b623db5432ac26ea7977dd00ce4bf315f5ac0b97",
          "message": "fix(bench,docs): ec2-bench safety+parse fixes; un-stale M4 doc dating [OPUS-4.8]\n\nroborev #2264 (job 2264 @ ba9c281), 3 findings in bench/ec2-bench.sh:\n- HIGH: cmd_terminate now REQUIRES tag:purpose=sparq-bench before terminating —\n  last-line defence against fat-fingering prod (i-090531b4ede8f2d3f) or the dev\n  box (the safety contract the harness documents but did not enforce).\n- cmd_wait_result returns FAILURE (not 0) when the box terminates before emitting\n  the END result marker — a terminated-incomplete run is not a success.\n- extract_invoke strips from the closing quote onward, so a TOML inline comment\n  after invoke=\"...\" no longer leaves a trailing quote (`bench x\"`). Latent today.\nroborev #2267 (job 2267 @ 735ed6c): M4 doc 'unbuilt ... through 2026' -> 'as of\nthis survey (2026-06-13)' (a survey snapshot is not a forward claim).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T19:21:50Z",
          "tree_id": "400b7ed7240b8af41400ec0358e00bc4ce760aba",
          "url": "https://github.com/jeswr/sparq/commit/b623db5432ac26ea7977dd00ce4bf315f5ac0b97"
        },
        "date": 1781378627936,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.545,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3347.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4962.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 825.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13256.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59395.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 157397.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4531.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42204.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7085.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55078,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 154839.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3906.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38271.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.147,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "c129ab78ecd639b7ac3ea233db3b72bd4b2f85c2",
          "message": "fix(bench): extract_invoke must strip only the CLOSING quote, not the first [OPUS-4.8]\n\nroborev #2277 (codex, on b623db5): my prior `sub(/\".*$/,\"\")` stripped from the\nFIRST quote in the value, truncating valid TOML basic strings with escaped quotes\n(e.g. invoke = \"python -c \\\"print(1)\\\"\" -> `python -c \\`). Anchor at EOL —\n`sub(/\"[[:space:]]*(#.*)?$/,\"\")` removes only the final closing quote plus any\ntrailing inline comment, preserving escaped \\\" inside the value AND still eating\na ` # comment` tail. Verified against plain / inline-comment / escaped-quote /\ncomment-with-quote inputs.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T19:24:07Z",
          "tree_id": "b8be810c4c5529ff942e948fc6f8e94ffdb8ad45",
          "url": "https://github.com/jeswr/sparq/commit/c129ab78ecd639b7ac3ea233db3b72bd4b2f85c2"
        },
        "date": 1781378759594,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.55,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3366,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4853.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 819.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13062.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 61476.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 160348.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 5363.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42635.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7079.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57680.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 149907.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4696.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39182.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.147,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "b7be8ceb06a2ec7979753895ba7e1f42938ca0e5",
          "message": "chore(tracking): sync beads — retro roborev findings (nonce-gap, intersect-defer) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T19:24:33Z",
          "tree_id": "5c38fdfaf431e1890f326f8a47e4b40ddfa76f9f",
          "url": "https://github.com/jeswr/sparq/commit/b7be8ceb06a2ec7979753895ba7e1f42938ca0e5"
        },
        "date": 1781378862807,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.538,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3141.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4384.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 748.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12323.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55443.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 146517.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3203.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40104.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7381.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55356,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 148998.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2378.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 43851.5,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.144,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "04bbc4ac86fceeb3f7248c86deb0852557e08e70",
          "message": "fix(bench): parse TOML invoke string explicitly (escape-aware), not via regex [OPUS-4.8]\n\nroborev #2278 (codex, on c129ab7): the end-anchored regex still mis-handled an\nescaped quote followed by '#' inside the value (invoke = \"python -c \\\"# hi\\\"\"\ntruncated to 'python -c \\'), because a regex cannot tell an in-string # from a\ncomment. Replace the regex with an explicit walk to the first UNESCAPED closing\nquote, preserving escaped chars verbatim and ignoring any trailing comment.\nVerified: plain / inline-comment / escaped-quote / escaped-quote+hash /\nmulti-command(\\n) all extract correctly. (Also fixed a shell-quoting bug: an\napostrophe in a comment had closed the awk single-quote — bash -n now clean.)\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T19:28:13Z",
          "tree_id": "dfebb264bf63c3a01b5f36f16cc70e9997fc4c92",
          "url": "https://github.com/jeswr/sparq/commit/04bbc4ac86fceeb3f7248c86deb0852557e08e70"
        },
        "date": 1781379006806,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.534,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3137.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4363.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 759.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13087.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56293.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 151870.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4461.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40359,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7474.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55835.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 148511.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2665.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38046.2,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "5e5b6b0c566d2650ea48bc6352e1544dbf635b16",
          "message": "ci(conformance): bump inference ratchet 1654 -> 1967 (W3C rdf-turtle 313 added) [OPUS-4.8]\n\nThe Turtle B4 rejection oracle wires the W3C rdf-turtle suite into the\nsparq-inference-conformance binary. New overall: 1950 pass + 17 documented\ndivergence = 1967, 0 fail (was 1654; +313 rdf-turtle, all pass). Raise the\nratchet floor to 1967 so a regression in ANY suite — including the new Turtle\nrejection/eval cases — fails CI. fetch-inference-suites.sh already clones\nw3c/rdf-tests (which contains rdf/rdf11/rdf-turtle), so the inference job has the\ndata. Never lowered.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T19:33:52Z",
          "tree_id": "45d51ef810bf28fc9578930152f7db2ab105b3f5",
          "url": "https://github.com/jeswr/sparq/commit/5e5b6b0c566d2650ea48bc6352e1544dbf635b16"
        },
        "date": 1781379340051,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.546,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3393.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4853.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 823.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13302.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 61946.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 166170.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2915.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 43225.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7366.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 59719.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 163879.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2533.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40199.6,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.15,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "4ee0bb6129adb706d9174c1279cc59ef5d87e82e",
          "message": "chore(tracking): sync beads — Turtle B4 oracle + T1 landed (5e5b6b0) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T19:34:19Z",
          "tree_id": "00db82b17223d45f1bb1211b6629eaefe56858ab",
          "url": "https://github.com/jeswr/sparq/commit/4ee0bb6129adb706d9174c1279cc59ef5d87e82e"
        },
        "date": 1781379431512,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.42,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 2.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 2588.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 3758.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 640.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 11350.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 50768.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 126230.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3550.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 35407.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 5768.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 48003.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 122298.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 1796.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 32299.5,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.118,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "c991f6da2cb8d250c296a4c44e09378574d9aeef",
          "message": "fix(server): silence pre-existing result_large_err on time-travel resolve_pin [OPUS-4.8]\n\nDiscovered while landing sq-uqh: the #[cfg(feature=\"time-travel\")] variant of\nresolve_pin lacked the #[allow(clippy::result_large_err)] its non-time-travel\ntwin already carries, so clippy --features time-travel -D warnings failed (pre-\nexisting, not from the PodId change). Add the allow so the time-travel feature is\nclippy-clean too.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:15:10Z",
          "tree_id": "3a62cbc56600d550030cb5b383d349cd3612158a",
          "url": "https://github.com/jeswr/sparq/commit/c991f6da2cb8d250c296a4c44e09378574d9aeef"
        },
        "date": 1781381821203,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.545,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3451.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4938.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 823.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12894,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59063,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 161146.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4605.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41946.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7243.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57297.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 150489.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2829.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38152.4,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "3121ebe8ecdc89bb678e288aa8a5cba7a14a65b1",
          "message": "merge: ZK verifier robustness — durable nonce store + panic-free proof parse + nonce policy [OPUS-4.8]\n\nsq-aih (P1): FileSeenNonces — durable, restart-surviving, cross-process single-use\nstore (flock(LOCK_EX) check-and-append + fsync, fail-closed on every error path);\nInMemorySeenNonces relabelled non-durable/test-only. Restart-survival test green.\nsq-dua: malformed proof_hex already routes through CheckError::MalformedProof\n(verified panic-free incl panic=abort release); added 5-class e2e coverage.\nsq-3v2: burn-on-mismatch nonce policy is intentional (no rejection-oracle retry) —\ndocumented + test asserts nonce consumed on binding-mismatch then NonceReplay.\nclippy -D warnings clean; lib 11/11; e2e 69/0 incl real bb prove/verify.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:16:18Z",
          "tree_id": "ec071d3dbd2da03b07b2f12ad470e643d2aad85c",
          "url": "https://github.com/jeswr/sparq/commit/3121ebe8ecdc89bb678e288aa8a5cba7a14a65b1"
        },
        "date": 1781381945796,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.55,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3336.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4886.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 812,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13188,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59034.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 156956,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4167.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42275.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7228.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57533.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 151405.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2609,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37677.7,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.147,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "3e6974a4851886cf0ecb75811ee310343234ffea",
          "message": "merge: MPC masking uses a CSPRNG, not SplitMix64 (sq-1vt, security) [OPUS-4.8]\n\nReplace the deterministic SplitMix64 masking RNG with ChaCha20Rng OS-seeded\n(SecureRng, default); the deterministic SplitMix64 is now InsecureTestRng behind\n#[cfg(any(test, feature=\"insecure-test-rng\"))] — a normal build CANNOT construct a\npredictable masking RNG. Split Clone-able ShamirBackend config from a short-lived\nShamirDealer that owns the live keystream (fixes a clone-reuse keystream-dup bug);\ndealer() mints a fresh OS-seeded CSPRNG per session. Uniform F_{2^61-1} sampling via\nrejection (reject the single value P), no modulo bias. 40 tests pass; wasm-exclusion\nintact. Caveat: fixes masking unpredictability (confidentiality), not malicious\nsecurity / distributed dealer (deferred per M3/M4).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:18:54Z",
          "tree_id": "47e476c7b8480f06184dab5a4917e6e5cb1bb1b8",
          "url": "https://github.com/jeswr/sparq/commit/3e6974a4851886cf0ecb75811ee310343234ffea"
        },
        "date": 1781382135642,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.542,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3136,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4438.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 752.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13067,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 63647.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 152281.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2974.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40619.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7496.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57187.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 148834.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3480.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37939,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "995af754a6214f2e0a1d7b7ff4caeeeb143131ff",
          "message": "fix(rsp): allow large_enum_variant on Materializer (workspace feature-unification) [OPUS-4.8]\n\nThe full-workspace clippy gate (CI-matching) tripped large_enum_variant on\nMaterializer::PersistentDict (the variant holds the live Dict inline — that IS the\npersistent-dict mode), which only surfaces under workspace feature-unification, not\nthe isolated -p sparq-rsp clippy the change was gated against. There is exactly ONE\nMaterializer per ContinuousQuery, so the lint's 'every instance padded to the largest\nvariant' cost does not apply; boxing would add hot-path indirection per tick for no\nbenefit. #[allow] with that rationale.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:23:17Z",
          "tree_id": "91778188a0ffcc02e16786742c74007c9945dcba",
          "url": "https://github.com/jeswr/sparq/commit/995af754a6214f2e0a1d7b7ff4caeeeb143131ff"
        },
        "date": 1781382310682,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.539,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3392.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4860.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 821.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13074.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 58023,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 155955.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3526.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41915.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7082.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 53993.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 144899.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2319.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 36464.1,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.144,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "1baad25cf2570284703264fb1e8012ac69153590",
          "message": "chore(tracking): sync beads — Wave 1 closures (uqh/aih/dua/3v2/d57/1vt/lhg) + follow-ups [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:23:19Z",
          "tree_id": "f32d125097bf226a18a01048cd2e5ac0a981f671",
          "url": "https://github.com/jeswr/sparq/commit/1baad25cf2570284703264fb1e8012ac69153590"
        },
        "date": 1781382421161,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.547,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3156.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4380.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 755,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12918.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 57928.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 160057.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 5662.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41863.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7908.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 60326.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 165329.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4159.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 43594.2,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.15,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "e0efcb37bd4c8188a892a6558afdb28b892bd90a",
          "message": "merge: introspect VoID export + seed summary + CS join hints (sq-cc7) [OPUS-4.8]\n\nto_void(): W3C VoID as N-Triples (void:triples/entities/distinctSubjects/classes/\nproperties + class/property partitions, all exact; distinctObjects deferred — needs\na union pass). schema_summary_for(seeds,budget): the seed-scoped 10k-property-KG\nretrieval path (general summary already existed). join_hints: cross-class (C,p,D)\nedge table mined in the same SPO scan as characteristic sets. clippy clean; 14+1\ntests; cs-planner consumer 137/137 unregressed.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:28:47Z",
          "tree_id": "4d3788a2a462b5ff8a19bbe256b73ddfd96e0c4a",
          "url": "https://github.com/jeswr/sparq/commit/e0efcb37bd4c8188a892a6558afdb28b892bd90a"
        },
        "date": 1781382648678,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.421,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 2.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 2623.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 3803,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 633.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 10391.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 49652.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 131399.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4281.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 36105.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 5981.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 48468.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 126906.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3344.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 34644.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.119,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "273991fe997ea84309898760786b65ec87fd4f26",
          "message": "merge: opt-in positional postings + phrase-query operator (sq-23j) [OPUS-4.8]\n\nsparq-text: add opt-in token positions (Option<Positions>, default None = the cheap\n8B non-positional path UNCHANGED) via build_with_positions/with_positions, and a\nphrase(query) operator returning docs where the analyzed tokens appear at consecutive\npositions in order (order-significant, same UAX#29 analyzer as indexing). Parallel\nshard-merge unions position maps (disjoint doc-id ranges). 11 phrase tests; both\n-p sparq-text and full-workspace clippy green.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:37:10Z",
          "tree_id": "971844e36cedc9b118a7f91621fcac8bc0ee1563",
          "url": "https://github.com/jeswr/sparq/commit/273991fe997ea84309898760786b65ec87fd4f26"
        },
        "date": 1781383177360,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.55,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3368.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4863.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 820.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13163,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 60590,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 166243.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 5758.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 43477,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7491.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 61217.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 156484.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2954.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40019.6,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.152,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "87cc49bf891f365ff8d17909d82b6e9436424b0f",
          "message": "chore(tracking): sync beads — cc7/23j/1rr closed + follow-ups [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:39:02Z",
          "tree_id": "b32e784c8daa664d40d03cca2a6b3120c62f6f7c",
          "url": "https://github.com/jeswr/sparq/commit/87cc49bf891f365ff8d17909d82b6e9436424b0f"
        },
        "date": 1781383289261,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.539,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3369.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4877,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 818.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13001.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59204.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 157107.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4061.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41828.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7282,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 56512.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 154759.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3657.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37750.3,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.147,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "9caccc3e234721b3dd1d64b83f901c64bcab0e1f",
          "message": "merge: ZK test/bench infra — differential fuzzer + forge-and-verify + gate-count + cost curve [OPUS-4.8]\n\nsq-61g: differential prove->verify->cleartext fuzzer (seedable, prints seed; cheap\nwitness-path default + #[ignore] full-bb-prove). FOUND a real completeness gap:\nbuild_filter_int derives an unprovable member for 3/5-19-digit operands (only {1,2,4}\nbuckets compile; circuit requires exact digit-count=D) — NOT a soundness hole. Tracked.\nsq-ajl: forge-and-verify negative suite — one forge per binding through verify_manifest;\nall 10 structural + 3 byte-binding (toolchain-gated) gates REJECT.\nsq-c5f: gate-count regression gate vs checked-in baseline (measured 3% tolerance).\nsq-pn2: full-family (k,n,r,d) cost-curve bench (standalone workspace). clippy -D\nwarnings clean; full test suite green.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:40:09Z",
          "tree_id": "0a497f01e85309e06c5159fb7f29f64c1d39cdc9",
          "url": "https://github.com/jeswr/sparq/commit/9caccc3e234721b3dd1d64b83f901c64bcab0e1f"
        },
        "date": 1781383403916,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.533,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3135.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4372.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 759.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13022.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55983.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 151910.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2459.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40011.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7414.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55991.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 146937.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2721.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37727.5,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1518547,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "8d569a47224bbd7b2203b13e8a79745674239579",
          "message": "merge: wasm SELECT cursor + CONSTRUCT/DESCRIBE quad export (sq-0f7+sq-hlq) [OPUS-4.8]\n\nqueryCursor(sparql,batchSize)->SolutionCursor: batch-streamed SELECT solutions, each\nbatch a self-contained SPARQL-JSON doc (engine seam to_sparql_json_rows). queryQuads\n(CONSTRUCT|DESCRIBE)->N-Triples + queryQuadsChunks (engine seam construct_or_describe).\nclippy -D warnings + wasm32 build + 11 tests green. Caveat: batch-level JS-side bound\n(engine still materialises fully — a lazy iterator is the follow-up).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n# Conflicts:\n#\tcrates/sparq-core/src/lib.rs\n#\tcrates/sparq-engine/src/exec.rs",
          "timestamp": "2026-06-13T20:48:11Z",
          "tree_id": "16b4a3533411bae20b4f40f8ccfe0b22c5016a68",
          "url": "https://github.com/jeswr/sparq/commit/8d569a47224bbd7b2203b13e8a79745674239579"
        },
        "date": 1781383878420,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.54,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3359.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4912.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 812.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13072.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 60163.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 161476.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 5418.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42647,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7289.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57552.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 154586.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3886.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38709.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.148,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1564231,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "f6e41c2f62cdeaa94a71782f36415a20a49ac079",
          "message": "chore(tracking): sync beads — ZK test-bench + wasm closed; filter_int gap + follow-ups [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:49:47Z",
          "tree_id": "051fd30ce64ae565f65086e1b28ad03694b8a8ed",
          "url": "https://github.com/jeswr/sparq/commit/f6e41c2f62cdeaa94a71782f36415a20a49ac079"
        },
        "date": 1781383987464,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.534,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3140.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4375.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 749.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12396.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56825.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 151707.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4924.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40959.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7733.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57497.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 148717.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2690.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39780.7,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1564231,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "5fd96d358f3eb59ee1e5858ff673bf8ad60cbd9a",
          "message": "merge: persistent on-disk Vamana/DiskANN vector index (sq-7zc) [OPUS-4.8]\n\nsparq-vectors: DiskAnnIndex — self-built Vamana graph (RobustPrune, 2 alpha-passes,\nmedoid entry, greedy beam) with a versioned .spqg on-disk format (co-located vector+\nadjacency per record for one-page-per-hop locality). build() writes once; open() is\nmmap + header validation, NO rebuild. recall@10 0.966 vs brute force; reopen returns\nbyte-identical neighbours; HNSW id-set parity. clippy (incl full workspace) + tests\ngreen. PQ-compressed RAM cache honestly deferred to sibling sq-nq5.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:51:53Z",
          "tree_id": "26ccde5610a8489d840ab344f88920184f33ac32",
          "url": "https://github.com/jeswr/sparq/commit/5fd96d358f3eb59ee1e5858ff673bf8ad60cbd9a"
        },
        "date": 1781384214145,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.533,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3149.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4386.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 750.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12286.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55546.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 150393.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2631.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40127.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7440.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 54728.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 146975.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3053.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37458.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.141,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1564231,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "9f4673c3f09e31a7492c93eaf371af77caee801f",
          "message": "chore(tracking): sync beads — 7zc/gn3 closed, MPC joins deferred to design-review track [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:55:52Z",
          "tree_id": "cf05ae240ba61965e4baed3c23919564a9963ab8",
          "url": "https://github.com/jeswr/sparq/commit/9f4673c3f09e31a7492c93eaf371af77caee801f"
        },
        "date": 1781384332158,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.529,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3389.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4874.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 811.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13156.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59535.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 164033.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 5570.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42051,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 6926,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 59444.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 152634.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4382.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39713.1,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.149,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1564231,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "1f30dd7b0f15f09b98bace64fd69914882dce825",
          "message": "merge: engine GRAPH-scoped zero-len paths (verified) + EXISTS early-exit (sq-wij+sq-rd2) [OPUS-4.8]\n\nsq-wij: GRAPH-scoped zero-length property paths (:p* / :p?) verified correct\n(scoped to the named sub-graph, default node-set not leaked) + locked with a\nvalue-level 3-scope test. No bug.\nsq-rd2: native EXISTS early-exit — uncorrelated EXISTS now stops at the first\nsolution (Slice LIMIT 1) using spargebra's on_in_scope_variable to stay sound;\ncorrelated EXISTS unchanged; ASK already early-exited. Over-evaluation tripwire\ntests. clippy + workspace clippy clean; W3C ratchet 1229 unchanged.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T20:57:07Z",
          "tree_id": "879b3f7cdf66758659cee43c7f80534e371d753b",
          "url": "https://github.com/jeswr/sparq/commit/1f30dd7b0f15f09b98bace64fd69914882dce825"
        },
        "date": 1781384651375,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.545,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3340.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4846.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 817.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13058.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59147.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 158868,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2766.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41706.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7237.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 56560.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 147543.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2579.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37229.3,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.145,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1567668,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "b8f74e9e5f3ad01e5fa54f4049641ee75269608b",
          "message": "merge: noir_XPath string tests + bounded regex + Pedersen hash (sq-y73) [OPUS-4.8]\n\nzk/xpath (Noir-only): 15 string-fn tests (found+fixed a real OOB in translate — Noir\n& doesn't short-circuit), regex.nr (literal/anchored/prefix/char-class bounded subset;\nfull fn:matches scoped out — backrefs/\\p{}/unbounded quantifiers not circuit-feasible),\nhash.nr (domain-separated Pedersen content hash + hex formatter; SHA/MD5 scoped out —\nbeta.21 stdlib lacks them). nargo test 102/102 + 254/254. sq-p9t (float-API migration)\nwas already landed (b4aaa18 etc.) — verified nargo check clean, no new commit.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:04:23Z",
          "tree_id": "a4d1ff45f7718cc2ffd5b5755e5c3580bd271f2b",
          "url": "https://github.com/jeswr/sparq/commit/b8f74e9e5f3ad01e5fa54f4049641ee75269608b"
        },
        "date": 1781384768672,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.556,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3350.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4851.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 828.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12915.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59267.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 161391,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3077.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41701.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7106,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 56642.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 154972.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 5133.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38623.1,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.148,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1567668,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "f1fe955f527b6dec1d2032783475ae0c715c19af",
          "message": "chore(tracking): sync beads — wij/rd2/p9t/y73 closed + follow-ups [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:05:07Z",
          "tree_id": "9db89d6b298787d693a02b5016679b71c90be03d",
          "url": "https://github.com/jeswr/sparq/commit/f1fe955f527b6dec1d2032783475ae0c715c19af"
        },
        "date": 1781384874768,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.538,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3145.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4422.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 758.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12726.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55462.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 155929.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4596.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41020.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7554,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 56962.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 147400,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4040.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38807.3,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.14,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1567668,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "20b72f636202339f5aa85d0c0ddb4e5e0ab4f6cb",
          "message": "merge: public immutable Graph::snapshot + GraphSnapshot (sq-5lf) [OPUS-4.8]\n\nsparq-core: add Graph::snapshot()->GraphSnapshot, a cheap O(overlay) logically-\nindependent immutable point-in-time view layered on the existing fork() COW mechanism\n(Arc-shares the 6 perms + frozen dict base; mutations overlay per-generation, never leak\nacross snapshots). GraphSnapshot is Send+Sync, Deref<Graph> (all read methods + usable as\n&Graph), no DerefMut; into_graph() yields a mutable copy (sparq-py Graph.copy()),\nas_graph() borrows. base_strong_count() proves structural sharing. Cheapness test: +1\nrefcount/snapshot, no index dup. Full workspace 667/0; clippy default/compact/mmap clean.\nUnblocks sparq-rsp/sparq-py/js downstream TODOs (consumer-side work remains).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:08:27Z",
          "tree_id": "8eb244b890ee59de852f1f3cc422252c889c51d6",
          "url": "https://github.com/jeswr/sparq/commit/20b72f636202339f5aa85d0c0ddb4e5e0ab4f6cb"
        },
        "date": 1781385082810,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.552,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3369.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4924.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 811.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13224.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 58949.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 157652.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2713,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41761.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7203.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58157.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 158232.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3366,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37899,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1567668,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "0a5c6c22e45ae4c152f0a1cbc937339719f6bd87",
          "message": "merge: ZK hidden-index revocation proof (sq-3e5; partial sq-h2v) [OPUS-4.8]\n\ndepth-D Poseidon2 Merkle hidden-index revocation: PUBLIC (challenge, root), PRIVATE\n(index, bit, siblings[D]); proves bit==0 (active) + index<2^D without disclosing index/\nbit/path. Verifier bind_hidden_revocation recomputes the authoritative root from the RP's\nOWN StatusListSnapshot (audit-#12 trust anchor) + requires byte-equality; clear-index path\nunchanged (no soundness regression). revoked-unprovable / index-private / forged-root-rejected\ntests pass (real prove/verify); existing forges+e2e green; nargo 30/30; clippy+workspace\nclippy clean. Representative depth-10 (1024 idx); production sparse list deferred.\nRESIDUAL (sq-h2v not fully closed): issuer-attestation status_ref_digest still embeds the\nclear index -> needs a sparq_zk::sig commit-to-index change (follow-up).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:14:34Z",
          "tree_id": "3c617471f22f5b5db9229ef4f6503698a92ad29b",
          "url": "https://github.com/jeswr/sparq/commit/0a5c6c22e45ae4c152f0a1cbc937339719f6bd87"
        },
        "date": 1781385472722,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.547,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3376.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4872.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 814.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13138.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 61316.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 165956.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4999.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 43266.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7228,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 63781.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 158770,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2280.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39117.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.149,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1567668,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "0fbc939fad9c6a02712e4e84f30a2e362ae83ac9",
          "message": "merge: vectors scalar + product quantization (sq-nq5) [OPUS-4.8]\n\nsparq-vectors/quant.rs: ScalarQuantizer (per-dim f32->u8, 4x) + ProductQuantizer (M\nsubspaces, k-means++ codebooks, M-byte codes, ADC via DistanceTable) + EncodedStore\nRAM cache, all over the crate's L2-normalized cosine convention. HONEST recall/compression:\nPQ-alone ~0.60 @8x (coarse filter), PQ-filter + full-precision re-rank ~0.98 @8x. SQ\nworst-component error within half-step bound. Standalone (NOT baked into .spqg — deliberate,\nno driving workload yet; the DiskANN rank-on-codes+re-rank wiring is the documented follow-up).\nclippy + workspace clippy + 73 tests green.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:16:21Z",
          "tree_id": "26f38f9c110b054e9b92330f48664247ed97db5a",
          "url": "https://github.com/jeswr/sparq/commit/0fbc939fad9c6a02712e4e84f30a2e362ae83ac9"
        },
        "date": 1781385652460,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.529,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3140.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4370.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 760.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12428.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 54820.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 145400.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4488.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 39876.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7360.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 53822.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 146612.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2248.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 36275.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.142,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1567668,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "b5a0fa53c1db0fdaed5c99bf739b11abdac079f8",
          "message": "chore(tracking): sync beads — 5lf/3e5/h2v/nq5/9u1 closed + ZK/wasm/text follow-ups [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:20:00Z",
          "tree_id": "4451e7ff2fcd1649bf128a07e43865cbc44b30f7",
          "url": "https://github.com/jeswr/sparq/commit/b5a0fa53c1db0fdaed5c99bf739b11abdac079f8"
        },
        "date": 1781385762458,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.529,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3143.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4396,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 761.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12842.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 57613.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 151049.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2671.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41199.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7655.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 56943.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 146309,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3280.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38477.6,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.144,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1567668,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "c0246a4ad9741b3320288fde8dd8c8a85eefaa8f",
          "message": "merge: text:phrase magic predicate over the positional index (sq-z2u) [OPUS-4.8]\n\nsparq-text: text:phrase SPARQL magic predicate (vocab::PHRASE) wiring phrase() via the\nsame VALUES-rewrite as text:matches (ReqKind::Phrase in rewrite.rs); resolves matched ids\nto literals, inlines as VALUES joined onto the BGP; same UAX#29+casefold analyzer. Returns\na clear query error (no panic) if the index lacks positions; text:score on a phrase subject\nrejected. Engine untouched (all text: dispatch is in sparq-text). clippy+workspace+40 tests green.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n# Conflicts:\n#\t.beads/issues.jsonl",
          "timestamp": "2026-06-13T21:22:44Z",
          "tree_id": "569383b3ebd7aea98859cca04f6e4e5eb296755d",
          "url": "https://github.com/jeswr/sparq/commit/c0246a4ad9741b3320288fde8dd8c8a85eefaa8f"
        },
        "date": 1781385874918,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.579,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3206.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4955,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 818.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 15679.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 69266.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 185272.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2602.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 50730.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9443.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 64618.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 163555.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2585.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 45347.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.186,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1567668,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "e125a166090cb8746871021bd2391de4725e71c7",
          "message": "merge: lock in RDF-star T6 variable-in-quoted-triple binding (sq-kbs) [OPUS-4.8]\n\nsparq-engine: the T6 decomposition (parser desugars <<..>> to rdf:reifies BGP triple ->\nextract_quoted_constraints -> quoted_relation/unify_quoted scans stored triple terms,\nground filters/var binds/repeated-var consistency, joins via ordinary machinery; recursive\nnesting) was already implemented (F14, f32cb9f). Add 4 regression tests (var in s/p/o,\nground+var mix, <<?s ?p ?o>> enumerates all, nested) + reframe the stale 'T6 unsupported'\ncomment as an unreachable defensive backstop. eval-triple-terms 41/41, ratchet 1229.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:24:09Z",
          "tree_id": "21259ea5fe52503eb13c326fcc4e0fc0dd868f52",
          "url": "https://github.com/jeswr/sparq/commit/e125a166090cb8746871021bd2391de4725e71c7"
        },
        "date": 1781386222181,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.532,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3016.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4450.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 821.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12575.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56330.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 147449.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2515.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40798.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7623.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55466.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 143762,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2826.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39215.6,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1567591,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "7ab8bbaedefedb7b76cff8907cecbd35ceb41b74",
          "message": "chore(tracking): sync beads — z2u/ouq/kbs/16a closed [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:29:21Z",
          "tree_id": "0c4fa026fb4f424b29b60d4e91eae82f54414a4b",
          "url": "https://github.com/jeswr/sparq/commit/7ab8bbaedefedb7b76cff8907cecbd35ceb41b74"
        },
        "date": 1781386339279,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.538,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3014.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4456.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 810.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13685.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56073.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 147456.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2559.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40763.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7639.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57611.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 145966.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2720.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37555.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1571665,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "932c1a6975df8c5ece92fb0aab59afaebcd90bc1",
          "message": "merge: SHACL §6 SPARQL-based constraint components (sq-sm2) [OPUS-4.8]\n\nsparq-shacl: full §6 machinery — discover_components (incl rdfs:subClassOf closure),\nsh:parameter (optional, sh:name), activation keyed on param predicates, ASK-per-value +\nSELECT-per-focus validators (node/property/generic), multi-param VALUES pre-binding,\nsh:message {$param} substitution. Found+fixed a real bug: pre-binding VALUES joined ABOVE\na FILTER left vars unbound -> push_values_down joins below Filter/Extend/etc. Core 98/98,\nsparql 5/5, §5.2 12/12 (no regression), new component tests 7/7; workspace clippy clean.\nScoped out (documented): dash-imports W3C sparql/component suite, propertyValidator $PATH.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:37:38Z",
          "tree_id": "3d14d836f9059c0c0a5456bc3c8623221582b496",
          "url": "https://github.com/jeswr/sparq/commit/932c1a6975df8c5ece92fb0aab59afaebcd90bc1"
        },
        "date": 1781386800501,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.538,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3259.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4779.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 811.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12846.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 61127.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 161072.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 5061.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41634,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 6982.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55255.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 148938.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3620.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 36967.5,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1571665,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "0a1dc944116c22f5345059d242ef4c6c632cb862",
          "message": "merge: unique per-graph salts + per-named-graph commitment ingest (sq-610+sq-cn8) [OPUS-4.8]\n\ncrates/sparq-zk/ingest.rs (prover side, verifier untouched): SaltMint draws 32 OS-CSPRNG\nbytes -> 248-bit Fr salt, enforces global uniqueness structurally (redraw on collision,\nfail-closed SaltExhausted; reject caller dup via SaltCollision; from_registry seeds cross-\nsession). IngestedDataset commits each named graph SEPARATELY under its own minted salt\n(Stage 2), wired into ZkDataset::from_store so traced execution resolves into the per-graph\ncommitments. Tests: 1000 distinct mints, collision rejected, per-graph distinct, trace seam.\nDeferred (documented): in-circuit salt binding (audit #9b), durable cross-process uniqueness.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:49:06Z",
          "tree_id": "10e75a122f806ae2819de4882f68388141a7cec4",
          "url": "https://github.com/jeswr/sparq/commit/0a1dc944116c22f5345059d242ef4c6c632cb862"
        },
        "date": 1781387770405,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.536,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3012.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4437.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 811.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12662.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56373.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 151379.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3592.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41301.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7677.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55575.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 149349.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2952.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39331.7,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.144,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1571665,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "0214621d4875093a96f81adc2d6d3da8ce3ff75c",
          "message": "merge: SPARQL 1.1 SERVICE federated query (sq-tt0) [OPUS-4.8]\n\nsparq-engine: SERVICE behind a non-default 'service' feature (ureq blocking client, +\nserde_json, both gated off wasm via cfg(not wasm32) target deps — verified absent from the\nwasm graph). service.rs SPARQL-Results-JSON parser (uri/bnode/literal/lang/datatype/triple,\nabsent var->UNDEF); eval_service renders inner algebra back via spargebra Display, fetches,\ninterns like the VALUES path, joins via join_bindings (Service is non-conjunctive). SILENT ->\njoin identity on any failure; non-SILENT propagates. Transport trait + mock seam + real-\nloopback-endpoint tests (6). clippy default/service/workspace + wasm32 + ratchet 1229 green.\nScoped out: SRX (SRJ-only), bindings pushdown, variable endpoints (documented).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:54:59Z",
          "tree_id": "bc2a3fb0d590b036e4b1302c1689531469392de1",
          "url": "https://github.com/jeswr/sparq/commit/0214621d4875093a96f81adc2d6d3da8ce3ff75c"
        },
        "date": 1781388078472,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.548,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3264.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4803.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 815.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13103.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 62787.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 170741.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2698.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42602.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7199.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58546.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 151595.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2315.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38504.2,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1571665,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "fbe082cc10a6d33880208fc9b5330877e3e75fb0",
          "message": "chore(tracking): sync beads — z9l/610/cn8/tt0 closed + follow-ups; ZK-features+mg9 in progress [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T21:59:34Z",
          "tree_id": "9dbad9eeb83e344468baff2ff762b88bd0e07447",
          "url": "https://github.com/jeswr/sparq/commit/fbe082cc10a6d33880208fc9b5330877e3e75fb0"
        },
        "date": 1781388195739,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.529,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3016.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4461.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 812,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12413.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55901.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 151727.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2800.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40166.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7420.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55094,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 144224,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2147.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37158.2,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.142,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1571665,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "66426dd208d1be8980c769f83730b48fc62a71ce",
          "message": "docs(skills): usage SKILL.md tree + AGENTS.md + Claude Code plugin (agentic discoverability) [OPUS-4.8]\n\nAdds a root skills/ tree of Agent-Skills SKILL.md docs teaching agents how to USE sparq\n(distinct from .claude/skills/ which are dev-agent skills): a router skills/SKILL.md + 15\nper-surface skills (sparql-query, data-formats, cli, http-server, python, javascript-wasm,\ninference, shacl-validation, full-text-search, vector-search, geosparql, streaming-rsp,\nzk-query-proofs, genai-retrieval, mpc). Each uses the agentskills.io frontmatter (name==dir,\ntrigger-laden description). Plus a root AGENTS.md (the cross-agent entry point) carrying: the\npublic-API->SKILL.md maintenance rule, the beads task-tracking + agents-create-beads protocol,\nand the no-hard-coded-perf-numbers rule. Plus .claude-plugin/{plugin,marketplace}.json so the\nskills install as a Claude Code plugin (/plugin marketplace add jeswr/sparq).\n\nAuthored by the author-usage-skills workflow (16 agents surveying each public surface vs the\nlive API). Perf figures in skills will be normalized by the docs-hygiene pass (sq-5vm).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T22:05:30Z",
          "tree_id": "66e7e890df08160fdbae5d787e25c8a1056168e4",
          "url": "https://github.com/jeswr/sparq/commit/66426dd208d1be8980c769f83730b48fc62a71ce"
        },
        "date": 1781388440007,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.539,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3248.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4815.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 828.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13242.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 60237.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 160982.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 5954.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42335.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7048,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57187.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 154541.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2957.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38481.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.144,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1571665,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "b2b7ae79c5f1eec3e6e5bf9cf3eff68983d8e273",
          "message": "chore(tracking): bead 54 markdown TODOs (from docs-hygiene audit) + recent follow-ups [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T22:15:09Z",
          "tree_id": "b4edcaadb24a86408bf5ff01429743865aad6384",
          "url": "https://github.com/jeswr/sparq/commit/b2b7ae79c5f1eec3e6e5bf9cf3eff68983d8e273"
        },
        "date": 1781389002446,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.423,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 2.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 2520,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 3732.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 3.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 639.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 10375.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 48601.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 128314.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 5035.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 34737.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 5851.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 48174.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 123456.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3116.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 32160.6,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.116,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1571665,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "7585bc394c072c11883a99d00d321b473dde8137",
          "message": "merge: extend per-commit perf CI — PRs + Pages dashboard + 2 deterministic metrics [OPUS-4.8]\n\nExtends the existing bench.yml/ci-bench.sh/benchmark-data infra (sq-5vm part 3):\n- bench.yml runs on pull_request too; auto-push gated to main (PRs compute+comment via\n  comment-always/summary-always, never write the published series); history bootstrap +\n  pages skipped on fork PRs; alert-threshold 200->150%.\n- Pages: gh-pages-branch=benchmark-data, dir dev/bench -> dashboard at\n  jeswr.github.io/sparq/dev/bench once the owner enables Pages (toggle documented; bead sq-9aj).\n- ci-bench.sh: +store_bytes_per_triple_small (2nd scale) + parse_ns_per_byte (fixed corpus,\n  smaller-is-better) — deterministic, runner-noise-immune regression gates. 25 metrics, exit 0.\nA true hard-gate on the deterministic metrics is tracked as sq-i1d (the action's fail-on-alert\nis global). Agent created sq-i1d + sq-9aj via the beads protocol.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T22:25:05Z",
          "tree_id": "dfdc46eb94b579cdc5134713f32042dbf92dd651",
          "url": "https://github.com/jeswr/sparq/commit/7585bc394c072c11883a99d00d321b473dde8137"
        },
        "date": 1781389617221,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.537,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9721,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3263.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4781.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 858.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13007.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59022.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 158395.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4240.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41312.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7031,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 56980,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 148753.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2347.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37288,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.144,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1571665,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "759ec84eda17bc5677a620e039d728395d7ccd9c",
          "message": "merge: geof: spatial FILTER pushdown into GeoIndex (sq-mg9) [OPUS-4.8]\n\nEngine stays geometry-free: a thread-local SpatialProvider seam (installed like\nFunctionRegistry) that sparq-geo's GeoIndexProvider implements over its R-tree. The\nconjunctive-BGP residual-filter loop recognises geof:distance(?g,$pt)<R (metric units),\nsfWithin/sfIntersects(?g,$box) -> within_distance_literals / bbox_candidate_literals\ncandidate scan, pre-restricts the binding rows, then the original geof: FILTER still runs\n(pure candidate filter, never a replacement). is_indexed contract keeps any binding the\nindex has no opinion on (no silent drops). 11 with-vs-without identical-result tests\n(+fewer exact checks). Found+fixed 2 critical false-negatives (exterior R<distance mis-\nrecognised; retain dropped unindexed bindings). clippy+workspace+ratchet 1229 green.\nPost-hoc (no speedup) for >/>=, degree units, other sf-relations, OPTIONAL/UNION nesting.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T22:34:03Z",
          "tree_id": "e99fe638ad6b1023be811d2b8e3da3936a930106",
          "url": "https://github.com/jeswr/sparq/commit/759ec84eda17bc5677a620e039d728395d7ccd9c"
        },
        "date": 1781390411246,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.542,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9336,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3269.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4860.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 824.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13221.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 58523.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 160456.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4679.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40818.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8034.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57066.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 157261.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2389.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37674.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.14,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579288,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "82cbe6cb56f2007527f191ec6eaaaff800c9b3ff",
          "message": "merge: ZK-compose features — filter_int_d3 + HolderPoP + FilterF64-composable + entailmentRegime [OPUS-4.8]\n\nsq-wto: derive_filter_int_id requires EXACT compiled-D match (out-of-family -> clean error,\nnever wrong-D); added filter_int_d3 so the family is contiguous 1..=4 (fixes the fuzzer's\nsilently-unprovable 3/5-19-digit operands).\nsq-cwq: real challenge-bound Schnorr HolderPoP (holder_pop_message/sign_holder_pop) + external\nHolderRegistry trust anchor; bind_holder_pop FAIL-CLOSED (was a silent-accept placeholder).\nIssuer->holder credential binding deferred (documented).\nsq-q7e+sq-tat: CircuitId::FilterF64 manifest-composable for the integer-valued xsd:double\nfragment — binds operand to the committed literal + derives IEEE bits from the bound value\n(NO prover-free a_bits; the soundness fix). 4 members filter_f64_d{1..4}. Fractional/scientific\n+ query-text float-FILTER mapping deferred.\nsq-314: derivation module (RDFS rdfs9/rdfs7 subset) + EntailmentPolicy + bind_entailment\nfail-closed (entailment_regime was unchecked free metadata). In-circuit closure proof deferred.\nAll real nargo+bb; clippy+workspace green; gate_count_snapshot updated (all 17416).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T22:38:41Z",
          "tree_id": "ca90969fa62cd5a09f716e65f97678415161fb46",
          "url": "https://github.com/jeswr/sparq/commit/82cbe6cb56f2007527f191ec6eaaaff800c9b3ff"
        },
        "date": 1781390525547,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.548,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9721,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3270.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4865.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 819.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13113,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 62952,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 162515.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4806.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 43784.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8347.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 62091.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 166975.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3241.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 41212.4,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.14,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579288,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "3d1af71173b2567852f0fe85e5aa6cc61f8d62f8",
          "message": "merge: docs hygiene — markdown TODOs->bead pointers + perf numbers->references (sq-5vm) [OPUS-4.8]\n\n10 TODO.md -> thin 'bd ready -l area:<crate>' pointers (design rationale preserved under\nNotes); beaded TODO markers stripped from research/docs. Perf: README ~25 hard-coded figures\n-> a Performance section linking the dashboard (jeswr.github.io/sparq/dev/bench) + benchmarks.toml\n+ CATALOG; single-sourced the rung5 1B + QLever-comparison dups; per-crate README perf tables +\nresearch/BENCHMARKS.md + CHANGELOG + bench/*/README -> reference form (ZK READMEs cite their\ncommitted JSONs); skills cite the measured source once + drop drift figures. Kept external/cited\nthird-party numbers + conformance ratchets. Long-tail + harness-JSON-emission gap beaded\n(sq-my8, sq-d7d). Enforces AGENTS.md's existing no-markdown-TODOs + no-hard-coded-perf rules.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T22:40:24Z",
          "tree_id": "9415be91384b01fba8ff981638b3bce24635181d",
          "url": "https://github.com/jeswr/sparq/commit/3d1af71173b2567852f0fe85e5aa6cc61f8d62f8"
        },
        "date": 1781390635095,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.53,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.8179,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3018.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4395.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 759.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12305.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55447.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 144468.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4582.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40265.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8805,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58436.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 155280.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3777.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40414.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.134,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579288,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "ed122f00636073ea175a0efe5566b0ce72c4bae9",
          "message": "merge: discoverability reach steps — README AI-agents section + package metadata + llms.txt (sq-dg7) [OPUS-4.8]\n\nREADME 'Using sparq with AI agents' section linking AGENTS.md + skills/SKILL.md + the Claude\nCode plugin. npm: repository/homepage/bugs + AGENTS.md/skills in files[] via a prepack copy\n(single-sourced at root, gitignored in js/); verified the tarball includes the router + 15\nsurface SKILL.md. PyPI [project.urls] Homepage/Docs/Issues. crates.io repository/homepage\nalready inherited (keywords at the 5-cap — agent-skills deliberately not added, net discovery\nloss). Root llms.txt orientation index. FIXED broken AGENTS.md links (rust-api/js ->\nsparql-query/javascript-wasm) + added data-formats — the foundation's own links were wrong.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T22:51:03Z",
          "tree_id": "85d888540083693c6d86813c28b8162f837aaad7",
          "url": "https://github.com/jeswr/sparq/commit/ed122f00636073ea175a0efe5566b0ce72c4bae9"
        },
        "date": 1781391176139,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.532,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.895,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3023.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4314.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 753.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12638.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59377.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 157039.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2476.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41242.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9101.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 72411.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 163199.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4313.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40895,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.134,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579288,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "ad2e99fc4e720b27f2d26f4e9bb45c27ba801127",
          "message": "merge: HARD regression gate on deterministic perf metrics (sq-i1d) [OPUS-4.8]\n\nscripts/perf-gate.py: reads this run's deterministic metrics (store/dict bytes-per-triple,\nstore_bytes_per_triple_small, wasm_bundle_bytes, parse_ns_per_byte) + the previous committed\nvalues from benchmark-data/dev/bench/data.js; FAILS CI on a per-metric-threshold regression\n(+2% byte metrics, +10% parse), improvements never fail. Runs before the store step on push\n+ PR. PERF_GATE_ALLOW var = documented deliberate-bump path (new value auto-baselines). The\nglobal github-action-benchmark fail-on-alert stays off (can't gate only deterministic metrics).\n8 self-tests + integration vs real data.js pass. Parse-threshold recalibration tracked (sq-od2).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T22:51:25Z",
          "tree_id": "198becfff0f6ab9b563235282233e24fff2ec918",
          "url": "https://github.com/jeswr/sparq/commit/ad2e99fc4e720b27f2d26f4e9bb45c27ba801127"
        },
        "date": 1781391295016,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.54,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9721,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3266.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4827.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 816.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12879.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 60087,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 163689.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3949.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42003.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 10757.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 60610.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 156677.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2955.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38891.2,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.14,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579288,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "244d422def6be606744ae7a1e5b8e1851873899a",
          "message": "chore(beads): close sq-xxg + sq-ayv (ZK hidden-only attestation + committed-index revocation landed) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T23:24:23Z",
          "tree_id": "431fc642da7ec4a6d75270cc7d558929350c9ea0",
          "url": "https://github.com/jeswr/sparq/commit/244d422def6be606744ae7a1e5b8e1851873899a"
        },
        "date": 1781393174739,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.541,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9721,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3462.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 6735.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 1367,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13339,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 61618.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 164248.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4053,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 43604.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8519.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 62750.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 163939.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3699.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 41731.2,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.141,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579288,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "b1ea87c727e7dbe3547eadecef365fed0d4ac553",
          "message": "docs: repository hygiene cleanup — delete TODO.md + handover docs, codify where things live [OPUS-4.8]\n\nPer Jesse's request: turn TODOs into beads, move durable knowledge into\nAGENTS.md/skills, delete deprecated scratch docs, and add a going-forward\npolicy so no future cleanup runs are needed.\n\nDeleted (content fully migrated):\n- root TODO.md — its one substantial item (the cheap-snapshot API design) is\n  now bead sq-3p1 (full design in the bead's --design field; blocks the server\n  double-buffer / Python Graph.copy() / RSP overlay-snapshot consumers).\n- 9 per-crate TODO.md (geo/hdt/introspect/py/rsp/shacl/sim/vectors + js) — were\n  already bead-pointer stubs; deferred items + rationale live in beads (the\n  'from-md-todo' label), DONE-records live in git/CHANGELOG.\n- HANDOVER-CURRENT.md + HANDOVER-2026-06-12.md — session/orchestration scratch;\n  the work they tracked is done/beaded, durable rules live in Claude memory, and\n  the reusable bit (the gate/merge ritual) is now in AGENTS.md.\n\nKnowledge relocated / policy added:\n- AGENTS.md: new 'Repository hygiene — where things live' section (tasks→beads;\n  knowledge→AGENTS.md/CLAUDE.md/skills/README/research; no narrative scratch\n  docs; no hard-coded perf numbers). Enriched the build/lint/merge ritual.\n  Deduped the doubled SKILL.md maintenance rule. Genericised the machine-local\n  bd path.\n- CLAUDE.md: new thin pointer to AGENTS.md (Claude Code auto-reads it).\n\nDangling-reference sweep: repointed every 'see TODO.md' in crate READMEs, source\ncomments, Cargo.toml, and per-surface SKILL.md files at beads (bd list -l area:*).\n\nRemaining TODO markers beaded + stripped: the TODO(dict-spill) markers in\nbench/wikidata-8b/{RUNBOOK,STATUS}.md → bead sq-1q3 (post-merge runbook\nreconciliation).\n\nAlso: .gitignore now ignores .claude/worktrees/ (agent scratch).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-13T23:42:09Z",
          "tree_id": "980a575406256e02b5893e4b1c2440f59df9d46f",
          "url": "https://github.com/jeswr/sparq/commit/b1ea87c727e7dbe3547eadecef365fed0d4ac553"
        },
        "date": 1781394240367,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.533,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.8565,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3016.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4339.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 749.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12388.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55349.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 148728.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3026.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 39939.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9805.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 56502.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 154012.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3708.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40928.1,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.137,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579288,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "591a931549ad3896b655d82c86f62fc98f474df0",
          "message": "research+beads: coverage & benchmark expansion plan (sq-5o5/sq-bif) [OPUS-4.8]\n\nDesign record from the read-only coverage-audit workflow (6 auditors): benchmark\ninventory + well-known suites + dashboard + llvm-cov test coverage + test gaps.\nresearch/coverage-and-benchmark-plan.md captures the full design; 28 child beads\ncreated under sq-5o5 (benchmarks) and sq-bif (tests) with priorities from the audit.\n\nKey findings: package bench coverage already strong; real gaps are an operator-\ncoverage bench, a few packages (shacl/nlq/mpc/py/wasm), and the well-known suites\n(SP2Bench now-feasible; WatDiv/BSBM/LUBM/DBPSB generators -> EC2/nightly). Tests:\n14/23 crates >=85% lines already; honest ceiling is a ratcheted floor + presence\ngate, not literal 100%. The #1 flagged gap (zk-compose e2e.rs broken) was already\nfixed earlier this session.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T00:16:35Z",
          "tree_id": "fef36e8a622cfcb0b37a518a848284e610abade3",
          "url": "https://github.com/jeswr/sparq/commit/591a931549ad3896b655d82c86f62fc98f474df0"
        },
        "date": 1781396309186,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.534,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9721,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3258.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4847.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 813,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13305.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 62159.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 163323.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2737.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42387.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8192.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 60018.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 162093.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3189.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39820.5,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.141,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579288,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "b9418f0891b9a35e929673f7e12a0b6724357324",
          "message": "chore(beads): close operator-bench + serializer-oracle + builtin-error-table (sq-ca4/cu8c/nge6/6qkq/w7vh) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T00:55:26Z",
          "tree_id": "02f3b7f985c36a72b66a7a29a587daf1933b3ea1",
          "url": "https://github.com/jeswr/sparq/commit/b9418f0891b9a35e929673f7e12a0b6724357324"
        },
        "date": 1781398684089,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.543,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9721,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3255.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4773.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 820.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12818.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 61377.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 169878.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 5835.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42774.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7586,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58967.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 156010.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2637.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39574.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29880.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 3059503.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6368.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3991.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3702.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7300.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 476760.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 13043.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31445.8,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 54111,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 4039.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22483.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 149156.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 113699.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 186955.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 37623.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7615.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13508.6,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29978,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 19,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 2856109,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6273.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3845.8,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3720.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9330.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 479042.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 13051.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31044.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53624.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 4001.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22507.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 147378.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 112406.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 195288.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 39619.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7112.5,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13348.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29575,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 2739572.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6415.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3940.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3695,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9758.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 482952.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12835.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31646.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53782.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3958.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22275,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 151411.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 113201.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 198644,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 37758,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6746.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13313.4,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "ef910ed891333eb79cecec511edde8e6065dd931",
          "message": "chore(beads): close dashboard + SP2Bench + SHACL coverage (sq-apq/0jp/qap0) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T01:29:21Z",
          "tree_id": "5370da9c0ed2a8be61cc90b11b303e420ab588a9",
          "url": "https://github.com/jeswr/sparq/commit/ef910ed891333eb79cecec511edde8e6065dd931"
        },
        "date": 1781400731874,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.543,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9336,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3247.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4830.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 830.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12849,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 58580.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 151935.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4165.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41150.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7370,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55323,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 151640.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3117.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37394.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29398.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1503946.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6172.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3875.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3658,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7200.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 481496.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12739.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 30390.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52805.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3984.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22482.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 134041.3,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 101691.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 170767.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36716.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7423.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13046.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29882.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1490850.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6385.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3863.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3730.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8409.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 475537.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12941.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30357.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53535.6,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3870.5,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22092.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 133185,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 100810.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 169124.7,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36546.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7060,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13301.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29457.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1482596.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6345.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3865,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3707.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8250.9,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 480109.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12798.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 29975.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 52615.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3873.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21977.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 133719.7,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 101831.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 168431.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 36592.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7071.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13006.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6570.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16408.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16009.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 15806.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 446046.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17183.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24071.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 292819,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22840.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 23688.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 293381.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9247.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19450.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16273.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 15915.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 492575.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18153.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24293.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 290509.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 23010,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 62.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23970,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 290740.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12505.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 19096.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 15979.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 15834.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 491066.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 18991.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24217.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 300050.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23018.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 110.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 23036.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 294193,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.14,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "6a87d3ff1693584f139c13afedb4509975a114da",
          "message": "chore(beads): close P1 test gaps (sq-qu8o/t267/nuok); 3 bugs found+beaded (o1wp/hxgb/uu0u) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T02:03:58Z",
          "tree_id": "699547259f8336422151f8c34e0628a9cd233d43",
          "url": "https://github.com/jeswr/sparq/commit/6a87d3ff1693584f139c13afedb4509975a114da"
        },
        "date": 1781402807400,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.551,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 5.0106,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3131.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4474.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 818.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12702.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56683,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 148621.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4335.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40397.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8291.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57572.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 160343.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2259.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39820.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28621,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 14.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1291990.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6150.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3671.8,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3404.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7336.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 504695,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12252.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31981.8,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52834.6,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3686.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21384.8,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 129673.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 92207.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 157187.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35401.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6364.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13291.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28555.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1276998,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6262.8,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3796.8,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3429.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8742.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 504596,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12411.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31784.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53028.6,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3739.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21260.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 123937.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 90644.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 153280.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 34225.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6743.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12698.3,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 28719.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1342241.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6107.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3681.6,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3413.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8950,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 510479.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 11864.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 32328.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 52814.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3706.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21253.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 128217,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 90682.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 152937.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 35395.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6662.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12391.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6233.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15259.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14967.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14783.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 424923.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15458.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 21828,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 290042.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20419.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21850.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 284141.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8448.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 17473.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14849.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14483.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 471100,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16311.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 21812.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 281135.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20180.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 60.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 21741.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 279795.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 13.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 11743.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 17894.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14909.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14742.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 468795.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16754.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 21673.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 283569.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20506.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 117,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 23021,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 278066.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.137,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "68587dd93231b0b7aa23409a14b4d6bbf9f29821",
          "message": "chore(beads): close coverage gate sq-hbg7 (follow-up sq-bjct) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T02:34:02Z",
          "tree_id": "3d6c29161dfdd0df0bc26266e03c94ea38a3e075",
          "url": "https://github.com/jeswr/sparq/commit/68587dd93231b0b7aa23409a14b4d6bbf9f29821"
        },
        "date": 1781404621912,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.548,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 5.0877,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3130.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4449.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 811,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12498,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59395,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 155790,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4984.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41568.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8042.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58878.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 157956.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4644,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 41498.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29372,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 2159642.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6416.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3810.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3519.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7518.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 509252.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12421.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 32555.5,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53187.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3693.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22350.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 140502.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 103065.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 181269.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35892.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6327.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13284.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29089.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 22,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 2035458.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6457.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3794.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3539.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9615.1,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 507082.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12686.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 33224.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53852.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3909.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22345.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 131181.2,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 97240.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 172581.7,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 35981.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6447.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13051.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29225.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1612204.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6358.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3878.7,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3490.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9463.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 505125,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 11863.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31789.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 52889.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3790.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21759.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 134444.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 95904.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 165074.3,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 35029,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6666.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13024.6,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6255.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15553,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 15342,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 15119.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 448099.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15405.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 21860.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 290610.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20667.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22749.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 292255.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9429.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 17702.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 15050.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 15135,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 475810,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16504.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 21939,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 284771.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20705,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 64.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22907,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 289347,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12031.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 17488.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14717.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14548.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 479275,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16690.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22063.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 282442.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20320.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 118.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22410.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 283047.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.142,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "c16a23ead257c8b1fa9d16db3053ace170cceb16",
          "message": "chore(beads): close P2/P3 test oracles (sq-fj7a/5enp/bv5i/q50l); fixes + 3 new beads [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T02:47:40Z",
          "tree_id": "66104ec482dde17816665101b6a3d06b10a13ec1",
          "url": "https://github.com/jeswr/sparq/commit/c16a23ead257c8b1fa9d16db3053ace170cceb16"
        },
        "date": 1781405428476,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.538,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9336,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3078.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4329.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 749.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12297.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55443.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 148751.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2494.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40367.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9017.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 59014.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 158927.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2599.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38953.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28661,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1357040.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6124.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3730.6,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3370.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7396.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 505502.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12209.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31041.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53576.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3642.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21184,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 126711.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 89284.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 152457,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35875.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6870.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12632.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28118.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1239840.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6581.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3634,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3370.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8423.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 508701,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12314.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31112.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53280.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3645.3,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21015.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 129689.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 89763.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 156363.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 35592.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6424.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12529.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29036.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1437149.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6276.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3728.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3402.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9107.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 512949.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13030.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31582,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53782.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3786.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21503.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 131149.2,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 93261.8,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 157378.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 36344.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6447.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12642.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6198.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15202.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14718.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14781.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 436188.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15317.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 21968.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 286616.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20700,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21142.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 285845.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8404.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 15914.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14660.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14610,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 473206.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16521.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22141,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 285818.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20278,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 58.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 21509.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 288084.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12522.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 18931.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14682,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14507.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 475003.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16944.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22407.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 291491.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20535.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 136.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22049.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 288462.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "56e966748abb620087a95afa9d069bffbd33ca41",
          "message": "ci(wasm): guard the wasm dependency graph against native-only deps (sq-9qz6 pt1) [OPUS-4.8]\n\nEnforces the previously comment-only invariant that flate2/zstd/rayon/bzip2/\nsparq-parse/tokio/mio never enter sparq-wasm's wasm32 dependency graph (bundle\nbloat / broken browser build). scripts/wasm-deps-guard.sh + a wasm-job CI step;\nverified clean against the current tree and negative-tested. The headless\nwasm-bindgen-test run (needs a wasm runtime) remains on sq-9qz6.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T02:50:05Z",
          "tree_id": "efb69fedca12d733972b9c1133115c47dfcfe012",
          "url": "https://github.com/jeswr/sparq/commit/56e966748abb620087a95afa9d069bffbd33ca41"
        },
        "date": 1781405602757,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.53,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.895,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3080.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4360.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 749.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12436.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55031.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 145956.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2773,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 39809,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8711.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57039.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 160429.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4057.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39067,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28630.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 20.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1215247.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6145.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3673.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3275.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7327.1,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 507792.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12523.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 30642.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52926.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3595,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21229.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 129258.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 89946.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 152544.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 34998,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6359.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12787,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28026.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1238962.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 16.4,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6178.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3708.7,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3428.7,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8572.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 514556.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12260.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30737.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53282.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3721.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21122.8,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 126276.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 90124.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 154688,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 34474.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6439.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12303.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29078.1,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1257757.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6063.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3610.6,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3339.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8492.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 503988.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12842.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31689.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53536.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3607.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21221.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 122099.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 91237.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 156215.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 34364.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6284.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12873.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6201.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15304.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14897.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14679,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 424534.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15205.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22306.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 291269.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20546.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21122,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 289191.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8564.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16162.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14856.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14611.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 469476.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16048.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 21826.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 285794.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20156.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 58,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 20956.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 285613,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 13.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12276.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 17913.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14628.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14504.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 470993.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16785.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22052.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 287615.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20490.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 136.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 20981.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 289080.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.141,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "d90635898baecfe46dfd09b8694b076e61726b3e",
          "message": "ci: nightly tier runs only if HEAD changed since the last nightly (sq-8qxl) [OPUS-4.8]\n\nAdds a nightly-gate job: compares github.sha to the head_sha of the most recent\ncompleted schedule-triggered ci.yml run (gh api), fail-open on lookup error/no\nprior run; workflow_dispatch always runs. coverage-nightly now needs: nightly-gate\n+ if: fresh == 'true', so the heavy nightly tier no longer re-runs on an unchanged\nrepo. Future bench-nightly tiers reuse the same gate.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T08:47:56Z",
          "tree_id": "458f036aee3521b8d8cf4ef85c3585c747d71047",
          "url": "https://github.com/jeswr/sparq/commit/d90635898baecfe46dfd09b8694b076e61726b3e"
        },
        "date": 1781427055569,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.543,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.8565,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3336.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4736.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 828.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12756.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 57419.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 155786.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4615.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41328.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8034.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58283.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 154284.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2407.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38601.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29370.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.9,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1624868.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6290.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3885.8,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3645.7,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7170.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 482472,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 15801.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 38389.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53115.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3806.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22193.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 136565.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 102975.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 170777.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36250,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6893,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13326.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29823.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1630570.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6442.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3828.7,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3626.7,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8546.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 484945.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12735.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30452,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53578.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3967.3,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22141.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 141624.2,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 102000.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 173423.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36242.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7046.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13157.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29695.1,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1586921.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6163.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3805,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3639.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8363.9,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 475805.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13148.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30880.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53135.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 4047,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22086.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 129185.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 101420.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 170887.4,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 38749.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7033.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13059.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6667.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16442.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16112.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16182.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 452016.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17303.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 23692.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 300698.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22630.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22170.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 298139.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9150.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 17711.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16039.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16177.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 499435.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18178,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24010.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 300341.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 22782.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 59.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22541.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 296327.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13262.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 19695.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16026.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 15991,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 496861.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 19089.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24233.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 302523.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 22811.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 143.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22461.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 303394.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.147,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "f3a915eb487afae731fd0e1ca7e76f8a8c9af844",
          "message": "chore(beads): close DBPSB suite (sq-5mu) + wasm headless run (sq-9qz6) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T08:51:59Z",
          "tree_id": "53f1f8316a77e76cd6aa4a4b65f6da15aafcf8fd",
          "url": "https://github.com/jeswr/sparq/commit/f3a915eb487afae731fd0e1ca7e76f8a8c9af844"
        },
        "date": 1781427310041,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.545,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9721,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3082.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4366.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 750.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12738.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56083.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 152354.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3834.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41378.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9046.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 60062.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 163763.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3556.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40383.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28940.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1818716.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6104.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3655,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3414.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7365.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 497896.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12403.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 32566.8,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 55983.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3704,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21772,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 137871.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 95919.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 166088.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 37805.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6116,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12743.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29040.8,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1900799.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6249.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3811.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3363.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9332.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 502409.9,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12383.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 32179.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 54012.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3574.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22021.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 141002.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 99972.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 172180.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 37005,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6338.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13102.3,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29401.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1785124.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6338.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3695.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3358.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9176.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 495296.9,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12544.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31230.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53844.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3798.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21920.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 149573,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 105329.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 170391,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 36141.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6238.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12800.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6180.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15304.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14679.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14616.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 439615.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15619.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22758.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 295547.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 21644.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 23095.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 296347.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9134.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 17673.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 15008.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14715.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 481911.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16973.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 23213.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 288679.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 21248.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 61.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22247.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 289813.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13765.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 19980.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14916.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14669.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 491228.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 17329.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22735.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 291470.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 21246.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 136.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22072.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 290631.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "3ec3d59a94ef05f78a46f6296aed27613f3fc9dd",
          "message": "chore(beads): close sparq-py parity + FFI bench (sq-f9tu/th1u); dirLangString bug sq-bj7o [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T09:02:33Z",
          "tree_id": "fe5369e5cde113736271cb5dc91331beccdc67c5",
          "url": "https://github.com/jeswr/sparq/commit/3ec3d59a94ef05f78a46f6296aed27613f3fc9dd"
        },
        "date": 1781427958563,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.543,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9336,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3311.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4731.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 824.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13024.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 58084.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 153039.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4415.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40838.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8074.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57974.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 153866.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4462.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38780.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29509.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1925879.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6406,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3857.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3704,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7376.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 478237,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12577.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31031.8,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 54120.6,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3902.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22291.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 150034,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 110994.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 190623.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 37674.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7140.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13494.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29726.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 2635977.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6409.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3904.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3600.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9263.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 483293.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12945.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31262.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53172.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 4067.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22448.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 136416.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 104115.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 185834.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36998.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7132.5,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13289.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 30499.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 19.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 2527593.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6469.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3862,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3722.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9295.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 476526.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13933,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31663,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 54688.6,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3860.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22608.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 148323,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 110568.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 193805.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 37094.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7043.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13325.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6686.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16443.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16458.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16344.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 473433.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17744.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24851,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 296610.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22860.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 23827.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 301031,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9633.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19363.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16235.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16060.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 511288.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 20494.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24722.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 314223.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 23521.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 68.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 24285.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 299070,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 14515.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 22036.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16437.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16102.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 520458.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 19393.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24723.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 298132.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23668,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 131.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 23920.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 298515,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.145,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "daac4b665079bd45e060e787538202d70208acb8",
          "message": "chore(beads): close WatDiv/BSBM/LUBM suites (sq-13i/mvq/oti1) — wired into CI [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T09:38:10Z",
          "tree_id": "a186d3a602c0391fde9f6c3699dd6eab3567e4d3",
          "url": "https://github.com/jeswr/sparq/commit/daac4b665079bd45e060e787538202d70208acb8"
        },
        "date": 1781430156348,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.537,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.8565,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3079.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4363.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 760.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12920.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56379.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 151858.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4587.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40185.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8769.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 60072.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 162281,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2350.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39269.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28404.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 14.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1528126.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6036.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3700.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3379.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7374.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 507082.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12600.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31552,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 54043.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 4000,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21278.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 136199.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 109285.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 180111.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36819.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6418.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12836.4,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29141,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1564906.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6084.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3635.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3297.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9033.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 503154.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12247.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31684.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53599.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3761.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21095.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 125425,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 90925.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 156854.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36677.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7415.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13135,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 28543.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1452341.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6088.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3656.6,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3381.7,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8825.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 503720.9,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12727.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30682.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53624.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3567.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21222.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 127032.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 92920.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 164795.5,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 34574.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6367.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12586.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6224.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15198.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14867.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14860.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 422137.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15756.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22034.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 282364.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20668.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21611.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 288337.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8516,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16225,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14926.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14678,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 450356.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16205.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 21857.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 282349.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20528.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 60,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 21139.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 284725.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12364.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 18466.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 15012.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14713.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 469004.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16642.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 21944.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 284093,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20357.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 135.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 21006.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 285137.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 63,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 104,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 37.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 862,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 111.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 125.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 32.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 23.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1503.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 33.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 127.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 126.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 22.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 28,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 56.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 67.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 78.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 102.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 465,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 159.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 264.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 557.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 45.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 57.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 79.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 78.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 104.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 461.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 168.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 267.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 552.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 45.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 64.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 169,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 88.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 112.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 456.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 184.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 297.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 6.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 553.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 46.6,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 594.2,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 13.8,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 61.7,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 27.1,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 28.5,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2690.9,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3833.2,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22.4,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.145,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "3078f1c7e28934ace6a90a1243f0c4deb2653d50",
          "message": "fix(bench/dbpsb): robust slice fetch (retries+timeout) so DBPSB runs in CI [OPUS-4.8]\n\nThe first Benchmarks CI run showed DBPSB SKIPPED ('pinned slice unavailable'): the\nbare 'curl -fsSL' of the ~120 MB Databus slice (behind a 307 -> downloads.dbpedia.org)\nflaked on the GitHub runner with no retry/timeout, so the guarded hook skipped the whole\nsuite. Add --retry 5 --retry-delay 5 --retry-all-errors --connect-timeout 30 --max-time\n600; sha256 verification still guards integrity, actions/cache persists it after the\nfirst success. (sp2b/watdiv/bsbm/lubm already ran green in CI: 42/48/33/14 metrics.)\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T09:46:00Z",
          "tree_id": "4cb5d22f6166f9e5204b6c8cec642cc417b4fe0d",
          "url": "https://github.com/jeswr/sparq/commit/3078f1c7e28934ace6a90a1243f0c4deb2653d50"
        },
        "date": 1781430579303,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.542,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.895,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3350.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4719.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 814.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12943.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 58418,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 156165.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4375.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40699.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8164.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57239.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 157919.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3308.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38621.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29516,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1505447,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6257.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3847,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3594.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7166.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 481886.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12844.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 30686,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53029.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3835.5,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22197.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 135399.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 105132.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 177445,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36767.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6923.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13053.4,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29922.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1530503.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6185.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3762.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3569.7,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8220.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 481598.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12945.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30349.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53499.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3883,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22068.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 131688.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 104588.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 174370.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36083.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6846.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13137.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29224.8,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1502620.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6178.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3838.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3623.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9199.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 477795.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12884,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30641.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53614.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3852,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21979.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 140021.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 102178.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 172401.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 36040.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7026.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13228.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6665.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16129.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16027.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 15776.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 451878.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17292.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 23802,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 296843.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22644.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22301.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 300367,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9055.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 17773.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16022.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 15889.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 502139.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18128.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24044.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 294355.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 22834.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 65.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22161,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 296016,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13071.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 19712.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 15974.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 15779.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 505988.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 18784.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 23914.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 298729.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 22867.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 128.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22655.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 302573.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 61.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 30.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 99.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 16.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 31.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 915.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 25.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 29.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 101.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 109.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 31.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 14.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1520.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 27.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 124.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 116.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 33,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 16.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 52.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 65.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 72.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 103.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 488.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 171.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 281.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 601.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 43.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 55,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 78.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 80.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 100.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 502.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 181.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 286.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 620.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 45.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 60.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 156.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 77.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 110.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 494.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 193.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 321,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 609.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 43.5,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 624.7,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 66.9,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 28.5,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.4,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2929.4,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4461.7,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 24.4,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "70d16cce1696534f0256a5e9471c722d1b6ef289",
          "message": "merge: integrate origin [OPUS-4.8]",
          "timestamp": "2026-06-14T09:49:55Z",
          "tree_id": "605758e3886b35041d1ffbaf9dac941c6df2524f",
          "url": "https://github.com/jeswr/sparq/commit/70d16cce1696534f0256a5e9471c722d1b6ef289"
        },
        "date": 1781430782510,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.544,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9336,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3363.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4734.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 812.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13120,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 58379,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 157928.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2661.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42027.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8127,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58014.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 156812.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3386.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39426.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29663.8,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 14.8,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 2058571.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6624.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3858.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3714.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7358.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 480491.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12928.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31017.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53476.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3815.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22267,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 142588.7,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 104842.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 179119.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36255.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6939.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13235.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29864.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 2154544.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 20.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6274,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3791.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3672.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8968.1,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 477305.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12819.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30804.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53617.6,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3905.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22272.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 141868.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 105022.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 178721.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36765.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6910.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13198.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29507.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 2090604.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6375.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3831,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3637.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9327.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 476484.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12792.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30888.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53458.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3799.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22073.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 140083.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 105231.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 179512.3,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 35763.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6966.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13043.6,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6705.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16173.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 15913.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 15964.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 461456.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17784.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24269.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 300132.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22662.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22713.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 295155.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9326.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 18137.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16007,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16065,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 501164.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 19013.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24100.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 298618.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 22897,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 54.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23059.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 301952.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13835.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20924.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16043.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 15995.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 510905.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 19143.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24352.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 301816.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23210.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 132,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 23160.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 297394.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 60.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 31.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 27.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 98.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 21.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 30.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 927.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 25.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 26.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 104.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 15.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 106.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 31.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 18.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 14.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1532.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 27.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 135.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 113.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 20.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 56.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 64,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 73.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 106.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 500.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 172.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 282.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 595.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 44.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 58,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 89.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 73.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 99.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 495.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 177.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 284.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 604.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 43.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 58.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 159.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 74.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 104.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 492.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 193.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 316.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 601.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 43.2,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 612.7,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.7,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 68.9,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.8,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 30.2,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2956,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4034.3,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 23.9,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.148,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "937d9fd544067ba82954191b78777d4f4deb7285",
          "message": "merge origin [OPUS-4.8]",
          "timestamp": "2026-06-14T10:10:15Z",
          "tree_id": "5e283a3f815c14b7acd3338a77b32f0e577402eb",
          "url": "https://github.com/jeswr/sparq/commit/937d9fd544067ba82954191b78777d4f4deb7285"
        },
        "date": 1781431997221,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.545,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 5.0492,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3260.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4827.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 812.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12830.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59786.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 162144.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2748.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41922.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7172,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58054.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 150751.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2356.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37210.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29996.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1789131.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6330.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3722.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3536,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7353.1,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 480588.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12775,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31047.8,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53563.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3751.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22647.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 141709.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 103000.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 180241.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36606.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6901.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13186.3,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29619.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 24.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1674613.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6383.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3787,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3600.7,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9179.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 479767.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12710.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30980.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53207.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3790.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22285.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 146412.7,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 104335.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 177998.7,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36022.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7144.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13068.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29791.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 19,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1883008.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6362,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3798.6,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3595.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8913.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 480248.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13007.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30711.8,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53817.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3830.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22493.8,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 138540.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 103636.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 177929.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 37065.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7298,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13224.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6682.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16114.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16046.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16213.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 434059.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17546.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 23813.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 290146.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22580.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22638.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 290235.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9288.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 18370.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16602.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16177.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 481464.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18399,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24265.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 285531.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 22807.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 64.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23580.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 288950.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12489.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20211.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16111.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 15952.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 470341.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 18530.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24783.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 293151,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 22941.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 93.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22234.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 286692.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 55.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 30.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 30,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 104.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 33.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 916.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 25.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 26.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 109.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 111.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 31.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 23,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1278.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 126.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 21,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 18.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 19.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 123.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 33.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 20.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 16.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 54,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 65.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 74.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 101.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 497.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 173.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 283.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 606.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 44.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 62.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 76.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 76.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 106.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 491.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 180.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 285.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 611.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 45.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 55,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 142,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 76.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 107.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 502.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 198.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 327.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 615,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 44.3,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 622.3,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.6,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 66.7,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 28.6,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.8,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 3012.4,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4091.3,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 21.9,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.139,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1579432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "c95c55752a3d1057b75a8dd1d9e7976b066d3bb2",
          "message": "chore(beads): close compliance/hardening/threat-model/doc-sweep batch + capture fleet-discovered beads [OPUS-4.8]\n\nClosed: sq-0sye/2abm/jfer/biyi/hdfn/zqq6 (supply-chain), sq-3pq5/fvip/rau7/41ey\n(governance), sq-emay (forbid-unsafe), sq-ucwf/kdnj (MSRV/geiger), sq-o9u4 (threat\nmodel), sq-7woa (doc sweep). New beads from the fleet: 2 advisory migrations\n(sq-g2xs/l8bv), 5 threat-model gaps (sq-znld P1 UB / ed2i / v5dg / 2v6f / o4qf),\n8 doc-gap beads, upstream roll-your-own unblocks (sq-fxv3 + updates), sq-4plh, sq-t3rt.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T10:25:27Z",
          "tree_id": "00f8a2606b1fe2407d0e7a0d74cb8dc85047fbef",
          "url": "https://github.com/jeswr/sparq/commit/c95c55752a3d1057b75a8dd1d9e7976b066d3bb2"
        },
        "date": 1781432918645,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.535,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9336,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3080.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4356.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 750.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12473.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56447.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 148112.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3814.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 44029.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8632.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58232.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 154156.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3262.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39980.4,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28777.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1372098.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6071.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3634.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3316.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7337.9,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 499681.9,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12348.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 30905.8,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52337.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3611.3,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21316.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 126754.3,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 97219.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 162863.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35422.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6642.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12700.3,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28575,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1471388.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6469.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3700.6,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3377.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8924.1,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 509710.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12297,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31850.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 52291.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3729.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22203.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 135352.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 93207.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 162910.3,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36091.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6360.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12823.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 28274.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1448080.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6486.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3619.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3312.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8506.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 500222.6,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12281.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31335.8,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 52785.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3675.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21225.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 129918.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 89563.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 150812.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 35551.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6130.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12389.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6179.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 14969,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14583.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14384.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 410433.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15376.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22098.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 291832.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20955,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22032.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 284210,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8428.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16536.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14583.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14572.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 457466.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16383.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 21943.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 288453.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20567.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 60.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 20903.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 282041.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12421.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 18027.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14846.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14565.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 464640.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16579.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 21871.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 283905.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20502.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 133.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 21010.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 282579.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 61.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 31.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 100,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 16.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 38.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 856.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 31.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 105.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 121.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 31.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1515.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 140.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 135,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 22,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 56.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 70.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 77.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 105.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 465.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 161.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 263.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 562.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 46.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 56,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 81.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 78.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 104.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 470.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 168.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 262.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 548.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 46.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 68.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 171.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 92.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 112.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 465.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 184.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 296.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 550.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47.4,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 590.1,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 64.4,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.4,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 28.1,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2697.2,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3837.2,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 23,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.138,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1580432,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "78ae03cb2af9541e8818a7cdd04f924d29087ea8",
          "message": "chore(beads): close wave + CI-fix batch (sq-znld P1/ed2i/ky2a/fxv3/o4qf/qmth) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T12:05:44Z",
          "tree_id": "4fb7d7ec404a322e375329e3a8ad23dde0f439f0",
          "url": "https://github.com/jeswr/sparq/commit/78ae03cb2af9541e8818a7cdd04f924d29087ea8"
        },
        "date": 1781438945786,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.537,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9336,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3079.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4417.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 749.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12928.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55938.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 149876.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2466.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40788.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7786.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55720.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 151606.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3360.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39426.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28732.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1305378.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6132,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3726.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3394.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7405.1,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 495114.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12898.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 33267.5,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53551.9,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3710.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21457.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 127759.3,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 93347.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 154542.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 33835.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7023.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12730.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29086.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1292436.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6071.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3716.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3507.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8219,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 494080.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12255.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 32348.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53513.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3719.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21676.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 125680.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 91379.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 153375.4,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 35617.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6169.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12696.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 28744.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1300635.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6095.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3656.7,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3393.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8848.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 501117,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12242.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31966.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53126.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3728.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21374.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 126264.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 91838.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 155415.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 34333.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6088.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12805.3,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6217.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15405.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14951.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14959.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 431579.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15283.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22213.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 291191.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20287.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21298,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 293089.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8401.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16639.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14797.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14668.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 475785.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16316.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22271.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 286042.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20325.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 59.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 21332.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 286029.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 11512.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 18441.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 15071.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14824.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 471397.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16555.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22568.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 286921,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20471.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 102,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 21838.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 292132.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 64.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 31.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 99.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 37.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 864.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 120.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 15.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 122.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 30.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1290.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 27.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 126.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 19.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 129,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 54.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 69.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 79.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 103.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 477.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 161.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 258,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 547.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 46.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 57.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 79.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 82.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 105.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 472.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 169.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 264.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 545.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 46.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 57.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 143,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 80.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 111,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 486.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 182,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 295.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 556.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 45.9,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 598,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 73.8,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 28.3,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2665.1,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3885.5,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 16.6,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22.1,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.138,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1582149,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "0676cadf09022110bbb2c098b15030a8bab1dcda",
          "message": "ci(supply-chain): split cargo-deny — advisories non-gating (CVSS-4.0 parse lag), bans/sources/licenses stay gating [OPUS-4.8]\n\ncargo-deny 0.19.8 (latest) fails LOADING the fresh RustSec DB on CVSS-4.0 advisories\n(RUSTSEC-2026-0124) — a DB-wide parse error, not a real vuln + not ignorable. Hard-gate\nthe DB-independent checks; advisories continue-on-error + reported; daily cargo-audit\nmonitor is the backstop. Re-enable when cargo-deny supports CVSS4 (bead).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T12:19:48Z",
          "tree_id": "25422e5861d8f1df15d202bab95071fed0658504",
          "url": "https://github.com/jeswr/sparq/commit/0676cadf09022110bbb2c098b15030a8bab1dcda"
        },
        "date": 1781439794449,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.553,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 5.0492,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3341,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4913.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 1556.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13210.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 63685.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 172270.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 6805.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 44182.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7290.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 61952.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 160887.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 6735.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 41754.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 30270.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 3647559.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6406.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3788.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3675,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7570.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 478638.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 13700.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 32298.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53351.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3810.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22870,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 159754.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 120548.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 213316.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 39890.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7304.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13488.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 30539.1,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 3095051.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6394.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3804.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3562.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9143,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 475356.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12852,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31922.6,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 54242.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3789.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22662.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 144473.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 111764,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 185683.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 37206.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7423.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13178,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 30063.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 2746739.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6442.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3890,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3618.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9779.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 482296.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13153.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 32339.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 55119.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3821.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22957.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 148096.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 121572.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 209090,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 15.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 40867.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7466.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 14252.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6906.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 17216.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 17457.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16947.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 482170,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17495.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24594.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 295717.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 23041,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 23272.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 294373.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9533.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19559.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16260.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16123.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 503268.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 19241.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24512.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 296858,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 24874.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 61.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 24329.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 298196.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13475,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 22997.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16951,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16839,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 515643.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 18837.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24076.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 294660.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 22914.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 93.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22819.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 302021.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 66.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 100.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 32.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 916.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 25.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 106.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 109.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 31.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 23,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1256.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 131,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 19.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 112.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 33.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 57.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 68.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 76,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 102,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 542.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 174.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 290.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 611,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 44.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 56.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 81.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 80.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 101.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 507.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 186.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 284.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 602.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 44.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 61.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 144,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 87.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 106.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 506.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 190.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 311.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 610.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 45.4,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 626.9,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.8,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 69.6,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 30,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 30.4,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2998.2,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4096.8,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 23.4,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.5,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.144,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1582149,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "0f4b81b5e04adabe236b4fcf08d9ca231180760c",
          "message": "chore(beads): close HDT write (sq-2te) + GSP write verbs (sq-gxsj) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T12:22:06Z",
          "tree_id": "eacb07c340ab0937d082a28f932017fadadbabb1",
          "url": "https://github.com/jeswr/sparq/commit/0f4b81b5e04adabe236b4fcf08d9ca231180760c"
        },
        "date": 1781439969913,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.535,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9336,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3087.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4407.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 750.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12519.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55651.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 143491.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4402.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40063.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7571,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 55027.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 143334.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2245.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 36941.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28779,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1184070.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6114.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3689.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3400.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7326.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 499175.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12263.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31137,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53584.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3919.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21639.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 126373.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 90853.8,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 153649.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36754,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7112.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12749.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29280.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1198497.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6236.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3703.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3433.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8520.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 497396.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12256.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 32441.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53155.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3716.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21900.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.3,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 131011.3,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 91833.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 156370.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 33704.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6351.5,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12856.6,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 28810.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1204009.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6185.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3753.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3404.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8487.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 505608.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12347.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31166,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 52749.9,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3605.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21478.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 120808.7,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 90429.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 151402.7,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 34481.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6223,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12649,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6202.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15444.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14944.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14883.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 424833.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15335.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22238.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 290768.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20435.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21659.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 285082.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8396.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16178.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14784.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14760,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 471888.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16253.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22245,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 285948.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20279,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 56.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 21148.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 283518.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 11708.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 17521.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 15012.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14948.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 467700,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16454.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22074.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 285121.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20232.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 102.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 21418.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 288022.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 60.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 27.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 103.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 37.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 862.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 119.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 18.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 123.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 31.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 23.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1309,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 125.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 20,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 129.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 31.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 22.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 55.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 67.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 77.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 103.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 470.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 169.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 269.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 545,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 47.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 54.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 85.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 86.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 104,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 472.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 168.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 263.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 554.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 46.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 57.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 153.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 82.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 110.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 480.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 183.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 293.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 546.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47.3,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 600.7,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 62.4,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 27.6,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2686.8,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3895.5,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 16.7,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22.4,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.136,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1582149,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "05f203ab4fa179135a0ee7c764f833e9eff2fe0d",
          "message": "chore(beads): close GML (sq-zy0) + Solid write-ACL (sq-xor3) + engine SSRF (sq-2v6f) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T12:55:19Z",
          "tree_id": "2d0face09c245ef0635f9b1f18a38049638b2674",
          "url": "https://github.com/jeswr/sparq/commit/05f203ab4fa179135a0ee7c764f833e9eff2fe0d"
        },
        "date": 1781441898294,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.544,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 5.0106,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3076.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4423.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 750,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12953.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 57295,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 153229.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2407.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41073.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7737.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 56065.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 151202.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2181,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38569.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29415.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.7,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1221049.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6157.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3721.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3421.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7342.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 495895.9,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12158.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31928.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52214.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3699.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21554.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 127447.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 91753,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 155101.3,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35171.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6835.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12620.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28751.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1190696.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6343.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3734.7,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3421,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8321.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 503374.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12606.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30943.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 52633.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3646.3,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21754.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 121350,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 89958.8,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 150940.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 34475.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6644,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12726.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29242.1,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1189374,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6114.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3716,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3398.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8274.9,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 492627.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12414,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 32700.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 52792.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 4047.3,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21621.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 123824.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 92361.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 159039,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 33471.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7299.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12683.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6164.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15012.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14729,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14641.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 422349.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15267.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22066.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 286432.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20414.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21107.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 283154.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8391.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16466.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 15085.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14785.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 461970,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16262.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22017.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 283989.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20510,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 61.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 21042.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 289770.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 11254.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 17416.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 15001.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14925.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 465264.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16403.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22085.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 289491.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20155,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 104.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22305.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 284357.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 61.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 33.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 104.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 37.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 852.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 107.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 122.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 31.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 23.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1283.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 32.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 126.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 19.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 130.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 56.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 70.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 80.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 105.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 465.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 162.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 260,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 542.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 46.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 56.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 79.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 80.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 111.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 466,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 177.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 270.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 543.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 46.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 62.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 143.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 87.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 109,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 476,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 179.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 292.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 546.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 607.7,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 62,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2676.8,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3883,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 23,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.138,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1582149,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "2686cb7e68ec74a3e807285c286668c29ffc22fd",
          "message": "chore(beads): close Dependabot batch sq-672j (deferred sq-8bhq/sski) [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T13:55:19Z",
          "tree_id": "cf306eb8f34a90dc65f1664155f6a976e849284a",
          "url": "https://github.com/jeswr/sparq/commit/2686cb7e68ec74a3e807285c286668c29ffc22fd"
        },
        "date": 1781445514331,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.537,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.8565,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3136.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4525.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 815.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12563.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56540.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 148027.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3684.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 43483.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7620.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 54889,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 145303,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2128.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 36965.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28976,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1336603.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6251.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3740.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3493.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7434.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 511842.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12102.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 32121.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52581.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3737.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21618.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 126101.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 90991.8,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 153158.4,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 33410.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6626.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12632.4,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28564.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1325848.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6381.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3730.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3462.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9087.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 502122.6,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12770.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31081.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53046.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3831.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21195.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 132330.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 90013.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 153987,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 33567.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6203,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12861,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29350.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.8,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1388678.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6206.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3832,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3495,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8849.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 506418,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12296.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31246.6,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53385.6,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3820.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21515.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 125706,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 91522.8,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 153138.4,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 34302,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6396.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13021.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6215.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15123.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14857.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14506.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 424614.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15278.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22219.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 285960.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20667.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21900.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 289629.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8517.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 15899.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14622.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14381.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 473301.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16468.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22137,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 283837.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20435.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 59.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22720.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 284371.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 11269.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 17220.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14664.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14270.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 473861.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16901.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22460.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 286674.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 21262.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 101.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 23538,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 284358.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 62.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 27.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 101.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 16.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 38.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 879.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 109.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 15.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 120.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 31.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1283.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 127.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 20.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 127,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 31.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 57.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 74.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 78.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 104,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 487.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 161.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 257.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 541.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 47.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 60,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 84.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 77.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 103.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 482.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 170.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 264.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 547,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 47,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 62,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 161.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 79.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 117.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 492.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 182.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 298,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 555.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47.8,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 582.2,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 61.4,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 28,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.1,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2711.9,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3861.1,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.143,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1583491,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "committer": {
            "email": "jesse@jeswr.org",
            "name": "Jesse Wright"
          },
          "distinct": true,
          "id": "ac97475b7107daadebc031a9efeaec84aa67fd2c",
          "message": "chore(beads)+docs: close feature wave (sq-cvug/hajq/biss/rt6v) + reconcile-unnotified-agents note [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T14:30:20Z",
          "tree_id": "8cd5b16585b079edc0a6b78032c83c039ac94702",
          "url": "https://github.com/jeswr/sparq/commit/ac97475b7107daadebc031a9efeaec84aa67fd2c"
        },
        "date": 1781447636091,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.545,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9336,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3091.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4361.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 750.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12763.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 67341.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 159464.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3173.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41894.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8014.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 59274,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 155110.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4178.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39294,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29388.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1840250,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6352.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3799.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3421.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7296.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 518450.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12611.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 33587.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53628.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3670.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21891.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 132544.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 95850.8,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 177132,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36228,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6690.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12929.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28931.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1995389.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6108,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3776.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3546.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9051.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 516532.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12445.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 32092.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53065.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3734.3,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22370.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 143719.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 100778.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 172429.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36177,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6448.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13148,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 28999.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1794988.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6450.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3884.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3515.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9463.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 514013.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12794.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 34264.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53310.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3821.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22173.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 145335.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 102904,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 182434.5,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 38600.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6863.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13239.4,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6244.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15231.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 15279.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 15427,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 442146.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 16021.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 23116.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 289114.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22100.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 23320.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 287818.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9587.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 17672.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14863.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14823.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 485412.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16640.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22831.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 287177.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20643.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 61.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22848.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 287898.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13165.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20298.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14913.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 15128.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 487103.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 17370.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 23803.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 286237.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20878.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 117.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 23278.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 288629.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 61.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 104.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 38.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 883.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 27,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 121.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 127.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 30.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 23.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1395.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 32.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 124.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 20.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 126.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 31.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 57,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 71.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 87.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 101.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 480.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 160,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 260.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 549.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 47.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 61.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 81.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 77.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 106.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 469.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 171.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 268.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 548.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 47.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 60.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 162.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 82,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 111.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 473.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 191.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 303.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 564.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 51.3,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 579.2,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 85.8,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 30,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2732.5,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3868,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 18.8,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 26.1,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.138,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1581928,
            "unit": "bytes"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "63333554+jeswr@users.noreply.github.com",
            "name": "Jesse Wright",
            "username": "jeswr"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "1fd33c73592b255c072f823f24c841d5681c3865",
          "message": "fix(coverage): green main — dict-spill coverage tests + explicit mmap in coverage gate (#29)\n\n* test(sparq-core): exercise dict-spill pipeline + defensive paths (coverage ratchet) [OPUS-4.8]\n\nThe dict-spill external-build module (dictspill.rs) was the dominant\ncoverage drag at 38.90% line coverage: the whole end-to-end spill\npipeline (SpillInterner::intern_batch, consolidate, remap_staged,\nShardWindow) plus the config/resource surface (SpillConfig::from_env,\nensure_disk, free_disk_bytes) was unexercised.\n\nAdd:\n  - lib.rs: dict_spill_build_byte_identical_to_sharded — drives the\n    SPILLED build (build_external_spill) with a tiny 64 KiB budget so the\n    per-shard dedup caches overflow and clear at batch boundaries (the\n    EPOCH path) and the external sorts spill across many runs, then asserts\n    the on-disk dictionary/permutation/numeric/temporal files are\n    BYTE-IDENTICAL to the in-RAM sharded path's. This validates the\n    module's central correctness contract, not just line counts.\n  - lib.rs: dict_spill_rejects_non_ntriples_and_opens_mmap — the\n    documented N-Triples-only restriction + mmap read-back.\n  - dictspill.rs unit tests: SpillConfig detected/from_env gate+overrides,\n    ensure_disk above/below floor, parse_term all-shapes inverse,\n    serialize triple-term panic guard, MinSeqPair/SeqFinal/HashPair\n    read/write roundtrips, read_full clean-EOF vs truncation error,\n    TableBuilder dedup, ShardWindow cross-epoch map + out-of-window\n    fail-closed, ShardState resolve cache/spill.\n\nTests only; no production behavior change.\n\nsparq-core --features dict-spill line coverage: 82.88% -> 91.23%\n(dictspill.rs 38.90% -> 96.52%, lib.rs 83.38% -> 87.72%), well above\nthe floor of 78. 12 new tests; all sparq-core tests pass; clippy clean.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* chore(beads): docs-website program — 6 children under sq-w9sr (scaffold/wiring/embedding/jscpd/glue/needs-user) + design captured [OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(coverage): measure sparq-core with explicit mmap,dict-spill in coverage gate [OPUS-4.8]\n\nThe per-commit coverage ratchet (scripts/coverage.sh) measured sparq-core with\n`--features dict-spill` only. dict-spill ALREADY pulls in mmap transitively\n(Cargo.toml: `dict-spill = [\"mmap\", \"parallel\", \"dep:libc\"]`), so the on-disk-store\nSECURITY surface this gate is meant to guard — `Dict::open_mmap` validation,\n`CompressedPerm::from_mmap`, and the `tests/mmap_corruption_oracle.rs` integration\ntest (itself `#![cfg(feature = \"mmap\")]`) — already compiled and ran, and the\nmeasured number is 91.23% (>> floor 78), identical with or without naming mmap.\n\nThis commit names `mmap` EXPLICITLY in the coverage feature set + rewrites the\nWHY-comment so the gate cannot silently lose the security-code coverage it exists\nto enforce if a future refactor decouples dict-spill from mmap. No-op for today's\nnumber (measured identical 91.23% either way); a robustness/clarity fix.\n\nDiagnosis (empirical, on current main 82326d5):\n  cargo llvm-cov -p sparq-core --features dict-spill --summary-only      -> 91.23%\n  cargo llvm-cov -p sparq-core --features mmap,dict-spill --summary-only -> 91.23%\n  COVERAGE_CRATES=\"sparq-core\" scripts/coverage.sh (summary lines_pct)   -> 91.23%\nThe reported 65.2% (and the prior hotfix's 73.38%) are NOT reproducible on current\nmain: the 82326d5 dict-spill-pipeline test hotfix already restored sparq-core to\n91.23%. The brief's hypothesis (dict-spill-only doesn't run the mmap security tests\n-> 65%) is REFUTED: dict-spill enables mmap, so those tests already run.\n\nFull per-commit gate verified: coverage-gate.py --check = 23 ok / 0 fail / 0 missing.\nFloor unchanged (no lowering). No other per-commit crate misconfigured (engine/server/\nreason gated features are pure `[]` cfg gates -> compiled out when off, so they do\nnot undercount; self-consistent with their seeded floors).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* test(sparq-core): address Copilot review — panic-safe env guard + non-flaky RAM assert [OPUS-4.8]\n\n- spill_config_detected_has_sane_defaults: only assert RAM positivity when detected_ram_bytes()\n  returns Some (it is documented to return None when sysconf is unavailable) -> no flake.\n- spill_config_from_env_gate_and_overrides: restore env via an RAII Drop guard so the original\n  vars are put back even if an assertion panics mid-test (a manual end-of-test restore leaks\n  mutated vars into other tests that read SpillConfig::from_env on panic).\nBoth addressed Copilot inline comments on PR #29.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-14T17:14:52+01:00",
          "tree_id": "9a1f12a7779222d7cbd604490e8ae47c293fbeaf",
          "url": "https://github.com/jeswr/sparq/commit/1fd33c73592b255c072f823f24c841d5681c3865"
        },
        "date": 1781453903891,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.552,
            "unit": "s"
          },
          {
            "name": "store_bytes_per_triple",
            "value": 92,
            "unit": "bytes"
          },
          {
            "name": "dict_bytes_per_term",
            "value": 53,
            "unit": "bytes"
          },
          {
            "name": "parse_ns_per_byte",
            "value": 4.9721,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3092.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4367,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 754.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12958.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59133.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 155175.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4747.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41027.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8163.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58142.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 163065.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4868.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40090.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29314.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 2471019.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6400.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3824.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3517.3,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7462.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 510444.6,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12460,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 33124.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52654.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3744.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21974.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 147296.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 104738.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 194332.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 38530.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7680.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13141.6,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29557.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 2527782.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6360.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3780,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3517.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9170.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 515129,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 13126.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 35711.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53592.9,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3770.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22329.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 152478.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 106273.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 191748,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 38168.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6897.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13307.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29597.8,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 2746245,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6336,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3827.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3437.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9181.1,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 507662.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12781.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 33231.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53068.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3748.3,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22045.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 147447.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 101954.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 185739.5,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 39262.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6616.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13449.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6291.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16375,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 15900.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 15874.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 446383.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15637.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 23145.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 283688.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 21645.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 23535.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 286517.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 10114.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19251.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 15624.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 15252.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 490279.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16946.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 23407.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 291867.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 21663,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 60.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23554.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 289586.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13236.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 21012.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16161.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 15872.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 492328.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 17538.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 23245.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 282424.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 21664.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 118.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 24499.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 286118.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 64.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 27.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 106.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 37.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 879.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 107.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 120.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 30.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1399.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 29.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 126.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 20.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 133.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 22,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 58.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 75.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 87.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 103.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 471.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 180.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 268.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 537.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 47.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 61,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 82.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 80.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 104.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 486.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 169.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 265.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 539.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 47.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 61.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 173.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 97.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 109.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 490.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 188.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 300.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 564.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 48.1,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 602.1,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 68,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 31.8,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2715.2,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3905.4,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 24.5,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.139,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1581928,
            "unit": "bytes"
          }
        ]
      }
    ]
  }
}