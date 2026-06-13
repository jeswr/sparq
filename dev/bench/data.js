window.BENCHMARK_DATA = {
  "lastUpdate": 1781369131468,
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
      }
    ]
  }
}