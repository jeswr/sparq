window.BENCHMARK_DATA = {
  "lastUpdate": 1781377837415,
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
      }
    ]
  }
}