window.BENCHMARK_DATA = {
  "lastUpdate": 1781404622310,
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
      }
    ]
  }
}