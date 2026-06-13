window.BENCHMARK_DATA = {
  "lastUpdate": 1781365821934,
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
      }
    ]
  }
}