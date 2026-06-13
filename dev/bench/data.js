window.BENCHMARK_DATA = {
  "lastUpdate": 1781385763390,
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
      }
    ]
  }
}