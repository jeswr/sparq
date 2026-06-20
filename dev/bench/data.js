window.BENCHMARK_DATA = {
  "lastUpdate": 1781962680265,
  "repoUrl": "https://github.com/jeswr/sparq",
  "entries": {
    "sparq engine": [
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
          "id": "2e983668313af8b94ee8fff4ab2ff8d17d0b603e",
          "message": "feat(sparq-core): out-of-core N-Quads/TriG ingest for NAMED graphs (sq-5atq) (#187)\n\n* feat(sparq-core): out-of-core (bounded-memory) N-Quads/TriG ingest for NAMED graphs (sq-5atq)\n\n[OPUS-4.8] `build_external` could only construct a DEFAULT-graph dataset out-of-core;\na billion-scale N-Quads/TriG dataset (the PSS shape) had to be built in RAM then saved.\n\nAdd `Graph::build_external_quads` (mmap): stream the quad document ONCE, PARTITION BY\nGRAPH NAME into per-graph on-disk N-Triples spill files (bounded memory — only the\nparser buffer + small per-graph write buffers resident), then build EACH graph through\nthe existing single-graph external pipeline (external SPO sort, disk-backed runs, k-way\nmerge) into its own directory. Emits the SAME on-disk layout `save_named` produces —\ndefault graph in `dir`, each named graph under `dir/named/<i>/`, committed by `named.bin`\nin FIRST-OCCURRENCE order — so `open()` (mmap) reads the whole dataset back losslessly.\nRe-serialising each triple to canonical N-Triples for the spill is lossless (blank-node\nscope is per-graph). The N-Triples/default-graph path is unchanged.\n\nWire the CLI `build` to route N-Quads/TriG through the quad path (a default-graph-only\n`build_external` would silently flatten every quad into the default graph, losing named\ngraphs); triple formats keep the existing path.\n\nTests (sparq-core): lossless-open of a multi-graph N-Quads corpus at a TINY chunk (the\ngenuine spill path) vs the in-RAM `load_dataset`; differential vs in-RAM `save`/\n`save_named` (per-graph quad set + manifest ordering); edge cases — one-quad graph,\ndefault mixed with named in one stream, duplicate quads across graphs, default-only\n(no manifest/subtree), empty default graph; TriG smoke test. CLI: build N-Quads ->\nquery-mmap `GRAPH <g> {}` returns that graph's quads end-to-end.\n\nScope: N-Quads + TriG both landed (TriG reuses the identical partition + per-graph\nexternal build, only the parser differs).\n\nVerify: cargo build/nextest (104/104 core --features mmap, 39/39 cli) + clippy -D\nwarnings clean on both crates. (Workspace-wide `cargo fmt` deliberately NOT run — the\nrepo's rustfmt reformat is DEFERRED per rustfmt.toml; clippy is the hard gate.)\n\nRefs sq-5atq; lineage sq-3ui0 (named-graph persistence). NON-CANONICAL timing.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(core): out-of-core quad build — validate-before-delete, spill cleanup guard, bounded writer pool [OPUS-4.8]\n\nResolves 3 Copilot review threads on PR #187 (sq-5atq):\n\n1. DATA-LOSS: build_external_quads removed the existing named subtree/manifest\n   BEFORE validating `format`. Move the format check up-front, before any\n   create_dir_all/remove_dir_all, so a rejected format leaves an existing\n   dataset at `dir` fully intact and returns Err.\n\n2. RESOURCE LEAK: the spill dir (dir/quads-spill/) was only removed on success,\n   leaking huge spill files on any early Err. Add a SpillGuard drop-guard that\n   removes it on every exit path; disarm()+explicit remove on success.\n\n3. FD EXHAUSTION: the partition pass held one BufWriter<File> open per named\n   graph. Replace with a bounded LRU WriterPool (default 256 open, overridable\n   via SPARQ_QUADS_SPILL_MAX_OPEN) that flushes+closes the LRU writer and\n   reopens evicted graphs in APPEND mode, so open FDs are O(cap) regardless of\n   graph count while every quad still routes to the right per-graph spill.\n\nTests: build_external_quads_unsupported_format_preserves_dir,\n_error_cleans_spill_dir, _bounded_open_writers (cap=4, 200 graphs interleaved).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T17:13:49Z",
          "tree_id": "26fdf695cda8f1d77e8ecff7e26c6c6e4f2d8df5",
          "url": "https://github.com/jeswr/sparq/commit/2e983668313af8b94ee8fff4ab2ff8d17d0b603e"
        },
        "date": 1781543892656,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.56,
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
            "value": 3385.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4852.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 797.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13267.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 62511,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 168379.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4287.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 44604,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8450.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 62717.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 177664.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2948.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 41930.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 30024.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 2304596.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6443.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3892,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3500.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7251.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 474148.6,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12947.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 32491.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 54206.9,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3773.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22609.8,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 163223.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 113681.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 192305.4,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.5,
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
            "value": 37356,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7184.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13227,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 30584.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 18.8,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 2777958,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6510.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3862.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3658.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9517.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 481678.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 13506.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 33001.6,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 54825.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 4133.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 24905.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 151554.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 111381.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 191760,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 38090.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7366.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13354.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 30488,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 21.4,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 2242283.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6358,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3752.6,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3587.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9025.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 477115.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 14383.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 33254.6,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 55468.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3916,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22986.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 153276.2,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 105916.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 184807,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 36083.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7301.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13428.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6677.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16696.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16258.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16121.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 446126.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17546.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24154.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 292124.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22714.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22270.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 289968.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 15.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9275.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19713.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16326.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16471.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 501096.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18766.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24585.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 301426.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 22973,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 68.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23487,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 294014.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 14034.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 21045.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16433.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16210.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 510143.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 20303.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24184.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 291697,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 22714.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 130.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22621,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 295874.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 60.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 30.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 99.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 31.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 13.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 930,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 25.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 26.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 106.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 15.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 112.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 36.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1576.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 27.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 124.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 130.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 35.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 65.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 64.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 72.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 106,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 491.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 173.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 291.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 606.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 45.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 59.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 78.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 74.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 106.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 492.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 180.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 288.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 601.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 45,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 61.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 159,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 75.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 106.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 489.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 197.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 327.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 601.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 44.9,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 604.7,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 70.4,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.6,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 30.9,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2964.9,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4092.1,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 18.5,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 24.8,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.5,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.076,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 339.6,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 308.5,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13309.6,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6608.1,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 682701,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.147,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "602c77200fa6574c6226458c081730585641a316",
          "message": "feat(bench): full-text-search benchmark suite (sq-ustq) — bench/fts/ (#186)\n\n* feat(bench): full-text-search benchmark suite (sq-ustq) — bench/fts/ mirroring SHACL template\n\n[OPUS-4.8] Stamps the LUBM/SHACL per-surface template onto sparq-text (design §3.4):\na self-asserting deterministic gate + featured dashboard row + honest competitor wiring.\n\nTwo axes:\n- Latency axis (engine surface): rewrote crates/sparq-text/examples/bench_text.rs to the\n  G1 `name\\tcount\\tus` contract — N synthetic 8-word literals over a ~10k-term Zipf vocab,\n  positions-enabled BM25 index, AND/OR/prefix/phrase/near per-query latency. Per-commit\n  N=100000 seed=0 (~4s); heavy/latency tier N=1000000.\n- IR-quality axis (BEIR Recall@100/nDCG@10): GATHER-ONLY, not yet wired (corpus not\n  redistributable in-repo). Tracked as follow-up bead sq-1fz0 (blocked on sq-ustq).\n\nDeterministic gate (HARD, in bench/fts/run.sh, exit 1 on drift): total hit count over a\nFIXED 200-query set (drawn from an INDEPENDENT seed so counts shift only on search-semantics\nchanges) + integer index bytes-per-doc. Derived BY RUNNING sparq-text on the pinned corpus.\nbytes_per_doc also gets a mode:auto ratchet (fts_bytes_per_doc in bench/perf-baseline.json).\nProved the gate gates: perturbing any expected count -> exit 1.\n\nTiming advisory (mode:noise, NEVER gated, non-canonical dev box): text_<workload>_us +\ntext_build_s harvested by the guarded ci-bench.sh hook (cargo-only guard; PRs skip).\n\nDashboard: rebased onto origin/main (only a beads re-export had landed — geo PR #184 not\nyet merged, no conflict), then added the Full-Text row to FEATURED_SUITES + GROUP_ORDER +\ngen-metric-labels.py (regenerated metric-labels.json). Registry/catalog updated\n(text-index-bench now the self-asserting FTS suite).\n\nCompetitors (HONEST): Solr/ES are NOT SPARQL competitors and are kept OFF the dashboard.\nRegistered (engines/values empty per AGENTS.md): jena-text (Fuseki + jena-text Lucene SAIL,\nhttp-sparql, the only like-for-like FTS-over-SPARQL — dashboard peer) + lucene-anserini\n(embedded Lucene BM25 kernel ref via Anserini on BEIR, python-lib, dashboard_engine_id=null\n= labelled sub-component, off-dashboard).\n\nVerify: cargo nextest run -p sparq-text (58/58) + clippy --workspace --exclude sparq-py\n--all-targets -D warnings clean; dashboard-smoke + gen-metric-labels --check + perf-gate\n--self-test all pass; perf-gate confirmed gating fts_bytes_per_doc (+7.8% -> fail).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(bench-fts): resolve Copilot review — no-arg reproduces pinned corpus + PR-tier skip [OPUS-4.8]\n\nResolves the 6 Copilot threads on PR #186 (sq-ustq):\n\n- bench_text default seed 20260612 -> 0 and default N 1_000_000 -> 100_000, so a\n  no-arg `cargo run --example bench_text` reproduces the pinned gen.sh corpus\n  (N=100000, seed=0) and the committed expected.tsv counts. 1M heavy/latency tier\n  stays opt-in via the first arg. Doc-comment usage updated.\n- bench/fts/README.md: advisory-timing metric names now match what the ci-bench\n  hook actually emits (text_<workload>_us: and_terms/or_terms/prefix4/phrase/\n  near_slop2, plus text_build_s) — not the stale text_and_us/text_prefix_us.\n- scripts/ci-bench.sh: gate the FTS hook to the MAIN tier (GITHUB_REF=refs/heads/\n  main, or unset for local runs), SKIPPING the PR tier — mirroring how the\n  javac/rapper (LUBM/SHACL) suites avoid per-PR example builds (their toolchains\n  are main-only). Keeps the bench_text release compile + N=100000 corpus off every\n  PR build. Comment + else-branch skip message corrected to match the new logic.\n\nVerified: no-arg run + bench/fts/run.sh gate match expected.tsv; cargo nextest\n-p sparq-text (58/58); clippy --workspace --exclude sparq-py --all-targets -D\nwarnings clean; shellcheck (no new findings); dashboard-smoke.js passes.\nNON-CANONICAL timing (dev box).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T17:18:35Z",
          "tree_id": "215585412c4c1ce528fdec3eb3a3c83927d3ce92",
          "url": "https://github.com/jeswr/sparq/commit/602c77200fa6574c6226458c081730585641a316"
        },
        "date": 1781544177566,
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
            "value": 3104.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4352.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 816.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12485.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56655.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 148625.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2897.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40077.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9137.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 60844.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 163129.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2810.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39758.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29117.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1374113.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6121.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3665.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3339,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7184.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 501164.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12147.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31881.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53075.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3558,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21508.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 13.3,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 127553.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 90416.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 154013.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11,
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
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 34476,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6026.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12660.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28768.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1420192.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6069.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3643,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3395.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8343.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 504420,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12173.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 32629.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 52560.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3673,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21432,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 127215.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 92461.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 155386.3,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 33852.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6728.5,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12902.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 28790,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 19.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1436189.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6141.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3641.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3347.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9267.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 496012.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12155.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 32585.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53166.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3656.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21421.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 133068.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 91124.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 156293.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 33957.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7145.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12582.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6243.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15144.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 15002.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14703.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 415426.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15303.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22224.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 283742.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20563.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22182.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 288754.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8395.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16842.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14938.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14707.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 449484.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16221.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22008.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 277825.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20708.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 56.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 21165.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 282553.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 13.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12549.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 18796,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14936.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14629,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 468460.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16790,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22372.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 283271.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20898.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 136.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 21484.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 284634.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 64.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 36.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 100.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 39.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 14.1,
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
            "value": 862.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 108.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 125.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 30.7,
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
            "value": 22.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1528.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 141.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 145,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 31.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 22,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 69.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 68.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 78.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 111.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 468.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 168,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 268.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 537.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 47,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 57.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 82.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 78.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 105,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 458.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 173.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 269.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 550.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 47.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 59.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 170.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 82.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 111.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 461.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 186.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 305.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 562.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47.5,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 592,
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
            "value": 61.7,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 27.4,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2643,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3864.9,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 16.7,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.061,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 359.2,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 327.7,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 12884.3,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6747.6,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 757567.9,
            "unit": "us"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.9,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.5,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.5,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3649.2,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.454104,
            "unit": "s"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.138,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "96df2cf343265ab1233845bd845ffa74335a39bb",
          "message": "feat(zk-compose): join_eq proving path + N-way chain + real bb accept (sq-r2s8) (#188)\n\n* feat(zk-compose): join_eq proving path + N-way chain + real bb accept (sq-r2s8) [OPUS-4.8]\n\nCompletes the hidden cross-credential JOIN end-to-end (sq-bwwl epic): the\n`join_eq` PROVING path that sq-fi03 left as an `Err` and the full-bb accept\nthe sq-sfsi `bind_joins` suite `#[ignore]`'d because no real proof existed.\n\n- `build::build_join` — assembles the join_eq witness (enc_a/counts_a/enc_b/\n  counts_b/row_a/row_b/blinding) from the two GraphCommitments + the two\n  query-bound slots, locating the shared-value rows and binding the hidden\n  value under a per-presentation blinder into the public hiding\n  `join_commitment` (mirrors build_scan; single-source-of-truth with the\n  in-circuit join_value_commitment).\n- `toml::join_prover_toml` + `prover_toml_for`'s JoinEq arm — replaces the\n  `Err` with real witness-bearing TOML emission (pads enc_a/enc_b to the\n  member buckets, mirrors the scan arm). A JoinEq input WITHOUT its\n  JoinWitness returns the recoverable `JoinEqMissingWitness` (no panic in a\n  public fn). New trailing `join_witness: Option<&JoinWitness>` param; all\n  non-join callers pass `None`.\n- `full_bb_join_accept_real_proof` (was the `#[ignore]`d-empty\n  `full_bb_join_accept_deferred`) — generates a REAL join_eq bb proof (plus\n  the two real scan proofs) and asserts it verifies end-to-end through\n  `verify_manifest`: audit-#1 public-input reconstruction, audit-#2 CANONICAL\n  VK by CircuitId::JoinEq (enforcement point 2, proved end-to-end with a\n  genuine proof), bb verify, audit-#4 nonce binding, AND the bind_joins gate.\n  RUNS+PASSES on a box with nargo+bb via `--run-ignored all`; `#[ignore]`'d\n  off the per-PR fast path (heavy: three real bb proves), runs in the\n  zk-toolchain/nightly lane like full_manifest_prove_verify_scan.\n- N-way chain (design §2.4) — bind_joins now enforces that JoinEdges joining\n  the SAME query variable carry byte-equal `join_commitment`s\n  (JoinCommitmentChainMismatch), so a multi-hop join composes transitively\n  without disclosing the value. Structural accept (honest 3-way shared\n  commitment) + reject (divergent commitment) tests added.\n\ncargo build / clippy --all-targets -D warnings clean; nextest 236 pass (30\nskipped); the un-ignored full-bb accept passes with a real proof. Timing on\nthis box is NON-CANONICAL.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* test(sparq-zk-compose): use valid distinct hex for divergent-commitment case [OPUS-4.8]\n\nReplace the malformed FieldHex \"0xdiff\" (non-hex 'i') in chain_manifest's\nforge branch with a valid but different field element (0xdeadbeef). The\nnway_chain_divergent_commitment_rejected test now exercises rejection\nspecifically due to divergent join_commitments (JoinCommitmentChainMismatch\non edge 1) rather than a fail-closed malformed-hex parse, making the\nsoundness obligation precise. sq-r2s8.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T17:23:36Z",
          "tree_id": "bea0d7fae88eeeb6c9b58045fb82b8bd46774df8",
          "url": "https://github.com/jeswr/sparq/commit/96df2cf343265ab1233845bd845ffa74335a39bb"
        },
        "date": 1781544498799,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.44,
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
            "value": 4.0085,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 2.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 2597.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 3758,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 612.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 10336.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 50314.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 133555.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4053.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 36025.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 6953,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 53716.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 142298.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2683.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 36009.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.2,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 24000.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 13.8,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 3026709.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 5099.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 2972.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 2804.7,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 5655.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 371578.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 9984.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 27520.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 42069.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 2991.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 18313.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 131939.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 105015.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 174621.5,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 32278.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 5595.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 10463.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 23818.1,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 19.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 3171644.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 5201.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3055.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 2864.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 7666,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 373445.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 10682.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 26597.6,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 42706.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3012.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 18294.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 134982.3,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 104578,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 189186.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 31108.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6012.5,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 10538,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 23892,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 3328362.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 5219.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3056.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 2896.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8007.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 372612.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 10369.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 27096.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 43318.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3137.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 18425.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 138237.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 108116.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 191204.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 6.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 30484.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 5925.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 10551.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 5167.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 13121.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 12611.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 12837.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 381267.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 13899.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 19727.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 226313,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 18735.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 19280,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 230840,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8161.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16747.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 13000.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 12931.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 421592.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 15356.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 20361.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 228335.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 18297.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 55.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 18917,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 230701.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 11847.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 18282.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 13313.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 13401.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 432052.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 15010,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 19893.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 234868.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 18965.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 103.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 18537.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 228151,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 47.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 27.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 21,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 90.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 13.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 24.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 728.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 20.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 23.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 87.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 83.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 25.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1210.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 21.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 94.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 96.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 25.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 21.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 46.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 56.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 61,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 86.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 390.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 140.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 225.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 474.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 34.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 45,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 63.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 56.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 80.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 383.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 140,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 229.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 468.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 35,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 46.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 126.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 69.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 86.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 385.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 151.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 261.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 471.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 34.4,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 465.5,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 56.2,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 23,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 23.9,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2315.1,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3136.9,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 20.4,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 14.3,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.005,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.061,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 258.6,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 236.3,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 10402.7,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 5125.9,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 524933.9,
            "unit": "us"
          },
          {
            "name": "text_and_terms_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 0.9,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 0.9,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3133.2,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.447182,
            "unit": "s"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.12,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "f9a5df270e8431bb69dd6fa117d0e3d2d7cab6f2",
          "message": "fix(sparq-engine): resolve intra-body CLEAR/DROP against running state in durable journal (sq-aalh) (#189)\n\n* fix(sparq-engine): resolve intra-body CLEAR/DROP against running state in durable journal [OPUS-4.8]\n\nThe durable multi-op UPDATE path (`apply_effects`) builds its redo-journal\nframe once up front via `resolve_effect_records`, which resolved a CLEAR/DROP\nagainst the PRE-BODY graph state. So for an intra-body sequence like\n`INSERT DATA { GRAPH X { ... } } ; CLEAR GRAPH X`, the CLEAR computed its\nretraction set from X's state BEFORE the body ran — the quad inserted earlier\nin the SAME body was journaled as an insert with NO matching delete.\n\nThe materialised per-graph state was always correct (the per-effect loop runs\nin sequence), but the JOURNAL FRAME diverged: a crash-recovery redo (open\nreplays a committed-but-not-yet-materialised frame) resurrected the\ninserted-then-cleared quads — a latent durability-correctness bug. PSS orders\nDROP before INSERT, so PSS never hit it.\n\nFix: `resolve_effect_records` now walks the effects maintaining a RUNNING\nper-slot view (seeded from the current graph), and resolves each CLEAR/DROP\nagainst THAT view, so intra-body inserts are correctly retracted in the\njournal. The frame's set-semantic redo now reproduces the true final state.\n\nTests (sq-aalh): journal-frame nets to empty for INSERT;CLEAR and INSERT;DROP\n(reproduce-before/fixed-after); end-to-end crash-recovery (commit_txn, drop\nwithout materialising, reopen) leaves X empty; controls: DROP;INSERT keeps the\nquad (PSS ordering), multi-graph CLEAR X keeps Y; full apply_effects+reload and\nin-memory `update` parity. The journal-net + crash-recovery cases FAIL on the\npre-fix (pre-body) resolution and PASS after.\n\nRefs sq-aalh.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* perf(sparq-engine): lazily decode running view in resolve_effect_records; precise test graph matching [OPUS-4.8]\n\nreview #189 (sq-aalh):\n\n1. resolve_effect_records eagerly decode_triples'd the default slot AND every\n   named slot up-front, forcing a full-dataset scan on every multi-op durable\n   body even on the common delta-only path (no CLEAR/DROP). Now the running view\n   is seeded LAZILY per slot, on first touch, from that slot's current pre-body\n   decoded contents. A delta-only body decodes nothing; only a slot a CLEAR/DROP\n   (or Delta) actually touches pays for its decode. Correctness preserved:\n   slot_mut seeds pre-body triples on first access so a later CLEAR/DROP still\n   retracts pre-body AND intra-body quads; CLEAR/DROP NAMED|ALL still visits\n   every existing named graph (graph.named) plus any intra-body-created slot via\n   named_slots(), decoding each on demand.\n\n2. doc typo: \"failure the bead describes\" -> \"failure the bug describes\".\n\n3-5. crash-recovery + apply_effects parity tests located graphs via\n   to_string().contains(\"/x\")/(\"/y\") (brittle: matches /xyz, format-dependent).\n   Now compare against the exact NamedNode Term with ==.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T17:40:34Z",
          "tree_id": "63d0ee5813a8be7a79e93021693457818d0d0fbb",
          "url": "https://github.com/jeswr/sparq/commit/f9a5df270e8431bb69dd6fa117d0e3d2d7cab6f2"
        },
        "date": 1781545474670,
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
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3329.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4848.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 796.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12814.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 58003.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 155647.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3561.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40910.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8233.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57103.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 160027.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2619.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37885.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 33757.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 19,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1423600.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6168.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3782.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3483,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7135.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 478351.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12545.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31074.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53469.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3751.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22178.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 126897.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 99939.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 167753.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36407.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6758.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12974.4,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29660.8,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1463413.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6201.8,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3738.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3613,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8178.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 473056.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12982,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30305.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 54417.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 4097.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22576,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 133124,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 101382.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 168215,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 34683.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7001.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12959.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29329.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 19.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1420894.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6346.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3795.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3603.7,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8300.1,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 477851.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12781.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30582.5,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53777.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 4010.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22264.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 128722.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 100638.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 166823.4,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 6.7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 36807,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6703.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12940,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6719.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16641.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16262.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16305.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 431763.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17176.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24149.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 284735.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22599.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22488.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 283889.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8940.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 17806.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16234.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16046.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 472986.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18150.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 23973.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 285230.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 22502.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 60.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23091.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 285545.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13064,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20135.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16381.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16287.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 476716,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 18582.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24608.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 283419.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 22478.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 130.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22634.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 283750.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 61.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 103.2,
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
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 31.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 13.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 927.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 25.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 26.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 123.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 15.9,
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
            "value": 10,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 107.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 33.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 19,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1565.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 144.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 112,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 33.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 11.6,
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
            "value": 64,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 73.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 102,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 492,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 180.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 296.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 613.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 45.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 55.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 76.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 74.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 101.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 494.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 178.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 296.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 604.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 44.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 59.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 157,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 75.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 107.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 498.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 193,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 325.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 612.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 44.2,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 630.6,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.8,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 65.7,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 28.6,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.9,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2920.1,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4026.1,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 23.1,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.06,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 342.7,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 307.1,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13017.5,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6508.6,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 659914.4,
            "unit": "us"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.3,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.5,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3878,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.472617,
            "unit": "s"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.139,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "cafe5ff67a71a65c730a1601144354e904d1ffa8",
          "message": "feat(sparq-mpc): batched/vector secret sharing + row-binding (sq-dwb5) (#191)\n\n* feat(sparq-mpc): batched/vector secret sharing + row-binding (sq-dwb5) [OPUS-4.8]\n\nGeneralise the single-scalar `share_private_input` to batched/vector secret\nsharing so the secure aggregate + hidden-value join can range over more than\none private value per holder.\n\n- New `batched` module: `BatchedShares` / `BatchedAuthShares` carriers + the\n  documented `RowBinding` contract (Positional: element i = row i, value-sorted\n  so holders line up by index; Keyed: element i bound to a disclosed row key).\n- `ShamirDealer::share_batch(&[Fp]) -> Vec<ShareVec>` and\n  `MacSession::authenticated_share_batch(&[Fp])` — element-wise sharing, each\n  element on its own fresh degree-t polynomial (per-element privacy preserved).\n- `MpcBackend::share_private_inputs(holder, fragment)` (default = single-scalar\n  fallback; Shamir overrides with the real vector path via a new\n  `extract_integer_vector`).\n- `batched::per_row_sum` — the multi-row secure aggregate (zero-round Shamir\n  linear fold lifted to a vector), the demonstrated end-to-end multi-row path.\n\nTests: batched round-trip (n=3,5,7), vector privacy (<=t shares independent),\nauthenticated batch + alpha-leak guard, row-binding correctness differential\n(per-row secure sum == plaintext), multi-row two-holder end-to-end, and\nbackward-compat (single-scalar path unchanged). 251 tests pass; clippy clean.\n\nThe full batched HIDDEN-VALUE join wiring is deferred (follow-up).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(sparq-mpc): per-row arity check in extract_integer_vector + correct share_private_inputs doc (sq-dwb5) [OPUS-4.8]\n\nextract_integer_vector validated p.vars.len()==1 but read only row.first(),\nso an adversarial PartialResult with desynced per-row arity (extra columns)\nsilently mis-extracted by dropping trailing cells. Now reject any row whose\nlen != 1, failing closed. Add a unit test covering rejection + the well-formed\nsingle-column path.\n\nCorrect the share_private_inputs trait doc: the ordering is NOT by a 'disclosed\nvalue' (the values ARE the secret inputs). Document that the holder sorts by the\nsecret value LOCALLY before sharing, that nothing about the column is disclosed,\nand that the KEYED binding uses a disclosed KEY column, not the secret value.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T18:22:04Z",
          "tree_id": "572a3bbb9f69887e86f5dc749a6e2100dd95bcec",
          "url": "https://github.com/jeswr/sparq/commit/cafe5ff67a71a65c730a1601144354e904d1ffa8"
        },
        "date": 1781547999796,
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
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3328.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4839.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 795.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13097.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 57765.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 155028.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3893.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41252.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8155.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57537.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 155668.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3256.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38466.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29400.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 23.7,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1516981.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6194.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3739.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3546.3,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7202.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 480833.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 13095.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 30502.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 54842.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3723.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22448.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 132349,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 106507.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 175579.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35354.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7091.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12993.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29761.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1508314.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6418.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3789.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3484.3,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8473.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 480470.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12893.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31033.6,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53013.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3799,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22263.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 136520.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 103788.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 178112.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.5,
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
            "value": 35662.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6880.5,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13115.4,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29548.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 20.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1516782.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6208.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3756.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3507.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8671.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 475695,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13030.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31368.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53266.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3760.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 23454.8,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 134990.2,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 100959.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 169498.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 36561.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6576.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13304.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6688.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16918.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16675.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16626.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 436102.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17252.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24296.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 296549.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22929.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22686.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 296202.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9159.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 18432,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16389.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16310.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 483112.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18690.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24320.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 287615.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 22645.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 65.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22056,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 285079,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12990.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20372.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16659.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16311.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 479348.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 18498.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24553.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 289826,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23087.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 129.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22252.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 288438.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 61.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 33.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 108.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 32.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 929.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 26.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 105.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 15.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 14.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 112.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 33,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1541.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 123.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 111.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 34,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 57.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 65.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 77.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 109.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 489.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 178,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 292.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 612.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 44.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 54,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 76.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 73.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 100,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 491.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 183.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 301.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 626.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 44.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 58.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 158.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 74.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 113.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 494.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 197.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 330.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 613,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 45.4,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 615.5,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.7,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 69.1,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 30,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2982.6,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4043.2,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 21.9,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.062,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 352.3,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 308.4,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13050.3,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6630.7,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 674314.1,
            "unit": "us"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.4,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.5,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.2,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.5,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3884,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.478735,
            "unit": "s"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.14,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "7db50c0838a21f9e70b8809fdc812d4cad2c1c05",
          "message": "feat(bench): GeoSPARQL benchmark suite (sq-tf8n) (#184)\n\n* feat(bench): GeoSPARQL benchmark suite (sq-tf8n)\n\n[OPUS-4.8] Mirror the SHACL/LUBM template onto sparq-geo (design §3.5):\noverview dashboard row + self-asserting deterministic gate + competitor wiring.\n\nEngine surface: extend crates/sparq-geo/examples/bench_geo.rs with `gen` (emit the\nfixed CRS84 point corpus) and `bench` (emit name\\tcount\\tus) subcommands over a FIXED\n~100k-point corpus (seed 20260615, 8x8 deg window). The default report mode is unchanged.\n\nbench/geo/ mirrors bench/shacl/: gen.sh (shells out to `bench_geo gen` so expected.tsv\nand the in-process fallback are byte-identical), run.sh (self-asserting), expected.tsv\n(COUNTS derived by running), README.md, queries/*.rq (GeoSPARQL renderings for the\nhttp-sparql competitor).\n\nDETERMINISTIC gate (counts-not-coords; float geometry is not bit-stable): result-set\nSIZE of within10km/within50km/nearest_k10/nearest_k100/geof_within + a geo_compliance_pass\nOGC-topology-fixture ratchet, asserted in run.sh (exit 1 on drift). The compliance ratchet\nis hard-gated cross-commit as the DEFICIT geo_compliance_deficit (= 25 - passing, mode:auto\nin bench/perf-baseline.json; G4 of the design). Timing (geo_<name>_us) is ADVISORY only.\n\nGuarded ci-bench.sh hook (cargo-only guard; geo needs no javac/rapper/Docker), dashboard\nrow (FEATURED_SUITES + GROUP_ORDER + gen-metric-labels block; metric-labels.json\nregenerated), benchmarks.toml geo-bench entry + CATALOG row, competitors.json\ngeosparql-jena (http-sparql compliance bar) + postgis (loose non-SPARQL lower bound;\nengines/values empty in git).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(bench): align geo_compliance_deficit unit + de-dispatch PostGIS (sq-tf8n) [OPUS-4.8]\n\nResolve two Copilot review threads on PR #184:\n\n1. scripts/ci-bench.sh: emit geo_compliance_deficit with unit `fixtures`\n   (not `count`) to match gen-metric-labels.py's label map (unit: \"fixtures\"),\n   so the dashboard renders the correct unit.\n\n2. bench/competitors.json: change PostGIS `kind` from `http-sparql` to the\n   inert `reference` kind. PostGIS has no SPARQL endpoint; the shared-adapter\n   dispatch loop in gather-competitors.sh only handles\n   report-cli/js-lib/http-sparql/vector-lib, so `reference` is never\n   auto-dispatched — the harness will no longer attempt to HTTP-query it as a\n   SPARQL endpoint. Stays a documented loose lower-bound reference marker;\n   engines/values remain empty.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(bench-geo): resolve 5 Copilot threads on PR #184 [OPUS-4.8]\n\n[OPUS-4.8] GeoSPARQL bench suite (sq-tf8n) review fixes:\n\n1. perf-gate.py: geo_compliance_deficit (floor:0) was SKIPPED by the\n   `if floor <= 0` guard, so a compliance regression never gated. A floor\n   of EXACTLY 0 is the tightest legitimate best-ever for a DEFICIT metric\n   (0 == perfect coverage); now only NEGATIVE floors are skipped, and a\n   zero-floor DETERMINISTIC metric HARD-GATES (any deficit>0 -> exit 2),\n   with PERF_GATE_ALLOW + no-self-loosen preserved. New self-test 8b.\n   (Zero-floor noise/timing metrics are still skipped — band meaningless.)\n\n2. bench/geo/run.sh: thread $BENCH_GEO through to gen.sh so the corpus is\n   generated by the SAME binary used for `bench` (no seed/window/format\n   drift -> spurious count drift) — matches the doc's override claim.\n\n3. gen-metric-labels.py: the geo_<name> dashboard series are ADVISORY\n   QUERY TIMES (from geo_<name>_us), not counts; mode:count contradicted\n   the µs unit. Now mode:query + \", query\" label suffix, mirroring FTS.\n\n4. ci-bench.sh: gate the geo hook to the MAIN tier (GITHUB_REF skip),\n   mirroring the FTS block (#186) — keeps the sparq-geo example build +\n   ~100k-point corpus OFF per-PR CI like the javac/rapper suites.\n\n5. ci-bench.sh: the geo else-branch note now reflects the PR-tier skip\n   (only reached on main/local when cargo or run.sh is missing).\n\nVerify: perf-gate --self-test PASS + deficit=1 perturbation HARD-FAILs;\nbench/geo/run.sh deterministic gate PASS (6 workloads, deficit=0);\ngen-metric-labels --check + dashboard-smoke.js PASS; shellcheck clean\n(no warnings; pre-existing SC2012 info untouched). NON-CANONICAL timing.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T18:25:16Z",
          "tree_id": "a07485270d09b80df3c37213e4491a2e0a2b6e0a",
          "url": "https://github.com/jeswr/sparq/commit/7db50c0838a21f9e70b8809fdc812d4cad2c1c05"
        },
        "date": 1781548325267,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.554,
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
            "value": 3082.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4338.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 821.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13031.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56778.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 150205.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4298.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40582.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9220.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 63739,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 168254.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2649.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40328.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28503.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1667982.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6244.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3705.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3380.3,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7254.1,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 508752.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12556.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 32837.8,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53749.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3767.3,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21901.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 137334.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 100700.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 160022.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36003.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7177.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 14012.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29240.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1676598.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6578.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3769.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3509,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9161.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 506939.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12422.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 32520.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53395.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3981.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22365.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 135391.7,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 97356.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 180956.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36042.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6541.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13021,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 14.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29566.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 19.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1553512.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6375.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3794.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3449.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9373.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 499941.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12262.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 32806.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 52499.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3977.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21364.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 134564.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 100407.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 164222,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.5,
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
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 34035.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6087.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12398.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6228.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15164.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 15193.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14682.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 427778.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15621.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22249.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 291338.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 21969.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22718.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 285074.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8944.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 18032.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 15458.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14815.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 472538.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16834.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22456.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 287667.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20857.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 52.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22404.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 285357,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13913.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 21124.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14968.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14661.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 491729.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 17261.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22609.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 283797.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20825,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 136.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22183.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 281900.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 60.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 106.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 20.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 37.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 16.6,
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
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 873.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 107.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 121.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 29.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1549.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 34.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 130.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 127.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 31.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 28.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 55.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 69,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 78,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 104.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 459.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 172,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 261.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 538,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 50.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 56.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 94.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 80.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 103.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 464.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 171,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 266.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 543.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 47.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 63.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 174.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 80.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 110.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 467.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 188.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 299,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 540.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 50.3,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 576.3,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 70,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.4,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 30.3,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2699.6,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3885.7,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 23.9,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.064,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 365.4,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 321.6,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13028.4,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6723.3,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 785229.5,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 145.4,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 83.5,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 101251,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 94.1,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.8,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.5,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.2,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3833.2,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.513315,
            "unit": "s"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.14,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "ad01dd83fbd384e1fdf9aebbe7aa10b1a38ad705",
          "message": "feat(vectors): stream bulk .npy/dump embedding import (sq-3jc8) (#192)\n\n* feat(vectors): stream bulk .npy/dump embedding import (sq-3jc8)\n\nMake VectorStore::import_npy / import_numeric_dump streaming so peak heap is\nO(dim + index) instead of ~2× the embedding matrix. [OPUS-4.8]\n\nPreviously both import paths read the WHOLE input file into a Vec<u8>\n(std::fs::read), decoded it into a full owned flat Vec<f32>, then handed that\nto write_store (VectorStore::create + put + finalize), which buffered the whole\ndense payload in RAM before finalize — peak ≈ input file + decoded matrix (~2×).\n\nNow: parse only a bounded header prefix (.npy: ≤ MAX_NPY_HEADER_LEN + 12 bytes;\nnever the body), validate dtype/shape/order/dim/row-count + the declared body\nlength against the file, then seek to the data and feed the body row-by-row\nthrough StreamingWriter (which appends each row straight to the on-disk data\nsection). Two small reusable per-row buffers (dim·elem_size raw + dim f32) are\nthe only working memory; the transient 8·rows id→slot index that\nStreamingWriter::finalize sorts is the sole row-count-proportional allocation.\nThe resulting .spqv is byte-identical to the buffered writer's output.\n\nFormat handling unchanged: 2-D C-order little-endian f4/f8 only; f8 narrowed to\nf32; big-endian / fortran_order=True / non-2-D / dtype / dim / row-count\nmismatches all fail closed.\n\nTests: streaming output asserted byte-identical to a buffered\nVectorStore::create+put+finalize build for f32, f64-narrowed, and numeric-dump;\nmulti-chunk (257×13 and 200×8 — exercises the row loop many times); edge shapes\n(single row, d=1, empty matrix). Existing round-trip + fail-closed tests retained\n(duplicate-id now caught at StreamingWriter::finalize, still fail-closed). The\nobsolete whole-body decode_npy_body + its unit test were removed.\n\nRefs sq-3jc8, sq-xsq9.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(sparq-vectors): checked row_bytes + grow-race detection in streaming import (sq-3jc8) [OPUS-4.8]\n\nTwo overflow-safety bugs and one fail-closed regression in the streaming\nembedding importers:\n\n* row_bytes = dim*4 (dump) and dim*elem (npy) were UNCHECKED multiplies feeding\n  a Vec<u8> allocation BEFORE StreamingWriter validates dim. The rows*dim*4 /\n  rows*dim*elem expected-length checks do not bound dim*4 when rows == 0, so an\n  absurd dim could overflow (panic in debug) or force a huge allocation. Both\n  sites now use checked_mul and fail closed with a clear error.\n\n* The streaming dump importer only caught a metadata->read SHRINK race (mid-stream\n  read_exact error); a GROW race (bytes appended after the size check) was\n  silently ignored because we stop after  rows. Restore the pre-streaming\n  fail-closed-on-growth semantics: after the final declared row, confirm the\n  reader is at EOF and reject trailing bytes.\n\nAdd a deterministic absurd-dim overflow rejection test. The happy-path round-trip\ntest already guards the EOF check against false positives on correctly-sized files.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T18:30:57Z",
          "tree_id": "fa8e450802be78ead639ff77b45e28608faff555",
          "url": "https://github.com/jeswr/sparq/commit/ad01dd83fbd384e1fdf9aebbe7aa10b1a38ad705"
        },
        "date": 1781548625866,
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
            "value": 3084.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4350.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 812.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12350.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55414.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 144873.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4418.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 39320.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8725,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57011.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 153054.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2528.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39194.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28363.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1150301.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6218,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3582.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3262.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7140.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 491927.9,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12193.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 30569,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 51971.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3576.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 20977,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 126888.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 91423.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 151155.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.6,
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
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35499.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6547.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12266.1,
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
            "value": 28249,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1136353.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6198.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3643.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3314.3,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8025.3,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 498986.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 11867.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30266.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 51819.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3680,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 20945.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 125515,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 89468.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 151199.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36772.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6359.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12327.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 28058.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1142935.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6267.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3682.6,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3365.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8545.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 493559,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12242.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30280.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53060.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 4079.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 20966.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 121621.3,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 89597.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 150313.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 33631.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6038.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12307.6,
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
            "value": 6197.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15200.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14838.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14664.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 410769.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15270.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22415.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 286557.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20466.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21729.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 279237.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8362.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16040.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14799.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14572.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 450022.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16139.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22138.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 278791.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20475.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 49.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 20627.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 280176.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12383.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 18099.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14801.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14571.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 457601.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16463.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22474.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 277370.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20342.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 136.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 20788.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 284760.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 65,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 113.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18,
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
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 37.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 866.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 25.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 108.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 15.8,
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
            "value": 122.7,
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
            "value": 16.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1537.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 129.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 133.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 33.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 22.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 55.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 68.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 78.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 102.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 471.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 163,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 262.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 550.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 47.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 52.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 82.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 80,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 105.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 472.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 170.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 270.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 546.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 47.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 58.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 174.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 80.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 110,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 465.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 196.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 304.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 544.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47.7,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 596.1,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 61.7,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 27.2,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 28.7,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2661.1,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3858.5,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22.6,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.057,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 357.2,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 321.5,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 12838.7,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6707.8,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 772254.4,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 144.1,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 84,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 101251.7,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 91.5,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.9,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.5,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.3,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3627.7,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.494063,
            "unit": "s"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.134,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "24ae8ceaf210f95d819fd73d615aa871dc6ccb7f",
          "message": "fix(bench): per-shape SHACL expansion + push-before-shutdown EC2 gather harness (sq-8dp3) (#190)\n\n* fix(bench): per-shape SHACL expansion + push-before-shutdown EC2 gather\n\n[OPUS-4.8] sq-8dp3 harness fixes (Fable unavailable; re-review when Fable returns):\n\n1. pySHACL/report-cli/js-lib shapes-dir bug: SHACL_SHAPES is a DIRECTORY\n   (bench/shacl/shapes/, one workload per *.ttl). An external engine handed the\n   directory as a single graph arg loads NO shapes -> conforms:true/0 violations\n   (the apples-to-oranges bug). gather-competitors.sh now expands per shape file\n   and runs the adapter once per workload (keyed by shape stem), matching\n   sparq-shacl's per-workload expected.tsv. Proven locally with pyshacl 0.31.0:\n   directory-arg -> 0 violations; per-shape -> correct nonzero.\n\n2. Results-loss fix (sq-gbq0): the prior gather self-terminated before the\n   orchestrator could SSH-pull result files off a DeleteOnTermination volume.\n   New scripts/gather-ec2.sh pushes every result envelope OUT to the serial\n   console (base64-framed) from INSIDE user-data BEFORE shutdown, so the\n   orphan-proof self-terminate can never race the fetch. Triple watchdog backstop\n   (detached sleep+shutdown, systemd-run --on-active, at now+3h) + ISD=terminate\n   kept as the orphan backstop. Role has no S3/passable-instance-profile;\n   ec2:get-console-output is the proven role-compatible channel.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(bench): harden gather harness — instance-id abort + report-cli jq guard [OPUS-4.8]\n\nResolves Copilot review threads on PR #190:\n- gather-ec2.sh: guard run-instances output — abort cleanly if it does not\n  return a valid i-... id (empty/None on launch failure), instead of polling/\n  terminating a bogus id for ~50 min. INSTANCE_ID reset before exit so the\n  cleanup trap does not act on a bad id.\n- gather-competitors.sh: run_report_cli_engine now guards `have jq` (matching\n  the js-lib adapter), failing with a clear die() message instead of a bare\n  'command not found' under set -euo pipefail.\n\nThe console-channel threads (sleep-5 shutdown truncation, /dev/console-only\npush_console, and the 'quoted/manually expanded via printf' heredoc comment)\nwere filed against an earlier draft; the committed script already uses the\nSSH/scp pull channel (no in-userdata shutdown, sentinel + pull-while-alive),\nwhich structurally resolves all three.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(bench): gather-ec2 watchdog de-dup + SSH-abort guard [OPUS-4.8]\n\nAddress Copilot review on PR #190 (sq-8dp3):\n\n- Watchdog (user-data): the heredoc armed the 3h shutdown twice via a\n  duplicated (sleep …)& + `at …` pair around apt-get. Keep TWO independent,\n  agent-death-independent mechanisms that both arm BEFORE any package install\n  — detached sleep subshell + systemd-run transient timer — and drop the\n  redundant duplicate. `at` was a silent no-op there (not installed until the\n  apt-get below it), so it is removed from the backstop set and from the\n  install list; orphan-proofing is preserved (≥2 independent mechanisms +\n  instance-initiated-shutdown=terminate). Documented why each backstop exists.\n\n- SSH wait: if sshd never becomes reachable the loop exited but the script\n  continued, spinning the poll/pull phase ~45 min against an unreachable host.\n  Track reachability and `exit 1` on failure so the EXIT cleanup trap fires\n  and terminates the instance.\n\nThe report-cli `jq`-availability guard the review also flagged is already\npresent (matches the js-lib path's guard); no change needed there.\n\nshellcheck: no new findings (baseline 4×SC2086 + 1×SC2015 unchanged). bash -n clean.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(gather): harden gather harness scripts — 4 Copilot threads (sq-8dp3)\n\n[OPUS-4.8] PR #190 script-hardening fixes (verified via shellcheck + bash -n + logic;\nno EC2 launched):\n\n- gather-ec2.sh: trim whitespace from checkip MYIP (curl adds trailing newline) so\n  \"${MYIP}/32\" is a valid SG ingress CIDR; fail fast if the IP is empty.\n- gather-competitors.sh shacl_shape_files: fail CLOSED when a shapes dir expands to\n  zero *.ttl files (was a silent no-op producing zero result envelopes).\n- gather-competitors.sh shacl_workload_of: replace glob-matching `case` with a literal\n  `[ x = y ]` so single-file shape paths with glob metacharacters can't mis-match.\n- gather-competitors.sh fallback payload: include the `workload` field in the\n  sidecar-parse-failure fallback so the result stays attributed to its shape/workload.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T19:19:36Z",
          "tree_id": "e82e89a24d6078a94cad8f03a905045654cb8b11",
          "url": "https://github.com/jeswr/sparq/commit/24ae8ceaf210f95d819fd73d615aa871dc6ccb7f"
        },
        "date": 1781551481647,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.559,
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
            "value": 3090.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4361.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 822.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12993.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 60169.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 161128.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3443.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42807.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9293.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 66883.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 166257.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3440.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 46338.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29559.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 2678879.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6897,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3866.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3503.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7404.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 500331.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 13561.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 34468.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 56756.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3993.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22821.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 160743.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 117651.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 215204.4,
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
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 38191.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6724.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13465.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 34616.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 3055022.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6577.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3903.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3690.3,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9422,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 506716.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12205.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 33668.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 56358.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 4083.3,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 24026.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 142746.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 100289.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 176574.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 38322.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6771.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13370.5,
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
            "value": 29576.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 19.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 2467818.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 7886.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3748.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3350.3,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9362,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 503636.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12077.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 34609.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 55919.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3626.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21499.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 147832.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 100311.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 164945.6,
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
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 34240.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6997.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13563.3,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6381.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15735.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 15692.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 15759.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 439579.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15394.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22473.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 288987.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 21784.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 23240.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 294301.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 10091.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19768.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16494.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16770.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 495656.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16973.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22695.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 293020.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 22297.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 62.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22968.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 290637.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12829.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 21862,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16099.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 15662,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 488049.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 18332.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 23176.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 288290.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 21411.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 134.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22795,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 281304.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 67.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 33.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 30.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 106.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 37.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 15.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 884,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 109.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 23.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16,
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
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 124,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 30.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 23,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1563.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 29.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 132.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.9,
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
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 136.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 56.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 78.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 78.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 103.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 466.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 164.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 265.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 539,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 49.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 57.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 83.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 77.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 105.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 465.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 171.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 273.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 547.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 46.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 60.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 177.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 81.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 109.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 458.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 190.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 302.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 547.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 50.8,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 589.4,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 61.3,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.6,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 37.8,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2715.3,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3889.3,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 18.6,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 24.9,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.008,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.074,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 348.7,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 324.3,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13254.3,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 7090.7,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 779260,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 151.1,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 84.6,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 101882.4,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 94.2,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.6,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3683.5,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.478461,
            "unit": "s"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.139,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "752e402ddc82b528cc13d8f393b61c53b215199b",
          "message": "feat(sparq-mpc): batched hidden-value join over row columns + oblivious output (sq-khf9) (#194)\n\n* feat(sparq-mpc): batched hidden-value join over row columns + oblivious output (sq-khf9)\n\n[OPUS-4.8] Wire the sq-dwb5 BatchedShares/RowBinding primitive and the sq-jnkm\noblivious output transform into HiddenValueJoin so the hidden-value join ranges\nover a COLUMN of rows per holder under a documented row-binding, with the output\ncardinality/ordering hidden.\n\n- New `BatchedHiddenInput` carries a holder's per-row (private_key_fp, disclosed\n  payload) column PLUS its RowBinding; `shared_keys` deals the private key column\n  as a `BatchedShares` (the sq-dwb5 primitive, actually used — fresh degree-t\n  poly per row, <=t parties' views independent of every key).\n- `HiddenValueJoin::batched_join` lifts the existing secure-equal to the row\n  dimension: RowBinding decides candidate pairs (Positional = index-aligned\n  equal-length columns; Keyed = disclosed-key bucketed), the match is decided by\n  secure_equal (join key/value NEVER opened), and the candidates feed\n  oblivious_join_output with MatchBit::Public + a public bound B, so the OUTPUT\n  reveals neither the true match count (bounded to B) nor which input pair\n  produced which row (shuffle).\n- Both regimes (Positional + Keyed) landed. HONESTY: the per-candidate match BIT\n  is still opened by secure_equal (decision-time L2), unchanged from the scalar\n  HiddenValueJoin; the fully-oblivious secret-bit producer is gated on\n  sq-rrz4/sq-dvuc (the gate oblivious_join names) — followed up as a new bead.\n  Semi-honest only, unchanged from the ShamirBackend layer.\n\nTests (13 new): positional + keyed differential vs plaintext across n in {3,5,7};\nkeyed fan-out within a key bucket; output cardinality bounded to B (privacy);\nkey value never in output; output order oblivious across seeds; forged-match\nsoundness across n; fail-closed on mixed bindings / unequal columns / B < cands /\nkeyed key-count mismatch; shared_keys exercises the batched primitive.\n\nRefs sq-khf9, sq-dwb5.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(mpc): genuinely wire BatchedShares into batched_join secure-equal [OPUS-4.8]\n\nResolve 4 Copilot threads on PR #194 (sq-khf9, batched HiddenValueJoin):\n\n1. (REAL) batched_join dealt BatchedShares via shared_keys() then discarded\n   them and re-shared each scalar inside secure_equal — defeating the wired\n   primitive. Add secure_equal_shared() that consumes already-dealt ShareVecs;\n   batched_join now compares the per-row sharings from the up-front\n   BatchedShares (keys dealt ONCE, not re-shared per candidate). secure_equal\n   delegates to the shared core, so the scalar path is unchanged. Join results\n   + privacy property verified unchanged.\n2. (doc) Correct the BatchedHiddenInput struct doc: cleartext keys are used as\n   inputs to Shamir sharing AND the equality test, and are NEVER reconstructed\n   from shares (was \"consumed only to deal the batch\").\n3. (test) Strengthen output_cardinality_is_bounded_to_b_not_true_count: go\n   through oblivious_set_output to assert the revealed SLOT VECTOR length is\n   exactly B for 1 vs 3 true matches (substantive obliviousness), and that the\n   hidden join key never appears in any revealed slot — not just the bound echo.\n4. (comment) forged_match_attempt_fails: comment now matches the (42,42)\n   control / (42,43) forge data (was \"right has only 7 and 8\").\n\ncargo build + nextest (265 passed) + clippy -D warnings all green.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T19:26:07Z",
          "tree_id": "5a83c0eaeb4c21cb293eca160bccb359f6699a1c",
          "url": "https://github.com/jeswr/sparq/commit/752e402ddc82b528cc13d8f393b61c53b215199b"
        },
        "date": 1781551863522,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.549,
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
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3307.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4884.5,
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
            "value": 790.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13189.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59798.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 163666.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4120.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41841,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8255.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 59156.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 164792,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4023.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40452.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29593,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1884600.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6341.8,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3780.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3555.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7306.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 475047.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12698.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 30887.5,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53506,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3735.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22600.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 135167.2,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 104979.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 176972.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35314.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7411.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13282.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29677.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1870279.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6316.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3820.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3499.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8813,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 478372.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12867.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31474.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53639.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3740.5,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22595.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 14.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 145888,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 105766.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 175949.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36665.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6935.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13008,
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
            "value": 29545,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 20.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1651533.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6254.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3752,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3521.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8604.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 476248.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12903.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31280,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53849.9,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3747.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22479.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 13.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 141791.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 103063.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 176740.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 35410.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7223.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13598.3,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6689,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16434.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16302.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16304.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 459212.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17651.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24535,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 283781.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22738,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 23057.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 284164,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9537.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19213.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16320,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16321.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 494572.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18380.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24409.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 291378.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 23829.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 64.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22999.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 290869.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 14742.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 22736.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16957.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16743.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 506341.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 19444.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 25125.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 288838.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23533.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 133.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 24118.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 291805.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 61.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 27.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 98,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 31.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 941.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 30.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 101.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.2,
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
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 113.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 32,
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
            "value": 23,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1551.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 27.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 125.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 133.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 33.8,
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
            "value": 12,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 55.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 76.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 71.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 99.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 502.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 174.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 283.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 599.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 44.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 57,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 94.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 76.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 102.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 500.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 191.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 301.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 618,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 44.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 69.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 156.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 85,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 107.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 502.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 197.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 321.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 599.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 45.1,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 601.9,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 70.4,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.7,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 30.6,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2952.4,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4066.1,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 23.5,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.6,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.067,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 350.6,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 314.6,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13310.5,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6557.4,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 685714.8,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 148.3,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 80.8,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 107292.6,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 95.9,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.3,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.3,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.6,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.3,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3888.5,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.534897,
            "unit": "s"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.141,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "61f64e256dedcde004c0486d99a016f776990242",
          "message": "feat(zk-compose): canonicalise manifest edge ordering (sq-y2wy) (#196)\n\n* feat(zk-compose): canonicalise manifest edge ordering (sq-y2wy)\n\n[OPUS-4.8] Implement the deferred canonical ordering for both edge vectors on\n`ProofManifest` (PR #178 / sq-fi03 documented it then corrected the doc to\n\"deferred\"; this lands it).\n\n- Sort `binding_edges` ascending by `(from_proof, from_row, from_slot,\n  to_proof)` and `join_edges` ascending by `(scan_a, graph_a, scan_b, graph_b,\n  join_proof)` — each the struct's field-declaration tuple, derived via\n  `Ord`/`PartialOrd`, a TOTAL order over edges (the `join_proof` tail extends\n  the originally-proposed 4-tuple to a strict total order even when two join\n  edges share all four scan refs).\n- New `ProofManifest::canonicalize` applies it in place; `to_json` canonicalises\n  a CLONE before serialising, so the on-the-wire/hashable form is deterministic\n  in edge order without mutating `self`. Two manifests differing only in edge\n  order now serialise byte-identically (hence equal hash).\n- Edge-vector sorting preserves scan-reference validity: each edge is\n  self-contained (carries its own `sub_proofs` indices), and the verifier\n  (`binding_edges` stage 2 / `bind_joins` stage 2g) resolves edges by those\n  indices, never by vector position — so reordering cannot invalidate a\n  reference. Updated the `BindingEdge`/`JoinEdge` docs to describe the now-real\n  canonicalisation (replacing the deferred-ordering note).\n\nTests (manifest::canonical_edge_tests + join_gates):\n- determinism: binding_edges + join_edges built in order X vs Y canonicalise to\n  the same total order (tuple tie-breakers exercised incl. `join_proof`),\n  idempotent.\n- to_json byte-identical regardless of edge insertion order; does not mutate\n  self; canonical JSON round-trips to a sorted manifest.\n- multi-edge (>=2 of each) stable order regardless of insertion order.\n- build -> canonicalise -> verify: a canonicalised honest join manifest still\n  passes the structural bind_joins gate.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* perf(zk-compose): use sort_unstable in canonicalize\n\nEdges are distinct under derived total Ord, so a stable sort is\nunnecessary; sort_unstable is deterministic here and avoids the\nauxiliary allocation a stable sort may perform.\n\n[OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T20:22:05Z",
          "tree_id": "183d9e2c1a7e8886c6553a287d7bc4f43ba49134",
          "url": "https://github.com/jeswr/sparq/commit/61f64e256dedcde004c0486d99a016f776990242"
        },
        "date": 1781555207966,
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
            "value": 3074,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4340.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 820.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12967,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55633,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 147988.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2762,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40111.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 11626.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 69209.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 157322.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2426.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39509.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29308.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1482101.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6225.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3689.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3397.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7167.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 510743.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12650.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31375.6,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52286.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3634.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21600.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 136760.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 93400.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 157164.3,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 33673.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6260.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12650.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28660.1,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1349081,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6127.8,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3685.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3355.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9152.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 505809,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12425.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 32760.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53057.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3582.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21261.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 128953.3,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 89024.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 153605.4,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.7,
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
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 35064.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6889.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12812.6,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29021.1,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1295821.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6383.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3656.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3393.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8711.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 506112.9,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12840.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31482.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53101.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3933.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21475.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 129262.7,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 89721.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 155244.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.1,
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
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 34032.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6337.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12420.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6195.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15215.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14938.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14826.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 407996.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15193.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 21829,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 285138,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20568.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 20789,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 282354.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8424.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16836.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14801,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14543.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 459383.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16400.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22208.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 280327.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20481.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 58.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 20726.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 280557.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12603.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 18006,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14874.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14553.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 464283.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16786.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 21956,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 280192.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20473,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 135.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 21003,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 280637.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 65.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 101.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 38.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 14.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 866.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 28.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 110.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.1,
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
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 128.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 30.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 23.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1503.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 31.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 129,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.4,
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
            "value": 126.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 31.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 22.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 28.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 55.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 77.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 77.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 102.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 464.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 160.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 296.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 538.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 47.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 59,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 83.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 78.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 105,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 467.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 173.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 272.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 548.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 47.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 58.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 172.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 84.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 110.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 474.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 185.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 307,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 546.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47.3,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 589.4,
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
            "value": 62.4,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 27.3,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.7,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2701.8,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3845.5,
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
            "value": 22.6,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.062,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 349.3,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 325.6,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 12956.2,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6689.1,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 770049.3,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 143.4,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 83.4,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 102277,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 94.5,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.9,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.3,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.3,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3685.1,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.452312,
            "unit": "s"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.138,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "b2404eafe9b36003868136f9204782c0a843c50b",
          "message": "fix(bench): port family_curve to composable CircuitId::FilterF64 { d } (sq-kep2) (#197)\n\n* fix(bench): port family_curve to composable CircuitId::FilterF64 { d } (sq-kep2)\n\nThe standalone `bench/zk-compose/family_curve` harness is a detached cargo\nproject (own `[workspace]`), so it is NOT covered by the root workspace CI gate\nand broke silently when sq-q7e/sq-tat made the f64 filter manifest-composable:\n`CircuitId::FilterF64` went unit→struct (`{ d: u32 }`), `prover_toml_for` grew a\nsixth `join_witness` arg and now returns `Result`, and `ProofInputs::FilterF64`\ngained a real composable shape. [OPUS-4.8]\n\nFix:\n- `bench_filter_f64` now mirrors `bench_filter_int`: it uses the composable\n  `build_filter_f64(operand_enc, value, FilterOp::Lt, bound, true)` path,\n  reads `d` from `CircuitId::FilterF64 { d }`, and renders the Prover.toml via\n  `prover_toml_for` from the canonical decimal-digit witness — replacing the old\n  hand-written raw `a_bits`/`b_bits` Prover.toml + unit `CircuitId::FilterF64`.\n  `d` is the operand's decimal digit count (FILTER_F64_D_VALUES = {1,2,3,4}),\n  the same source `build_filter_int` derives `d` from. The member is now swept\n  over d ∈ {1,2,4} like filter_int.\n- Updated the two existing `prover_toml_for` call sites (scan, filter_int) for\n  the new 6-arg / `Result` signature (`None` join witness + `.expect`).\n- Cargo.lock refreshed to resolve against current crate sources.\n\nRecurrence prevention: rather than fold the harness into the root workspace\n(which would defeat its deliberate isolation — own `[workspace]`, `lto = \"fat\"`\nrelease profile, kept off the fast build/clippy/wasm gate), add a near-free\n`cargo build` of family_curve to `.github/workflows/zk-toolchain.yml`, the lane\nthat already triggers on `crates/sparq-zk-compose/**`. So a future\nCircuitId/ProofInputs/prover_toml_for schema change surfaces as a red check on\nthe PR that introduces it. README documents the standalone rationale + the guard.\n\nBuilds + runs clean against current main (all 9 members prove AND verify);\nclippy clean. Refs sq-kep2.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(bench/family_curve): guard f64 bound overflow, --locked CI build, reconcile README\n\n- main.rs: widen `value + 1` to u128 before the f64 cast so it cannot\n  wrap silently in release at u64::MAX; debug_assert the 2^53 exactness bound.\n- zk-toolchain.yml: build the detached family_curve harness with --locked so\n  CI fails on Cargo.lock drift instead of silently rewriting it.\n- README: reconcile the stale \"NOT gated in CI\" claim with the build guard —\n  CI builds (compile-only drift guard) but does not run the harness.\n\n[OPUS-4.8]\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T20:22:42Z",
          "tree_id": "e3ba9aac159b50fcde6454b881172ed7896bfbfd",
          "url": "https://github.com/jeswr/sparq/commit/b2404eafe9b36003868136f9204782c0a843c50b"
        },
        "date": 1781555499196,
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
            "value": 3.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3084.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4341,
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
            "value": 814.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12474.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55615.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 148867.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3872.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 39294.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9025.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 58327.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 158210.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2573.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 47113.4,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28975.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.9,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1208139.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6232.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3648.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3402.3,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7162.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 509566.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 11753.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31325.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52712,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3574.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21181.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 125568.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 89488.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 149814.5,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 34765.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6518.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12871.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28426.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.4,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1181881.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6197.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3708.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3356.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8296.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 512084.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12298.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 37923.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 51976.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3657.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21074.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 124717.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 93157.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 158431.9,
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
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 33422.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6953.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12917,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 28600.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1170795.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6131.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3651.7,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3374.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8449.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 507681.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12151.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30912.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 52704.9,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3602.5,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 20972.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 123336.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 89205.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 152351.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 33343.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6549.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12482.8,
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
            "value": 6279.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15098.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14823.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14838.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 406595,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15206.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22251.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 283945.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20830.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21434.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 283152.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8695.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 17306.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14891.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14739.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 461339.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16172.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22035.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 278000.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20469,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 62.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 20824.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 282167.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12575.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 18266.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14769.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14544.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 475784.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16676.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22158.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 285722.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20946.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 135.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22193.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 285458,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 59.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 31.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 107.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 37.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.5,
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
            "value": 876.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 112.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.6,
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
            "value": 123.1,
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
            "value": 22.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1538.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 131.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 124.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 22.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 56.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 73.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 79.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 101.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 467.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 163.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 261.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 547.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 46.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 56.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 80.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 87.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 104.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 462.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 170.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 272.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 550.8,
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
            "value": 58.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 177.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 79.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 109.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 463.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 196.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 316.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 565,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47.1,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 587.4,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 76.2,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.7,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 28.9,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2644.9,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3850.4,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22.3,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.058,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 356.2,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 334.6,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13054.8,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6685.3,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 775881.3,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 153.2,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 83.5,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 103143.5,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 95.4,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.6,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3701.9,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.505946,
            "unit": "s"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.134,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "fd219babd41d36e36b8e72e161c5c4c8277583fe",
          "message": "feat(bench): vector/ANN benchmark suite — recall@10-deficit gate (sq-v02y) (#195)\n\n* feat(bench): vector/ANN benchmark suite — recall@10-deficit gate (sq-v02y) [OPUS-4.8]\n\nThe 4th capability-surface bench suite (after SHACL/FTS), mirroring the LUBM/FTS\ntemplate: overview dashboard row + self-asserting deterministic gate + competitor\ncomparison. Design: research/capability-benchmark-program.md §3.3.\n\n- Promotes the crate recall asserts (tests/{recall,diskann,quant}.rs) into TRACKED\n  METRICS, emitted AS DEFICITS recall_deficit_milli = round((1-recall@10)*1000) vs the\n  nearest_exact ground truth, so they slot into the smaller-is-better mode:auto ratchet\n  with ZERO perf-gate.py change (gap G4). diskann/pq are EXACT-gated + mode:auto\n  ratcheted (single-threaded fixed-seed builds => byte-deterministic; floors DERIVED BY\n  RUNNING: diskann 34, pq 22); hnsw is FLOOR-gated only (instant-distance build is\n  rayon-parallel => +/-1 deficit jitter, so exact equality would flake — honest, not\n  dishonest-flaky).\n- bench/vector/{gen.sh,run.sh,expected.tsv,README.md} + the G1 runner\n  crates/sparq-vectors/examples/bench_vectors.rs (same recall measurement as the gate\n  tests). run.sh self-asserts (exit 1 on drift) — proven: PASS exit 0, perturb diskann\n  34->0 => exit 1.\n- Guarded ci-bench.sh hook with the FTS GITHUB_REF PR-tier skip (vector build + corpus\n  off per-PR CI; runs on main / locally).\n- Dashboard: Vector / ANN row in FEATURED_SUITES + GROUP_ORDER (4th suite); gen-metric-labels.py\n  block + regenerated metric-labels.json (--check passes); dashboard-smoke.js assertions.\n- benchmarks.toml (vector-ann-bench) + CATALOG.md + competitors.json (ann-benchmarks\n  python-lib: hnswlib/FAISS/ScaNN/DiskANN-ref via vector_lib_adapter.py + exact-kNN\n  oracle; recall-QPS Pareto at MATCHED recall, never a single latency; Qdrant/Milvus/\n  Weaviate loose-only; engines/values EMPTY in git). No competitor does ANN-inside-SPARQL\n  over dict-encoded ids (uncontested surface). Big SIFT1M/GloVe recall-QPS is gather-tier.\n\nVerify: nextest -p sparq-vectors 93 passed; clippy --workspace --exclude sparq-py clean;\nperf-gate --self-test passes; gen-metric-labels --check + dashboard-smoke.js pass. No\nhard-coded perf in markdown; deficit baselines derived by running.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(bench/vector): correct recall-floor boundary, workload-set check, milli unit, build count [OPUS-4.8]\n\nResolves the 4 Copilot review threads on PR #195 (sq-v02y, the Vector/ANN bench suite):\n\n1. bench/vector/run.sh floor gate: the inclusive crate condition is recall@10 >= floor,\n   and deficit = round((1-recall)*1000), so a deficit EQUAL to the floor is EXACTLY the\n   floor recall and must PASS. Changed `-ge floor` (failed at the floor, one milli-deficit\n   stricter than documented) to `-gt floor`; fixed the boundary docs (recall>=0.95 <=>\n   deficit<=50, not <50).\n\n2. bench/vector/run.sh final check: replaced the line-COUNT check (fooled by a\n   vanished+duplicated pair) with a DISTINCT-NAME-SET comparison against expected.tsv,\n   so a missing workload is caught even if another is duplicated.\n\n3. scripts/ci-bench.sh: the vectors_*_recall_at10 value is a MILLI-scaled deficit, so the\n   emitted unit is now `milli` (matching metric-labels.json), not the ambiguous `deficit`.\n\n4. crates/sparq-vectors/examples/bench_vectors.rs: VectorIndex::build ran iters+1 times\n   (one warm-up + iters in the loop). Restructured so build runs EXACTLY `iters` times\n   for the build_s metric; the last-built index is retained for the recall measurement.\n\nVerified: run.sh gate passes; boundary proven (deficit==floor PASS, deficit>floor FAIL);\ndistinct-set check catches a simulated missing+duplicated workload; bench_vectors builds +\nruns with finite output; cargo nextest -p sparq-vectors (97 pass) + clippy clean;\ngen-metric-labels.py --check + dashboard-smoke.js + shellcheck clean. NON-CANONICAL timing.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(bench-vectors): resolve 9 Copilot threads on the vector/ANN suite (sq-v02y) [OPUS-4.8]\n\ncompetitors.json (thread 1): the shared-adapter dispatch only routes\nkind ∈ {report-cli,js-lib,http-sparql,vector-lib}. Make the vector entries'\nkind match intent: ann-benchmarks (hnswlib/FAISS via vector_lib_adapter.py)\n-> `vector-lib` so it auto-gathers; vector-dbs (Qdrant/Milvus loose-only,\n\"do NOT gather as SPARQL\") -> the INERT `reference` kind (was wrongly\n`http-sparql`, which would HTTP-query them as SPARQL endpoints).\n\nrun.sh thread 2 (~41): guard the gen.sh params under `set -u` — check it\nproduced >=2 non-empty lines (N + seed) and fail with a clear message\ninstead of an opaque unbound-variable crash.\n\nrun.sh thread 3 (~108): `grep -vE` exits 1 when it filters every line\n(empty seen_names), which under `set -euo pipefail` aborted before the\nmissing-workload report. `|| true` keeps an empty set empty (still caught\nby the set-mismatch check). [OPUS-4.8]\n\nREADME / expected.tsv / run.sh threads 4-6: make the HNSW floor semantics\nagree on ONE boundary. run.sh is INCLUSIVE (fails only on `deficit > floor`),\nso state it as `deficit <= 50 ⇔ recall@10 >= 0.95` everywhere (was strict\n`<` in the docs).\n\nbench_vectors.rs threads 7-9 (~219/268/279): drop() the file-backed handles\n(disk / pq_store+enc+pq / index+store) BEFORE remove_file/remove_dir so\ncleanup succeeds under mandatory file locks (Windows). No-op on Linux.\n\nVerify: run.sh passes on the pinned N=50000 corpus; gen.sh short/empty +\nempty-seen_names -> graceful exit 1 (no crash); floor boundary 50 passes,\n51 fails. bench_vectors builds + runs (finite output, temp files cleaned).\ncargo nextest -p sparq-vectors (97 pass) + clippy --workspace --exclude\nsparq-py -D warnings clean. shellcheck + jq + dashboard-smoke pass.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T20:36:24Z",
          "tree_id": "999dae6f25c0d08b40e1179899deff25f813cbc2",
          "url": "https://github.com/jeswr/sparq/commit/fd219babd41d36e36b8e72e161c5c4c8277583fe"
        },
        "date": 1781556283889,
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
            "value": 3.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3077.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4345.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 820.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12484.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 54529.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 141055.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2396.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 38893.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8818.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 56832.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 154039.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2704.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 37913.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 27981.8,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1145557.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6100.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3623,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3412.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7096,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 505675.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 11969.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 29783.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52429.9,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3620.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21033.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 123062.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 88967.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 150950.3,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.2,
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
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 33892.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6543,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12457.3,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28307.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1146871.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6258,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3633.8,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3300.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8016.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 498437.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 11879.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30494.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 51742.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3753.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 20935.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 119734.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 90281.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 150772.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 34768.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6603.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12421.4,
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
            "value": 28484.8,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1136408.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6105.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3741.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3371.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8775.9,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 502139.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 11729.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30838.5,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 52271.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3567.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21098.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 22.3,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 118928.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 89547.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 151484,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 35919.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6328.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12569.4,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6199,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15248.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14824.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14721.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 410692.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15075.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 21961.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 284129.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20569.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21155.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 280658.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8362.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16049.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14634,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14586.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 446819.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16071.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22207.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 282716.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20417.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 58,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 20685.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 277510.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12261,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 17982.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14892.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14694.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 459869.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16267.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22030.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 277711.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20667,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 135.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 20860,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 279690.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 60.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 102,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 19.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 38.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 867.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 108.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 18.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 13.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 13.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 123.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 30.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.6,
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
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1523.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 131.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 130.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 28.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 54.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 69,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 108.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 102.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 476,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 163.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 266.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 554.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 46.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 54.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 82.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 80.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 106.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 473.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 181.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 268,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 541.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 47.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 57.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 169.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 79.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 109.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 475.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 189.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 301,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 546.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 49.1,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 587.7,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.4,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 61.5,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.3,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 28.7,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2626.6,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3843.9,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 16.7,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22.6,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.058,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 347.9,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 328.8,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 12979.3,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6755.7,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 760170.2,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 154,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 87.1,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 100260.6,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 96.2,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.8,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.5,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.4,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3615.6,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.427311,
            "unit": "s"
          },
          {
            "name": "vectors_diskann_recall_at10",
            "value": 34,
            "unit": "milli"
          },
          {
            "name": "vectors_diskann_query_us",
            "value": 330.2,
            "unit": "us"
          },
          {
            "name": "vectors_hnsw_recall_at10",
            "value": 0,
            "unit": "milli"
          },
          {
            "name": "vectors_hnsw_query_us",
            "value": 390.5,
            "unit": "us"
          },
          {
            "name": "vectors_pq_recall_at10",
            "value": 22,
            "unit": "milli"
          },
          {
            "name": "vectors_pq_query_us",
            "value": 408.1,
            "unit": "us"
          },
          {
            "name": "vectors_build_s",
            "value": 41.935511,
            "unit": "s"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.133,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1604690,
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
          "id": "9d659e5dabc2b713c42d96a6b7e9051d4e2e936c",
          "message": "ci(release-plz,bench): stop two recurring non-gating main reds (sq-owci) [OPUS-4.8] (#962)\n\nrelease-plz / Release-PR failed on every push:main: `release-plz release-pr`\nreuses cargo's VCS dirty-check, which treats a file that is BOTH committed AND\nmatched by a .gitignore rule as \"uncommitted\". On a fresh checkout the four\ntracked-yet-gitignored artifacts (.beads/interactions.jsonl,\nbench/wikidata-8b/STATUS.md, inference-conformance-report.md, zk/compose/STATUS.md)\nare the only thing flagged, so the step aborted \"failed to determine next versions\"\nevery time (live: run 27867775455). Set allow_dirty = true in release-plz.toml so\nrelease-pr proceeds — there is no diff content in those committed-but-ignored paths,\nso versions are still computed from commit history exactly as before; no release\nregression is masked. The `tag` job was always green (release does no dirty-check).\n\nBenchmarks / 'run + track benchmarks' failed on every push:main: ci-bench.sh runs\n`cd bench/zk && cargo bench` and `cd bench/zk-trace && cargo bench` (standalone\ncargo projects, no --locked), rewriting their committed Cargo.lock and dirtying the\ntree; github-action-benchmark's auto-push (main only) then `git switch benchmark-data`\naborted \"local changes would be overwritten\" (live: run 27867742598). Add a\nmain-only step that `git checkout -- .` restores the incidental tracked churn before\nthe switch — untracked outputs (bench-results.json, prev-data.js) are left intact and\nthe measurement is unchanged.\n\nNeither workflow touches the ci-summary/gate aggregator or any gating lane:\nrelease-plz only triggers on push:main (never PR/merge_group, so never on the gate);\nthe new bench step is if: refs/heads/main (skips → non-failing on PR/merge_group) and\nis a step inside an existing job, so it adds no new check-run name. Root-cause repo\nhygiene (untrack the 3 generated artifacts; drop inference-conformance-report.md from\n.gitignore per AGENTS.md) is filed as a follow-up bead — out of this lane's scope.\n\nCo-authored-by: Jesse Wright <jeswrsolidserver@gmail.com>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-20T10:38:05Z",
          "tree_id": "98641b09a319186cd5591ac5e90ff319a6653de1",
          "url": "https://github.com/jeswr/sparq/commit/9d659e5dabc2b713c42d96a6b7e9051d4e2e936c"
        },
        "date": 1781953735606,
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
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3080,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4333.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 821.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12854.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 57480.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 157310.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3808.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40475.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8800.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57837.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 154194.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2648.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38720.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29119.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1507397.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6310,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3714.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3404.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7421.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 508780.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12210.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31228.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52437.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3654.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21393.8,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 127146.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 94409.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 160008.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.7,
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
            "value": 34308.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6117.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12584.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28776.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1423899.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6480.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3747.7,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3371.3,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8838.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 508780.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12568.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31073.5,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 52600.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3677.4,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21514.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 125522.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 92364.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 157233.4,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 33520.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6310.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12831.6,
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
            "value": 28366.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.4,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1608912.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6218.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3849,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3467.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9560.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 505476.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13045.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31759.5,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53263.8,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3715.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22064.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 137078.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 105734.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 184089,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 36254.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6575.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13012.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6967.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15987.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 15683.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16107.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 457344.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 16283.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 25696.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 296452.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22547.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 25313.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 295633.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 12711,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19631.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 15712.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 15855.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 505349.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18691.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24552.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 301245.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 23096.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 54.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 25143.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 299632.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 16897.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 21559.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16757.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16673,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 505842,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 19378.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 23485.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 296753,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 21502.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 135.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 24002.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 299228.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_count_us",
            "value": 61.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_count_us",
            "value": 33.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_count_us",
            "value": 28.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_count_us",
            "value": 113,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_count_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_count_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_count_us",
            "value": 38,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_count_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_count_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_materialize_us",
            "value": 850,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_materialize_us",
            "value": 27.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_materialize_us",
            "value": 27.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_materialize_us",
            "value": 122.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_materialize_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_materialize_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_materialize_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_materialize_us",
            "value": 121.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_materialize_us",
            "value": 30.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_materialize_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_materialize_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_materialize_us",
            "value": 22.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_materialize_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_json_us",
            "value": 1518.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_json_us",
            "value": 29.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_json_us",
            "value": 29.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_json_us",
            "value": 126.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_json_us",
            "value": 20.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_json_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_json_us",
            "value": 21.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_json_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_json_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_json_us",
            "value": 128.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_json_us",
            "value": 32.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_json_us",
            "value": 23.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_json_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_json_us",
            "value": 29.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_json_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_json_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 56.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 72.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 80.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 109.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 465.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 170.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 267.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 541.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 48.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 58.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 85.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 82.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 105.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 464.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 169.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 269.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 560.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 49,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 60.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 173.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 87.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 114.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 465.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 189.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 305.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 554.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 48.5,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 575.7,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 67.2,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 27.8,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2760.7,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3918.8,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 24.9,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.07,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "sameas_size8_closure_s",
            "value": 0,
            "unit": "s"
          },
          {
            "name": "sameas_size8_query_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "sameas_size8_closure_triples",
            "value": 352,
            "unit": "triples"
          },
          {
            "name": "sameas_size32_closure_s",
            "value": 0.001,
            "unit": "s"
          },
          {
            "name": "sameas_size32_query_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "sameas_size32_closure_triples",
            "value": 4480,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 357.1,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 321.4,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13817.9,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6771.7,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 769537.9,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 148.2,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 87.4,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 104936.8,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 93.1,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.5,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 23.9,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.6,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 4010.5,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.609842,
            "unit": "s"
          },
          {
            "name": "vectors_diskann_recall_at10",
            "value": 34,
            "unit": "milli"
          },
          {
            "name": "vectors_diskann_query_us",
            "value": 390,
            "unit": "us"
          },
          {
            "name": "vectors_hnsw_recall_at10",
            "value": 1,
            "unit": "milli"
          },
          {
            "name": "vectors_hnsw_query_us",
            "value": 455.9,
            "unit": "us"
          },
          {
            "name": "vectors_pq_recall_at10",
            "value": 22,
            "unit": "milli"
          },
          {
            "name": "vectors_pq_query_us",
            "value": 403.6,
            "unit": "us"
          },
          {
            "name": "vectors_build_s",
            "value": 42.604239,
            "unit": "s"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w0_rows",
            "value": 3,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_persistentdict_triples_per_s",
            "value": 2651671,
            "unit": "triples_per_s"
          },
          {
            "name": "snikmeta_triples",
            "value": 328,
            "unit": "count"
          },
          {
            "name": "snikmeta_terms",
            "value": 205,
            "unit": "count"
          },
          {
            "name": "snikmeta_distinct_predicates",
            "value": 23,
            "unit": "count"
          },
          {
            "name": "snikmeta_rdf_type_triples",
            "value": 49,
            "unit": "count"
          },
          {
            "name": "snikmeta_direct_eq_upstream",
            "value": 1,
            "unit": "count"
          },
          {
            "name": "hdt_load_s",
            "value": 0.044405,
            "unit": "s"
          },
          {
            "name": "hdt_vs_ntgz_load_s",
            "value": 3.3596,
            "unit": "ratio"
          },
          {
            "name": "zk_compose_filter_decimal_i3_f2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_gates",
            "value": 3113,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_hidden_issuer_d4_gates",
            "value": 16932,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_pok_gates",
            "value": 10334,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_set_d4_gates",
            "value": 10650,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb16_gates",
            "value": 7025,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb64_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb16_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb64_gates",
            "value": 18681,
            "unit": "gates"
          },
          {
            "name": "zk_compose_revoke_unset_d10_gates",
            "value": 899,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r4_gates",
            "value": 5991,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r8_gates",
            "value": 7038,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r4_gates",
            "value": 14923,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r8_gates",
            "value": 18850,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r4_gates",
            "value": 9254,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r8_gates",
            "value": 11261,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r4_gates",
            "value": 27054,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r8_gates",
            "value": 34821,
            "unit": "gates"
          },
          {
            "name": "zk_canon_bnode_1024_us",
            "value": 5780.635,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_256_us",
            "value": 1404.754,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_64_us",
            "value": 339.605,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_1024_us",
            "value": 4223.792,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_256_us",
            "value": 1001.497,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_64_us",
            "value": 236.418,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_1024_us",
            "value": 95105.528,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_256_us",
            "value": 24204.695,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_64_us",
            "value": 5933.31,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_1024_us",
            "value": 72145.141,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_256_us",
            "value": 18096.826,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_64_us",
            "value": 4502.472,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_1024_us",
            "value": 67756.818,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_256_us",
            "value": 16935.566,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_64_us",
            "value": 4221.954,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_hash40_us",
            "value": 184.331,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_permutation_us",
            "value": 13.122,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_traced_1000_us",
            "value": 4917.509,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_untraced_1000_us",
            "value": 1102.754,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_traced_1000_us",
            "value": 1714.373,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_untraced_1000_us",
            "value": 417.9,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_traced_1000_us",
            "value": 8058.14,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_untraced_1000_us",
            "value": 2415.586,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_traced_1000_us",
            "value": 574.702,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_untraced_1000_us",
            "value": 111.464,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_traced_1000_us",
            "value": 482.757,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_untraced_1000_us",
            "value": 74.529,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_traced_1000_us",
            "value": 1388.201,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_untraced_1000_us",
            "value": 269.826,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_traced_1000_us",
            "value": 4187.852,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_untraced_1000_us",
            "value": 1033.793,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_traced_1000_us",
            "value": 1327.275,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_untraced_1000_us",
            "value": 428.038,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_traced_100_us",
            "value": 332.406,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_untraced_100_us",
            "value": 117.011,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_traced_100_us",
            "value": 184.513,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_untraced_100_us",
            "value": 51.696,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_traced_100_us",
            "value": 631.051,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_untraced_100_us",
            "value": 169.373,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_traced_100_us",
            "value": 66.443,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_untraced_100_us",
            "value": 16.059,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_traced_100_us",
            "value": 52.213,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_untraced_100_us",
            "value": 10.535,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_traced_100_us",
            "value": 162.793,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_untraced_100_us",
            "value": 30.862,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_traced_100_us",
            "value": 325.725,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_untraced_100_us",
            "value": 112.903,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_traced_100_us",
            "value": 144.552,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_untraced_100_us",
            "value": 52.583,
            "unit": "us"
          },
          {
            "name": "solid_wac_named_graphs",
            "value": 1148,
            "unit": "count"
          },
          {
            "name": "solid_wac_quads",
            "value": 3060,
            "unit": "count"
          },
          {
            "name": "solid_wac_auth_triples",
            "value": 3783,
            "unit": "count"
          },
          {
            "name": "solid_acp_auth_triples",
            "value": 6355,
            "unit": "count"
          },
          {
            "name": "solid_alice_readable_graphs",
            "value": 800,
            "unit": "count"
          },
          {
            "name": "solid_full_dataset_rows",
            "value": 864,
            "unit": "count"
          },
          {
            "name": "solid_authorized_rows",
            "value": 599,
            "unit": "count"
          },
          {
            "name": "nlq_synth_triples",
            "value": 6000,
            "unit": "count"
          },
          {
            "name": "nlq_prompt_chars",
            "value": 1973,
            "unit": "chars"
          },
          {
            "name": "nlq_ask_repairs",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "nlq_ask_result_rows",
            "value": 2,
            "unit": "count"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.142,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1663825,
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
          "id": "683de009038a09fb651902608fb0db61e1661079",
          "message": "feat(sparq-serve): response-bytes result cache over PodEpochs (visibility-scope keyed, single-flight, off by default) (sq-jluc) (#936)\n\n* feat(sparq-serve): response-bytes result cache over PodEpochs (visibility-scope keyed, single-flight, off by default) (sq-jluc) [OPUS-4.8]\n\nImplements research/concurrent-serving.md §5 row 2 + §6.3: a serving-layer\ncache from a request identity to the complete pre-serialized response body, so\na repeated read returns bytes instead of re-executing. OPT-IN behind the\n`result-cache` cargo feature, OFF by default — the default build carries zero\ncache code and no new dependency (reuses in-tree rustc-hash + std).\n\nKey = (canonical-query x visibility-scope x per-pod epoch-vector):\n- canonical-query: cheap whitespace (label-safe default) + opt-in variable\n  renaming canonicalization (canon.rs);\n- visibility-scope: ScopeKey derived from the accessible graph SET identity\n  (AuthIndex::accessible), NEVER the WebID (the Hasura lesson). This is an\n  access-control CORRECTNESS invariant (a different scope MUST miss — tested),\n  documented honestly as NOT itself a privacy/confidentiality guarantee;\n- epoch-vector: each entry records the per-pod epoch of the graphs it touched;\n  a write that bumps any of them is a stale miss. Unbounded-footprint queries\n  pin the global generation (invalidated by any write).\n\nSingle-flight leases collapse a stampede on a hot uncached key into one\nexecution + N waiters (the one MQO survivor). Byte-budget LRU + admission cap\nnever caches oversize/streaming bodies. Library-first: sync, runtime-agnostic,\nno HTTP/async types, never depends on sparq-solid (the caller derives the scope\nkey + footprint and hands them in); the wasm dependency-direction guard stays 0.\n\nThis is a DIFFERENT layer from sparq-engine's embedded result-cache (the\nin-engine algebra-keyed LRU); the shared feature name is per-package, the\nboundary is spelled out in the module docs.\n\nTests (feature ON): hit/miss, the scope-isolation MISS invariant, per-pod and\nglobal-generation epoch invalidation, single-flight dedup (one execution under\na 16-thread stampede) + lease-abandon promotion, byte-budget LRU eviction,\noversize admission rejection, format keying, canonicalization hit. A\nnon-feature-gated cache_feature_state.rs pins the default-OFF substrate and\ngates the surface from both cfg arms.\n\nGates GREEN in BOTH states (build, clippy -D warnings, test): default (OFF) and\n--features result-cache.\n\nPERF VALIDATION IS CANONICAL-PENDING: the design's perf targets (scheduler+epoch\noverhead <=1%, all-distinct adversary <=0.2us/req, the Mreq/s throughput class)\nrequire a canonical host. This work-box is NON-CANONICAL and cannot produce them;\nno perf number is baked into docs/tests. Correctness is the gate satisfied here.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(sparq-serve): repair feature-gated rustdoc intra-doc links in result cache (sq-jluc) [OPUS-4.8]\n\nThe `result-cache`-gated cache.rs carried broken intra-doc links that failed\nthe CI lint job's `cargo doc --workspace --no-deps --all-features` step under\n`RUSTDOCFLAGS=-D warnings` (the only GATING failure on this PR; the default-\nfeatures rustdoc, the workspace `clippy -D warnings`, and tests in both feature\nstates were already clean):\n\n- A PUBLIC item (`CacheConfig::rename_variables`) linked `[`canon`](crate::canon)`\n  to the PRIVATE `canon` module -> `rustdoc::private_intra_doc_links` -> build\n  fails. Re-pointed it (and the `[`canon::canonicalize_renamed`]` link) at the\n  crate-root re-exports `crate::canonicalize` / `crate::canonicalize_renamed`.\n- Three module-doc links named `[`Lease::Hit/Lead/Wait`]`, but `Lease` is a\n  PRIVATE struct with no such variants; `ResultCache::lease` actually returns the\n  public enum `LeaseOutcome`. Corrected to `[`LeaseOutcome::Hit/Lead/Wait`]`.\n\nDocs-only, inside the feature-gated module; no cache-design change, no `#![allow]`.\nVerified the literal CI commands clean: `cargo doc --workspace --no-deps\n--all-features` (RUSTDOCFLAGS=-D warnings) EXIT 0; `cargo clippy -p sparq-serve\n--all-targets --all-features -- -D warnings` EXIT 0; `cargo test -p sparq-serve`\nboth default (OFF) and `--features result-cache` (ON) EXIT 0.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jeswrsolidserver@gmail.com>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-20T11:55:31+01:00",
          "tree_id": "e9f1d04514b5ce35bf91c3159bab5225ab7a36d0",
          "url": "https://github.com/jeswr/sparq/commit/683de009038a09fb651902608fb0db61e1661079"
        },
        "date": 1781954714276,
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
            "value": 3077.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4346,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 810,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 14174.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 57087.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 152306.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3907.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40793.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8791.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57920,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 157128.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3215.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40012.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 28619.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1286885.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6162.8,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3701.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3304.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7393.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 511298.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12560.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31109.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52390.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3665,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21132.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 126294.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 90349.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 153727,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 34099.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6288.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12478.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28543.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 21.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1295303.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6381.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3731.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3387.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8221.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 506544.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12694.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31174.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 52062,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3630.5,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21388,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 128214.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 92073.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 159057,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 34219.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7194.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12932.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 28342.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1284380.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6189.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3713.7,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3409.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8745.9,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 496797.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12369.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 32206.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 51810.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3626.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21179.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 129286.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 93493.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 156051,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7,
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
            "value": 33968.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6282.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12416.8,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 16.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6557,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15852.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14879.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14855.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 440800.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 16005.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22291.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 293974.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20838.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21795.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 294822.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9023.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16921.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14988.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14905.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 485380.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 17675.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22357.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 294601.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20581.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 74,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 21844.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 293515.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13727.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 18031.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14834.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14725.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 483570,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 18309.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22595.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 291784.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 21270.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 135.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 21584.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 293915.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_count_us",
            "value": 64.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_count_us",
            "value": 32,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_count_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_count_us",
            "value": 99.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_count_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_count_us",
            "value": 16.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_count_us",
            "value": 37.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_count_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_count_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_count_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_count_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_materialize_us",
            "value": 1262.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_materialize_us",
            "value": 42.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_materialize_us",
            "value": 43.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_materialize_us",
            "value": 163.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_materialize_us",
            "value": 31.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_materialize_us",
            "value": 28.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_materialize_us",
            "value": 22.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_materialize_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_materialize_us",
            "value": 18.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_materialize_us",
            "value": 131.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_materialize_us",
            "value": 30.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_materialize_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_materialize_us",
            "value": 23.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_materialize_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_json_us",
            "value": 1500.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_json_us",
            "value": 28.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_json_us",
            "value": 28.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_json_us",
            "value": 127.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_json_us",
            "value": 20.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_json_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_json_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_json_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_json_us",
            "value": 130.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_json_us",
            "value": 32.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_json_us",
            "value": 22.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_json_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_json_us",
            "value": 28,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 55.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 72,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 79.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 103,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 478.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 163.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 258.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 541.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 13.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 49.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 56.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 82.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 80.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 105,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 462.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 177.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 269.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 541.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 49,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 59.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 175.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 78.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 114,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 465.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 185.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 302.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 553,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47.7,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 592.8,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.3,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 61.8,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.8,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.9,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2693.2,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3853.3,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 16.3,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22.3,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.061,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "sameas_size8_closure_s",
            "value": 0,
            "unit": "s"
          },
          {
            "name": "sameas_size8_query_us",
            "value": 3.2,
            "unit": "us"
          },
          {
            "name": "sameas_size8_closure_triples",
            "value": 352,
            "unit": "triples"
          },
          {
            "name": "sameas_size32_closure_s",
            "value": 0.001,
            "unit": "s"
          },
          {
            "name": "sameas_size32_query_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "sameas_size32_closure_triples",
            "value": 4480,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 341.5,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 329.6,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13151.2,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6493,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 757202.3,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 183.1,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 103.7,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 105629.2,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 93.9,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.8,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.5,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.2,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3650.4,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.485186,
            "unit": "s"
          },
          {
            "name": "vectors_diskann_recall_at10",
            "value": 34,
            "unit": "milli"
          },
          {
            "name": "vectors_diskann_query_us",
            "value": 357.6,
            "unit": "us"
          },
          {
            "name": "vectors_hnsw_recall_at10",
            "value": 2,
            "unit": "milli"
          },
          {
            "name": "vectors_hnsw_query_us",
            "value": 391.8,
            "unit": "us"
          },
          {
            "name": "vectors_pq_recall_at10",
            "value": 22,
            "unit": "milli"
          },
          {
            "name": "vectors_pq_query_us",
            "value": 402.6,
            "unit": "us"
          },
          {
            "name": "vectors_build_s",
            "value": 41.931077,
            "unit": "s"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w0_rows",
            "value": 3,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_persistentdict_triples_per_s",
            "value": 2725464,
            "unit": "triples_per_s"
          },
          {
            "name": "snikmeta_triples",
            "value": 328,
            "unit": "count"
          },
          {
            "name": "snikmeta_terms",
            "value": 205,
            "unit": "count"
          },
          {
            "name": "snikmeta_distinct_predicates",
            "value": 23,
            "unit": "count"
          },
          {
            "name": "snikmeta_rdf_type_triples",
            "value": 49,
            "unit": "count"
          },
          {
            "name": "snikmeta_direct_eq_upstream",
            "value": 1,
            "unit": "count"
          },
          {
            "name": "hdt_load_s",
            "value": 0.042118,
            "unit": "s"
          },
          {
            "name": "hdt_vs_ntgz_load_s",
            "value": 3.3485,
            "unit": "ratio"
          },
          {
            "name": "zk_compose_filter_decimal_i3_f2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_gates",
            "value": 3113,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_hidden_issuer_d4_gates",
            "value": 16932,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_pok_gates",
            "value": 10334,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_set_d4_gates",
            "value": 10650,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb16_gates",
            "value": 7025,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb64_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb16_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb64_gates",
            "value": 18681,
            "unit": "gates"
          },
          {
            "name": "zk_compose_revoke_unset_d10_gates",
            "value": 899,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r4_gates",
            "value": 5991,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r8_gates",
            "value": 7038,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r4_gates",
            "value": 14923,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r8_gates",
            "value": 18850,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r4_gates",
            "value": 9254,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r8_gates",
            "value": 11261,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r4_gates",
            "value": 27054,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r8_gates",
            "value": 34821,
            "unit": "gates"
          },
          {
            "name": "zk_canon_bnode_1024_us",
            "value": 5836.877,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_256_us",
            "value": 1406.862,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_64_us",
            "value": 337.994,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_1024_us",
            "value": 4252.513,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_256_us",
            "value": 1007.2,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_64_us",
            "value": 237.36,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_1024_us",
            "value": 95365.789,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_256_us",
            "value": 23806.919,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_64_us",
            "value": 5937.755,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_1024_us",
            "value": 72254.061,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_256_us",
            "value": 18038.408,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_64_us",
            "value": 4512.003,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_1024_us",
            "value": 67896.865,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_256_us",
            "value": 16985.935,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_64_us",
            "value": 4226.287,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_hash40_us",
            "value": 185.371,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_permutation_us",
            "value": 13.107,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_traced_1000_us",
            "value": 4833.696,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_untraced_1000_us",
            "value": 1106.158,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_traced_1000_us",
            "value": 1719.392,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_untraced_1000_us",
            "value": 418.785,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_traced_1000_us",
            "value": 8138.079,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_untraced_1000_us",
            "value": 2418.552,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_traced_1000_us",
            "value": 579.376,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_untraced_1000_us",
            "value": 111.97,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_traced_1000_us",
            "value": 489.951,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_untraced_1000_us",
            "value": 72.324,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_traced_1000_us",
            "value": 1397.452,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_untraced_1000_us",
            "value": 264.21,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_traced_1000_us",
            "value": 3054.902,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_untraced_1000_us",
            "value": 1042.732,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_traced_1000_us",
            "value": 1347.312,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_untraced_1000_us",
            "value": 426.685,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_traced_100_us",
            "value": 330.625,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_untraced_100_us",
            "value": 112.788,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_traced_100_us",
            "value": 184.565,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_untraced_100_us",
            "value": 52.295,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_traced_100_us",
            "value": 643.007,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_untraced_100_us",
            "value": 166.147,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_traced_100_us",
            "value": 67.347,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_untraced_100_us",
            "value": 16.101,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_traced_100_us",
            "value": 52.339,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_untraced_100_us",
            "value": 10.198,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_traced_100_us",
            "value": 162.862,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_untraced_100_us",
            "value": 30.211,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_traced_100_us",
            "value": 316.352,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_untraced_100_us",
            "value": 109.917,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_traced_100_us",
            "value": 143.677,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_untraced_100_us",
            "value": 50.808,
            "unit": "us"
          },
          {
            "name": "solid_wac_named_graphs",
            "value": 1148,
            "unit": "count"
          },
          {
            "name": "solid_wac_quads",
            "value": 3060,
            "unit": "count"
          },
          {
            "name": "solid_wac_auth_triples",
            "value": 3783,
            "unit": "count"
          },
          {
            "name": "solid_acp_auth_triples",
            "value": 6355,
            "unit": "count"
          },
          {
            "name": "solid_alice_readable_graphs",
            "value": 800,
            "unit": "count"
          },
          {
            "name": "solid_full_dataset_rows",
            "value": 864,
            "unit": "count"
          },
          {
            "name": "solid_authorized_rows",
            "value": 599,
            "unit": "count"
          },
          {
            "name": "nlq_synth_triples",
            "value": 6000,
            "unit": "count"
          },
          {
            "name": "nlq_prompt_chars",
            "value": 1973,
            "unit": "chars"
          },
          {
            "name": "nlq_ask_repairs",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "nlq_ask_result_rows",
            "value": 2,
            "unit": "count"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.139,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1663825,
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
          "id": "c2b0106bdb4f84b41dd98515518d570f285cf5fc",
          "message": "feat(site): persistent cross-session workspace model + Tauri/web/memory persistence (sq-atb0) [OPUS-4.8] (#965)\n\nAdds a persistent, cross-session WORKSPACE model to the GUI so a user's work\nsurvives an app/browser restart (epic sq-ixc3, bead sq-atb0).\n\nA named workspace holds: a SNAPSHOT of the loaded dataset (whole default+named-\ngraph content as N-Quads — a save/open cache per the maintainer decision on\n#757, NOT a re-ingest-from-source on open, mirroring the engine online-snapshot\nprimitive sq-o5bi); the imported-source metadata (local + URL, with the URL kept\nso a remote source can be re-fetched); and the SPARQL editor state (query text +\nrun mode + endpoint URL — never a bearer token).\n\nPersistence is ONE abstraction with three runtime-selected backends\n(@sparq/client workspace.ts, framework-agnostic): Tauri local-disk on the desktop\napp (feature-detected, when the shell grants the fs capability), browser\nlocalStorage on GitHub Pages (the static-export path + the Tauri-webview fallback),\nand an in-memory session fallback. The static export never depends on a Tauri API\n(the plugin is a webpackIgnore'd runtime-only dynamic import, gated by\nisTauriRuntime()); the last workspace re-hydrates on startup.\n\nHonest limitation, stated plainly in the UI + README: a previously chosen local\nfile cannot be silently re-read across sessions (the browser keeps no persistent\nhandle), so the snapshot is a local import's durable copy.\n\n- packages/sparq-client/src/workspace.ts — the model + WorkspaceStore interface +\n  Web/Memory/Tauri backends + createWorkspaceStore feature-detection factory\n- site/src/lib/{workspace-snapshot,use-workspaces,tauri-fs}.ts — site-side glue\n- site/src/components/workspace-panel.tsx — the workspace switcher + honesty copy\n- site/src/components/repl{,-datasets}.tsx — wired into the REPL (source capture,\n  save/open/new/rename/delete, startup re-hydration)\n- site/test/workspace.test.mjs — 248-test suite green (model, all three backends,\n  the selection factory)\n\nGates: static export (Pages + Tauri modes) GREEN, lint clean, tsc --noEmit clean\n(site + @sparq/client), unit suite green. No Tauri package shipped in the static\nbundle. No bearer token ever persisted.\n\nCo-authored-by: Jesse Wright <jmwright.045@gmail.com>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-20T11:24:51Z",
          "tree_id": "8adb8f521488f69f95070e496cf75456e82bc73b",
          "url": "https://github.com/jeswr/sparq/commit/c2b0106bdb4f84b41dd98515518d570f285cf5fc"
        },
        "date": 1781955626556,
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
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3351.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4861.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 786,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13436.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 61070,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 161668,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4256.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42822.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8192.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57795.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 151649.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2925.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38683.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29665.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1451885.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6112.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3721,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3581.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7390.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 483796.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12955.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31262.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53018.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3833,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22220,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 135932.3,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 102166.8,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 171585,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36073.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6827.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13182,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29701.8,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 25.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1469557.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6088,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3805.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3650.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8874.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 484792.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 13193.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30901.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 54046,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3828.5,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22041,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 134471.2,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 100965.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 171483.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.2,
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
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36020.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6962.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13096,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29862.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 19,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1435673.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6438.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3855.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3631.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8816.1,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 482604.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13094.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30812.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 52813.9,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 4142.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22078.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 136392.7,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 103607.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 169554.5,
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
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 35079.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7189.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12969.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 7126.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16672.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16488.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16348,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 473769.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17755.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24156.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 302894,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 23176.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 23770.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 301809.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 10064.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 18358.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16460.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16364.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 506052.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 20965.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24319.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 303180,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 23065.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 51.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23310.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 310105.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 19.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 14693.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20595.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16714.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16665.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 527448.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 20617.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24516.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 303196.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23007.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 130.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 24463.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 305786.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_count_us",
            "value": 61.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_count_us",
            "value": 33,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_count_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_count_us",
            "value": 98.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_count_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_count_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_count_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_count_us",
            "value": 32.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_count_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_materialize_us",
            "value": 933.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_materialize_us",
            "value": 25.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_materialize_us",
            "value": 26.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_materialize_us",
            "value": 101.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_materialize_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_materialize_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_materialize_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_materialize_us",
            "value": 109.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_materialize_us",
            "value": 31.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_materialize_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_materialize_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_materialize_us",
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_materialize_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_json_us",
            "value": 1542.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_json_us",
            "value": 28.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_json_us",
            "value": 28.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_json_us",
            "value": 126.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_json_us",
            "value": 20.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_json_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_json_us",
            "value": 21.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_json_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_json_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_json_us",
            "value": 111.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_json_us",
            "value": 33,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_json_us",
            "value": 21.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_json_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_json_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_json_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 57.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 67,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 74.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 101.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 498.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 170.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 280.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 612.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 47.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 54.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 80.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 77.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 102.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 494.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 178.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 295.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 612.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 46.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 59.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 161.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 75.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 113.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 495.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 195.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 320.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 642.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 45.8,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 611.3,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.9,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 67.4,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 28.7,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 34.8,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2915.1,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4064,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.6,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.063,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "sameas_size8_closure_s",
            "value": 0,
            "unit": "s"
          },
          {
            "name": "sameas_size8_query_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "sameas_size8_closure_triples",
            "value": 352,
            "unit": "triples"
          },
          {
            "name": "sameas_size32_closure_s",
            "value": 0.001,
            "unit": "s"
          },
          {
            "name": "sameas_size32_query_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "sameas_size32_closure_triples",
            "value": 4480,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 330.4,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 303.7,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13322.4,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6318.6,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 681021.2,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 144.6,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 87.8,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 108065.7,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 95,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.3,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.4,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3858.5,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.491894,
            "unit": "s"
          },
          {
            "name": "vectors_diskann_recall_at10",
            "value": 34,
            "unit": "milli"
          },
          {
            "name": "vectors_diskann_query_us",
            "value": 345.1,
            "unit": "us"
          },
          {
            "name": "vectors_hnsw_recall_at10",
            "value": 2,
            "unit": "milli"
          },
          {
            "name": "vectors_hnsw_query_us",
            "value": 441.7,
            "unit": "us"
          },
          {
            "name": "vectors_pq_recall_at10",
            "value": 22,
            "unit": "milli"
          },
          {
            "name": "vectors_pq_query_us",
            "value": 424.9,
            "unit": "us"
          },
          {
            "name": "vectors_build_s",
            "value": 41.120125,
            "unit": "s"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w0_rows",
            "value": 3,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_persistentdict_triples_per_s",
            "value": 2727763,
            "unit": "triples_per_s"
          },
          {
            "name": "snikmeta_triples",
            "value": 328,
            "unit": "count"
          },
          {
            "name": "snikmeta_terms",
            "value": 205,
            "unit": "count"
          },
          {
            "name": "snikmeta_distinct_predicates",
            "value": 23,
            "unit": "count"
          },
          {
            "name": "snikmeta_rdf_type_triples",
            "value": 49,
            "unit": "count"
          },
          {
            "name": "snikmeta_direct_eq_upstream",
            "value": 1,
            "unit": "count"
          },
          {
            "name": "hdt_load_s",
            "value": 0.042203,
            "unit": "s"
          },
          {
            "name": "hdt_vs_ntgz_load_s",
            "value": 3.4099,
            "unit": "ratio"
          },
          {
            "name": "zk_compose_filter_decimal_i3_f2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_gates",
            "value": 3113,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_hidden_issuer_d4_gates",
            "value": 16932,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_pok_gates",
            "value": 10334,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_set_d4_gates",
            "value": 10650,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb16_gates",
            "value": 7025,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb64_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb16_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb64_gates",
            "value": 18681,
            "unit": "gates"
          },
          {
            "name": "zk_compose_revoke_unset_d10_gates",
            "value": 899,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r4_gates",
            "value": 5991,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r8_gates",
            "value": 7038,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r4_gates",
            "value": 14923,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r8_gates",
            "value": 18850,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r4_gates",
            "value": 9254,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r8_gates",
            "value": 11261,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r4_gates",
            "value": 27054,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r8_gates",
            "value": 34821,
            "unit": "gates"
          },
          {
            "name": "zk_canon_bnode_1024_us",
            "value": 5895.482,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_256_us",
            "value": 1444.266,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_64_us",
            "value": 345.071,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_1024_us",
            "value": 4417.225,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_256_us",
            "value": 1044.989,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_64_us",
            "value": 246.598,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_1024_us",
            "value": 103946.003,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_256_us",
            "value": 25951.578,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_64_us",
            "value": 6456.957,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_1024_us",
            "value": 78864.489,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_256_us",
            "value": 19648.816,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_64_us",
            "value": 4904.455,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_1024_us",
            "value": 73884.888,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_256_us",
            "value": 18477.814,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_64_us",
            "value": 4580.297,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_hash40_us",
            "value": 202.647,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_permutation_us",
            "value": 14.369,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_traced_1000_us",
            "value": 5244.051,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_untraced_1000_us",
            "value": 1110.267,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_traced_1000_us",
            "value": 1761.712,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_untraced_1000_us",
            "value": 425.668,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_traced_1000_us",
            "value": 8565.362,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_untraced_1000_us",
            "value": 2553.02,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_traced_1000_us",
            "value": 608.236,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_untraced_1000_us",
            "value": 129.805,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_traced_1000_us",
            "value": 500.908,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_untraced_1000_us",
            "value": 78.836,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_traced_1000_us",
            "value": 1395.651,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_untraced_1000_us",
            "value": 265.867,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_traced_1000_us",
            "value": 3212.867,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_untraced_1000_us",
            "value": 1094.363,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_traced_1000_us",
            "value": 1355.611,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_untraced_1000_us",
            "value": 435.903,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_traced_100_us",
            "value": 333.756,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_untraced_100_us",
            "value": 115.333,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_traced_100_us",
            "value": 187.44,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_untraced_100_us",
            "value": 53.235,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_traced_100_us",
            "value": 652.896,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_untraced_100_us",
            "value": 170.974,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_traced_100_us",
            "value": 68.659,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_untraced_100_us",
            "value": 17.769,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_traced_100_us",
            "value": 55.217,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_untraced_100_us",
            "value": 10.871,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_traced_100_us",
            "value": 162.915,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_untraced_100_us",
            "value": 29.667,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_traced_100_us",
            "value": 323.525,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_untraced_100_us",
            "value": 115.214,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_traced_100_us",
            "value": 148.319,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_untraced_100_us",
            "value": 52.627,
            "unit": "us"
          },
          {
            "name": "solid_wac_named_graphs",
            "value": 1148,
            "unit": "count"
          },
          {
            "name": "solid_wac_quads",
            "value": 3060,
            "unit": "count"
          },
          {
            "name": "solid_wac_auth_triples",
            "value": 3783,
            "unit": "count"
          },
          {
            "name": "solid_acp_auth_triples",
            "value": 6355,
            "unit": "count"
          },
          {
            "name": "solid_alice_readable_graphs",
            "value": 800,
            "unit": "count"
          },
          {
            "name": "solid_full_dataset_rows",
            "value": 864,
            "unit": "count"
          },
          {
            "name": "solid_authorized_rows",
            "value": 599,
            "unit": "count"
          },
          {
            "name": "nlq_synth_triples",
            "value": 6000,
            "unit": "count"
          },
          {
            "name": "nlq_prompt_chars",
            "value": 1973,
            "unit": "chars"
          },
          {
            "name": "nlq_ask_repairs",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "nlq_ask_result_rows",
            "value": 2,
            "unit": "count"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.145,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1663825,
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
          "id": "1a46e526cd10953d5acb2f8bee89c6831a4759f6",
          "message": "fix(sparq-engine): four JSON-LD 1.1 Compaction correctness bugs (data-loss + over-restriction) (sq-oy1f.8/.9/.10/.11) [OPUS-4.8] (#963)\n\nFixes four execution-confirmed compaction bugs found by the adversarial audit\ndifferential-tested against the pyld W3C reference processor. All behind the\nopt-in `serialize-rdf` feature; default build byte-identical (the whole\n`serialize` module compiles out when the feature is off).\n\nP1 data-loss (restore the documented losslessness invariant):\n\n- sq-oy1f.8 @list container: only unwrap a PURE single `{\"@list\":…}` value;\n  co-located non-list siblings now emit under the property IRI compacted\n  without the list term (new `compact_iri_no_list`), never dropped. pyld\n  toRdf on our output reproduces all 6 triples.\n- sq-oy1f.9 @language container: a value lacking a usable `@language` (plain\n  string or typed/numeric literal) now falls under the reserved `@none`\n  member (matching pyld) instead of being silently skipped. Reader re-expands\n  `@none` via the value path so the round-trip keeps the datatype.\n- sq-oy1f.10 @reverse: track the exact relocated (subject,predicate,object)\n  edges and strip ONLY those; an edge to a non-subject object survives as a\n  forward property instead of being bulk-stripped. pyld toRdf reproduces all\n  3 triples incl. the previously-dropped eve->frank edge.\n\nP3 output-quality (lossless):\n\n- sq-oy1f.11 @vocab-relative: drop the over-restrictive `/`+`#` exclusion so a\n  fragment-bearing suffix compacts to `ns#value` (matching pyld). The `:`\n  exclusion is deliberately kept — a `:`-suffix is ambiguous with a compact\n  IRI on read-back, so emitting it would be a lossy round-trip.\n\nTests: 4 permanent regression tests (each bead's repro) asserting the\nspec-correct (pyld-verified) shape AND the losslessness round-trip the\noriginal tests lacked. Also fixes two reader (test-inverse) gaps surfaced by\nthe new cases: alias-aware `@id` in `@reverse` objects, and `@none` handling\nin the `@language` map. 21 compaction tests + full sparq-engine lib suite (267)\ngreen; default-feature + serialize-rdf clippy `-D warnings` green; rustdoc\n`--all-features -D warnings` clean.\n\nHonest interop caveat documented in skills/data-formats/SKILL.md: round-trip\nlosslessness is verified against sparq's own JSON-LD->RDF reader; two output\nshapes are not faithfully re-expandable by a strict external processor (a\n`@reverse`-term document double-inverts, and a non-string `@none` language-map\nvalue is invalid per spec) — pre-existing boundaries, tracked as follow-ups.\n\n🤖 SPARQ agent\n\nCo-authored-by: Jesse Wright <jmwright.045@gmail.com>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-20T11:25:29Z",
          "tree_id": "0bce0386a9e625c316e674c4ec2fa6d5d1296ac1",
          "url": "https://github.com/jeswr/sparq/commit/1a46e526cd10953d5acb2f8bee89c6831a4759f6"
        },
        "date": 1781956659554,
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
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3314.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4849.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 801.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13272,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59237.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 161785,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3817.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41945.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8106.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 56858.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 157836.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2491.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39282.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29472.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 23.7,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1443957.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6246.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3797,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3591.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7245.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 491979.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12555.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31636.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52710.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 4098,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21818.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 133684,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 102417.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 169372,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.5,
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
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 34681,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6656.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12990.6,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29396.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1438876.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6267.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3808.6,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3569.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9960.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 485160.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12968.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31029.8,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53506.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3737.3,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21867.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 133025,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 101660.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 169947.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 35287,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6934.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13037.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29776.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 20,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1439445.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6198.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3786.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3596.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8300.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 480363.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13092,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30831.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53414.6,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3779.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21847.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 130248.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 100828,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 167413.3,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 36659.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6749.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12989.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 7134,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16878.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16436.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16400.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 460923,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 18056.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24719.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 301343.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 23015.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 24620.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 302538.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 12.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 10167,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 18501.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16673.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16524.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 507295.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 20281.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24524.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 308827.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 23367.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 64.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23909.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 302528.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 14147.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20890.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16734.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16652.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 509558.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 20638.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24625.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 309087.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23565.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 132.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 23484.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 303901.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_count_us",
            "value": 60.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_count_us",
            "value": 33.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_count_us",
            "value": 28.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_count_us",
            "value": 98.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_count_us",
            "value": 18.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_count_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_count_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_count_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_count_us",
            "value": 32.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_count_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_materialize_us",
            "value": 938.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_materialize_us",
            "value": 27.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_materialize_us",
            "value": 27.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_materialize_us",
            "value": 115.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_materialize_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_materialize_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_materialize_us",
            "value": 14.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_materialize_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_materialize_us",
            "value": 109.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_materialize_us",
            "value": 31.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_materialize_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_materialize_us",
            "value": 15.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_materialize_us",
            "value": 23.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_materialize_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_json_us",
            "value": 1539.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_json_us",
            "value": 28,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_json_us",
            "value": 28.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_json_us",
            "value": 122.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_json_us",
            "value": 19.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_json_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_json_us",
            "value": 21.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_json_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_json_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_json_us",
            "value": 111.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_json_us",
            "value": 34.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_json_us",
            "value": 21.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_json_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_json_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_json_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 55.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 67.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 75.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 101.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 504.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 172.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 280.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 608.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 46.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 54.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 86.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 74.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 103.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 505.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 183.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 284.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 596.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 48.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 57.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 165.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 76.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 107.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 507.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 202.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 338.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 612.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47.1,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 612,
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
            "value": 66.1,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.6,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2939.3,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4056.6,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 25.6,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.063,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "sameas_size8_closure_s",
            "value": 0,
            "unit": "s"
          },
          {
            "name": "sameas_size8_query_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "sameas_size8_closure_triples",
            "value": 352,
            "unit": "triples"
          },
          {
            "name": "sameas_size32_closure_s",
            "value": 0.001,
            "unit": "s"
          },
          {
            "name": "sameas_size32_query_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "sameas_size32_closure_triples",
            "value": 4480,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 337.4,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 309.3,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13407.5,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6265.1,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 689058.2,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 154.1,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 82.6,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 107480.3,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 95.1,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.5,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.3,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3899.3,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.467185,
            "unit": "s"
          },
          {
            "name": "vectors_diskann_recall_at10",
            "value": 34,
            "unit": "milli"
          },
          {
            "name": "vectors_diskann_query_us",
            "value": 352.5,
            "unit": "us"
          },
          {
            "name": "vectors_hnsw_recall_at10",
            "value": 1,
            "unit": "milli"
          },
          {
            "name": "vectors_hnsw_query_us",
            "value": 457.2,
            "unit": "us"
          },
          {
            "name": "vectors_pq_recall_at10",
            "value": 22,
            "unit": "milli"
          },
          {
            "name": "vectors_pq_query_us",
            "value": 429.3,
            "unit": "us"
          },
          {
            "name": "vectors_build_s",
            "value": 41.219932,
            "unit": "s"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w0_rows",
            "value": 3,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_persistentdict_triples_per_s",
            "value": 2698613,
            "unit": "triples_per_s"
          },
          {
            "name": "snikmeta_triples",
            "value": 328,
            "unit": "count"
          },
          {
            "name": "snikmeta_terms",
            "value": 205,
            "unit": "count"
          },
          {
            "name": "snikmeta_distinct_predicates",
            "value": 23,
            "unit": "count"
          },
          {
            "name": "snikmeta_rdf_type_triples",
            "value": 49,
            "unit": "count"
          },
          {
            "name": "snikmeta_direct_eq_upstream",
            "value": 1,
            "unit": "count"
          },
          {
            "name": "hdt_load_s",
            "value": 0.043474,
            "unit": "s"
          },
          {
            "name": "hdt_vs_ntgz_load_s",
            "value": 3.3893,
            "unit": "ratio"
          },
          {
            "name": "zk_compose_filter_decimal_i3_f2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_gates",
            "value": 3113,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_hidden_issuer_d4_gates",
            "value": 16932,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_pok_gates",
            "value": 10334,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_set_d4_gates",
            "value": 10650,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb16_gates",
            "value": 7025,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb64_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb16_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb64_gates",
            "value": 18681,
            "unit": "gates"
          },
          {
            "name": "zk_compose_revoke_unset_d10_gates",
            "value": 899,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r4_gates",
            "value": 5991,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r8_gates",
            "value": 7038,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r4_gates",
            "value": 14923,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r8_gates",
            "value": 18850,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r4_gates",
            "value": 9254,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r8_gates",
            "value": 11261,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r4_gates",
            "value": 27054,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r8_gates",
            "value": 34821,
            "unit": "gates"
          },
          {
            "name": "zk_canon_bnode_1024_us",
            "value": 5879.721,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_256_us",
            "value": 1441.699,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_64_us",
            "value": 345.015,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_1024_us",
            "value": 4368.817,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_256_us",
            "value": 1043.101,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_64_us",
            "value": 244.338,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_1024_us",
            "value": 103946.956,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_256_us",
            "value": 25957.293,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_64_us",
            "value": 6454.62,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_1024_us",
            "value": 78710.499,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_256_us",
            "value": 19682.496,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_64_us",
            "value": 4900.464,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_1024_us",
            "value": 74048.733,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_256_us",
            "value": 18496.861,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_64_us",
            "value": 4591.019,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_hash40_us",
            "value": 202.941,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_permutation_us",
            "value": 14.353,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_traced_1000_us",
            "value": 5210.978,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_untraced_1000_us",
            "value": 1118.145,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_traced_1000_us",
            "value": 1771.686,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_untraced_1000_us",
            "value": 429.929,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_traced_1000_us",
            "value": 8628.204,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_untraced_1000_us",
            "value": 2579.438,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_traced_1000_us",
            "value": 617.259,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_untraced_1000_us",
            "value": 130.845,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_traced_1000_us",
            "value": 506.933,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_untraced_1000_us",
            "value": 79.838,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_traced_1000_us",
            "value": 1422.38,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_untraced_1000_us",
            "value": 266.073,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_traced_1000_us",
            "value": 3237.718,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_untraced_1000_us",
            "value": 1107.307,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_traced_1000_us",
            "value": 1363.516,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_untraced_1000_us",
            "value": 445.001,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_traced_100_us",
            "value": 333.353,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_untraced_100_us",
            "value": 115.684,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_traced_100_us",
            "value": 186.899,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_untraced_100_us",
            "value": 52.402,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_traced_100_us",
            "value": 670.129,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_untraced_100_us",
            "value": 184.897,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_traced_100_us",
            "value": 68.612,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_untraced_100_us",
            "value": 17.565,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_traced_100_us",
            "value": 54.03,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_untraced_100_us",
            "value": 10.874,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_traced_100_us",
            "value": 163.947,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_untraced_100_us",
            "value": 30.122,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_traced_100_us",
            "value": 325.472,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_untraced_100_us",
            "value": 114.974,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_traced_100_us",
            "value": 145.545,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_untraced_100_us",
            "value": 52.749,
            "unit": "us"
          },
          {
            "name": "solid_wac_named_graphs",
            "value": 1148,
            "unit": "count"
          },
          {
            "name": "solid_wac_quads",
            "value": 3060,
            "unit": "count"
          },
          {
            "name": "solid_wac_auth_triples",
            "value": 3783,
            "unit": "count"
          },
          {
            "name": "solid_acp_auth_triples",
            "value": 6355,
            "unit": "count"
          },
          {
            "name": "solid_alice_readable_graphs",
            "value": 800,
            "unit": "count"
          },
          {
            "name": "solid_full_dataset_rows",
            "value": 864,
            "unit": "count"
          },
          {
            "name": "solid_authorized_rows",
            "value": 599,
            "unit": "count"
          },
          {
            "name": "nlq_synth_triples",
            "value": 6000,
            "unit": "count"
          },
          {
            "name": "nlq_prompt_chars",
            "value": 1973,
            "unit": "chars"
          },
          {
            "name": "nlq_ask_repairs",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "nlq_ask_result_rows",
            "value": 2,
            "unit": "count"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.142,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1663825,
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
          "id": "44f585b3e2e42784b1df7f16403c046df3e0d3ea",
          "message": "fix(ci/sbom): harden js-sbom lane against external dev-deps on a published member (sq-pl1p) [OPUS-4.8] (#968)\n\nThe js-sbom lane DERIVES a standalone per-member lock (derive-workspace-member-lock.mjs)\nthen runs cyclonedx-npm twice (runtime --omit=dev + full dev). sq-f04e/#887 hardened only\nthe workspace-LINK case. When a PUBLISHED member (js/ = @jeswr/sparq) declares an EXTERNAL\n(registry) devDependency, the derive script projects its full transitive closure into the\nlock; a real dev-dep tree's unsatisfied peerDeps make cyclonedx-npm's FULL `npm ls`\n(no --omit) exit non-zero -> cyclonedx throws (ignoreNpmErrors=false default) -> exit 254,\nGATING the SBOM lane (the #922 class; #922 worked around it by relocating dev-deps to root).\n\nProper fix:\n- runtime SBOM stays STRICT (no --ignore-npm-errors): `--omit=dev` prunes the dev subtree\n  BEFORE validation, so the consumer-facing SBOM keeps full validation + the full runtime\n  surface (fzstd). A genuine runtime-graph conflict still reds the lane.\n- dev (full) SBOM passes cyclonedx `--ignore-npm-errors`: suppresses only npm ls's non-zero\n  EXIT (install-free lock-validation noise) while STILL enumerating the full component set —\n  nothing dropped from the build-time SBOM.\n- HONESTY GUARD: gen-js-sbom.sh now asserts the dev SBOM is a purl-superset of the runtime\n  SBOM, so the relaxed dev pass can never silently drop a real runtime component.\n- regression guard: scripts/tests/test_js_sbom_external_devdep.sh (hermetic, synthetic root\n  lock, node+npm only) pins runtime-strict + dev-break-is-real + runtime-honesty; wired into\n  the gating ci-scripts job in docs-quality.yml (shellcheck + run).\n\nVerified locally: real gen-js-sbom.sh produces both SBOMs + guard passes; both regression\ntests green; YAML/actionlint/shellcheck clean.\n\nRefs sq-pl1p, #887 (sq-f04e), #922.\n\nCo-authored-by: Jesse Wright <jmwright.045@gmail.com>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-20T13:28:52+01:00",
          "tree_id": "bdef66b5ab6d1b69c6835a1e7d44264ae32efea9",
          "url": "https://github.com/jeswr/sparq/commit/44f585b3e2e42784b1df7f16403c046df3e0d3ea"
        },
        "date": 1781959430196,
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
            "value": 3.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3085,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4358.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 812.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12935.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56974.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 156116.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3283.3,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41881.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8939.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 59801.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 160556.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3098.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40699.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29232.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1280021.7,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6392.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3733,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3483,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7415.9,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 516987.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 13092.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31763.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52310.9,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3803.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21298.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 132233.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 95638.8,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 165848,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 9,
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
            "value": 34466.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7202.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12761.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28851.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1224720.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6084.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3640.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3368.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8396.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 504077.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12889.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 32316.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 52456.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3704.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21575.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 130798.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 93937.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 162929.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 34327.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7483.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13104.3,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29137.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1590682.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6343.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3848.8,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3427.7,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9313.1,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 508820.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12777.9,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 32955.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53904.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3737.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22203.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 132674.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 99830.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 163166.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 34481.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6588.5,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12789.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 16.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6588.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15226.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14978.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14976.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 440435.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 16117.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22551.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 296198.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 21924.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 24250.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 295272.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 10306.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 18190.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 15241.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14899.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 494735.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18757.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 23955,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 299178.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 21712.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 58,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 25217.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 300187.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 16495.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20705.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 15283.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 15377.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 503827.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 19107.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 23884.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 298276.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 21551,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 135,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 24490.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 296628.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_count_us",
            "value": 60.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_count_us",
            "value": 33.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_count_us",
            "value": 29.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_count_us",
            "value": 102.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_count_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_count_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_count_us",
            "value": 39.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_count_us",
            "value": 14.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_count_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_count_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_count_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_materialize_us",
            "value": 876.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_materialize_us",
            "value": 26.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_materialize_us",
            "value": 27.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_materialize_us",
            "value": 111.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_materialize_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_materialize_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_materialize_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_materialize_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_materialize_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_materialize_us",
            "value": 122,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_materialize_us",
            "value": 30.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_materialize_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_materialize_us",
            "value": 15.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_materialize_us",
            "value": 23.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_materialize_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_materialize_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_json_us",
            "value": 1504.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_json_us",
            "value": 29.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_json_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_json_us",
            "value": 126.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_json_us",
            "value": 20.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_json_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_json_us",
            "value": 21.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_json_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_json_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_json_us",
            "value": 126.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_json_us",
            "value": 32.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_json_us",
            "value": 22.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_json_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_json_us",
            "value": 28.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_json_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 55,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 68.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 88.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 103.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 456.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 162.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 261.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 549.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 48,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 55.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 91.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 78.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 105,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 470,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 170,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 264.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 544.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 49.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 57.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 172,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 79.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 110.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 469,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 185.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 299.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 551.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 48.3,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 588.8,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 85.3,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 27.6,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.9,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2666.1,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3860.5,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 23.1,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.065,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "sameas_size8_closure_s",
            "value": 0,
            "unit": "s"
          },
          {
            "name": "sameas_size8_query_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "sameas_size8_closure_triples",
            "value": 352,
            "unit": "triples"
          },
          {
            "name": "sameas_size32_closure_s",
            "value": 0.001,
            "unit": "s"
          },
          {
            "name": "sameas_size32_query_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "sameas_size32_closure_triples",
            "value": 4480,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 352.8,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 319,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13386.9,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6546.9,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 761012.2,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 152.3,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 88.3,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 106116.8,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 93.9,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.7,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.4,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.9,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3692.4,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.461472,
            "unit": "s"
          },
          {
            "name": "vectors_diskann_recall_at10",
            "value": 34,
            "unit": "milli"
          },
          {
            "name": "vectors_diskann_query_us",
            "value": 338.7,
            "unit": "us"
          },
          {
            "name": "vectors_hnsw_recall_at10",
            "value": 1,
            "unit": "milli"
          },
          {
            "name": "vectors_hnsw_query_us",
            "value": 389.1,
            "unit": "us"
          },
          {
            "name": "vectors_pq_recall_at10",
            "value": 22,
            "unit": "milli"
          },
          {
            "name": "vectors_pq_query_us",
            "value": 404.2,
            "unit": "us"
          },
          {
            "name": "vectors_build_s",
            "value": 42.002607,
            "unit": "s"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w0_rows",
            "value": 3,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_persistentdict_triples_per_s",
            "value": 2687006,
            "unit": "triples_per_s"
          },
          {
            "name": "snikmeta_triples",
            "value": 328,
            "unit": "count"
          },
          {
            "name": "snikmeta_terms",
            "value": 205,
            "unit": "count"
          },
          {
            "name": "snikmeta_distinct_predicates",
            "value": 23,
            "unit": "count"
          },
          {
            "name": "snikmeta_rdf_type_triples",
            "value": 49,
            "unit": "count"
          },
          {
            "name": "snikmeta_direct_eq_upstream",
            "value": 1,
            "unit": "count"
          },
          {
            "name": "hdt_load_s",
            "value": 0.041851,
            "unit": "s"
          },
          {
            "name": "hdt_vs_ntgz_load_s",
            "value": 3.3574,
            "unit": "ratio"
          },
          {
            "name": "zk_compose_filter_decimal_i3_f2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_gates",
            "value": 3113,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_hidden_issuer_d4_gates",
            "value": 16932,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_pok_gates",
            "value": 10334,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_set_d4_gates",
            "value": 10650,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb16_gates",
            "value": 7025,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb64_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb16_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb64_gates",
            "value": 18681,
            "unit": "gates"
          },
          {
            "name": "zk_compose_revoke_unset_d10_gates",
            "value": 899,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r4_gates",
            "value": 5991,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r8_gates",
            "value": 7038,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r4_gates",
            "value": 14923,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r8_gates",
            "value": 18850,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r4_gates",
            "value": 9254,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r8_gates",
            "value": 11261,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r4_gates",
            "value": 27054,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r8_gates",
            "value": 34821,
            "unit": "gates"
          },
          {
            "name": "zk_canon_bnode_1024_us",
            "value": 5844.971,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_256_us",
            "value": 1408.829,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_64_us",
            "value": 342.086,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_1024_us",
            "value": 4243.082,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_256_us",
            "value": 1009.66,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_64_us",
            "value": 243.972,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_1024_us",
            "value": 95154.707,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_256_us",
            "value": 23963.414,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_64_us",
            "value": 5939.119,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_1024_us",
            "value": 71944.102,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_256_us",
            "value": 18007.508,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_64_us",
            "value": 4502.909,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_1024_us",
            "value": 67722.726,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_256_us",
            "value": 16941.326,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_64_us",
            "value": 4219.474,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_hash40_us",
            "value": 184.281,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_permutation_us",
            "value": 13.126,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_traced_1000_us",
            "value": 4813.331,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_untraced_1000_us",
            "value": 1096.777,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_traced_1000_us",
            "value": 1697.707,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_untraced_1000_us",
            "value": 412.672,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_traced_1000_us",
            "value": 7985.917,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_untraced_1000_us",
            "value": 2413.795,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_traced_1000_us",
            "value": 569.91,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_untraced_1000_us",
            "value": 111.95,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_traced_1000_us",
            "value": 474.765,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_untraced_1000_us",
            "value": 75.212,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_traced_1000_us",
            "value": 1368.047,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_untraced_1000_us",
            "value": 266.132,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_traced_1000_us",
            "value": 3039.909,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_untraced_1000_us",
            "value": 1053.857,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_traced_1000_us",
            "value": 1321.803,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_untraced_1000_us",
            "value": 426.251,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_traced_100_us",
            "value": 331.204,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_untraced_100_us",
            "value": 115.204,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_traced_100_us",
            "value": 185.457,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_untraced_100_us",
            "value": 53.846,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_traced_100_us",
            "value": 629.261,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_untraced_100_us",
            "value": 166.45,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_traced_100_us",
            "value": 66.553,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_untraced_100_us",
            "value": 16.429,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_traced_100_us",
            "value": 52.067,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_untraced_100_us",
            "value": 10.535,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_traced_100_us",
            "value": 162.647,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_untraced_100_us",
            "value": 30.294,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_traced_100_us",
            "value": 316.447,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_untraced_100_us",
            "value": 111.853,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_traced_100_us",
            "value": 144.092,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_untraced_100_us",
            "value": 51.72,
            "unit": "us"
          },
          {
            "name": "solid_wac_named_graphs",
            "value": 1148,
            "unit": "count"
          },
          {
            "name": "solid_wac_quads",
            "value": 3060,
            "unit": "count"
          },
          {
            "name": "solid_wac_auth_triples",
            "value": 3783,
            "unit": "count"
          },
          {
            "name": "solid_acp_auth_triples",
            "value": 6355,
            "unit": "count"
          },
          {
            "name": "solid_alice_readable_graphs",
            "value": 800,
            "unit": "count"
          },
          {
            "name": "solid_full_dataset_rows",
            "value": 864,
            "unit": "count"
          },
          {
            "name": "solid_authorized_rows",
            "value": 599,
            "unit": "count"
          },
          {
            "name": "nlq_synth_triples",
            "value": 6000,
            "unit": "count"
          },
          {
            "name": "nlq_prompt_chars",
            "value": 1973,
            "unit": "chars"
          },
          {
            "name": "nlq_ask_repairs",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "nlq_ask_result_rows",
            "value": 2,
            "unit": "count"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.139,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1663825,
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
          "id": "839014efb88b7517518d46a0c3a84b001e45e000",
          "message": "feat(sparq-serve/sparq-server): online consistent snapshot backup + restore (gated /admin/backup + /admin/restore, feature `backup`) (sq-o5bi) [OPUS-4.8] (#941)\n\n* feat(sparq-serve/sparq-server): online consistent snapshot backup + restore (gated /admin/backup + /admin/restore, feature `backup`) (sq-o5bi) [OPUS-4.8]\n\nImplements ONLINE backup/restore of the in-memory serving store. sparq-serve owns\nthe artifact format (export/import of an already-immutable pinned Generation —\ntriples + per-pod epoch vectors + writer seq — to a single self-describing Option-A\nartifact WHILE SERVING, no stop-the-world); sparq-server mounts the gated\nPOST /admin/backup + POST /admin/restore admin routes and a --restore restore-on-start.\n\n- OPT-IN behind a `backup` cargo feature (sparq-serve and sparq-server), default OFF;\n  serving core fully buildable without it (the module + the ArcSwap-backed serving\n  core swap path are #[cfg]-stripped / never exercised — read path byte-identical).\n- Artifact = Stardog backup-ID model: textual header (generation/writer-seq, per-pod\n  epoch vectors, triple count, FNV-1a body digest) + full dataset as N-Quads. No new\n  dependency (N-Quads serialiser + loader already in the tree; self-contained digest).\n- Online export pins the current generation lock-free and serialises off the immutable\n  Arc — readers never block the writer, writer never blocks readers.\n- Restore atomically installs a freshly-built ring+writer (ArcSwap serving core);\n  fail-closed on a corrupt / truncated / version-mismatched / non-artifact body (live\n  store left untouched). In-memory only in v1 (--persist server → 409; beaded follow-up).\n- At-rest ENCRYPTION of the artifact is OUT OF SCOPE (a separate concern).\n- Distinct from offline `sparq-cli save` (stop-the-world, index rebuild) and the\n  --persist per-graph WAL.\n\nTests: sparq-serve unit round-trip (incl. epochs + writer-seq) + 5 fail-closed cases;\nsparq-server integration round-trip (backup -> restore into a fresh server, identical\nqueryable generation, post-restore writes), restore-on-start, fail-closed\n(corrupt + non-artifact), write-auth gating, --persist refusal. Gate GREEN in BOTH\nfeature states (clippy -D warnings + cargo test -p sparq-serve -p sparq-server).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(sparq-serve): demote private/unresolved intra-doc links in backup.rs to code spans (sq-o5bi) [OPUS-4.8]\n\nThe `clippy (gate)` job bundles the rustdoc `--all-features` lint\n(`cargo doc --workspace --no-deps --all-features` with RUSTDOCFLAGS=-D warnings),\nwhich failed `rustdoc::broken-intra-doc-links` on two `//!`/`///` links in the new\nbackup module:\n\n  - `[`Graph::load_dataset`]` — `Graph` (sparq_core) is not in intra-doc scope from\n    the module-level `//!` doc, so the associated-fn link is unresolved.\n  - `[`import`]` (twice) — unresolved from the doc context.\n\nBoth are decorative prose references, not navigation, so demote them to plain code\nspans (backticks) per the established repo fix for this lint class. No code change;\nthe `backup` feature behaviour is untouched.\n\nLocal gates (re-run green, both feature states):\n  - cargo clippy --workspace --all-targets --all-features -- -D warnings: clean\n  - RUSTDOCFLAGS=\"-D warnings\" cargo doc --workspace --no-deps --all-features: clean\n  - cargo test -p sparq-serve --features backup: 14+4 pass\n  - cargo test -p sparq-server --features backup --test backup: 6/6 pass\n  - cargo clippy -p sparq-serve -p sparq-server --all-targets (no-backup): clean\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* refactor(sparq-server): gate ArcSwap<ServingCore> behind `backup` so the default read path is byte-identical to pre-#941 (sq-0g6g) [OPUS-4.8]\n\nThe online backup/restore PR (#941, sq-o5bi) made `AppState.core` an\n`Arc<ArcSwap<ServingCore>>` UNCONDITIONALLY, adding one atomic load per\nread even when `backup` is OFF. sq-0g6g documents the design question;\nthis resolves it in the lean direction (opt-in-feature-architecture):\nthe `ArcSwap` swap mechanism — needed only for the atomic online restore —\nis now `backup`-gated.\n\n- DEFAULT (`backup` OFF): `AppState` holds `ring`/`writer` directly, the\n  exact pre-#941 representation. The read/write sites route through a pair\n  of `#[inline(always)]` accessors (`ring()`/`writer()`) that return a\n  zero-cost `&T` field borrow, so the default path compiles to the same\n  access pattern as `main` — no `ArcSwap::load` on the hot path.\n- `backup` ON: `AppState` holds `Arc<ArcSwap<ServingCore>>`; the accessors\n  clone the ring/writer `Arc` out of the loaded core; `restore_from` swaps\n  atomically. The 6 backup integration tests (atomic restore) still pass.\n- `dep:arc-swap` moves from the unconditional `server` feature to `backup`,\n  so the default `sparq-server` build no longer has arc-swap as a direct\n  dependency (matches `main`; it remains a transitive dep of sparq-serve\n  for the ring's own internal ArcSwap, unchanged).\n- Updated the doc-comments that claimed the ArcSwap was unconditional, and\n  the http-server SKILL backup note, to reflect the `backup`-gating.\n\nGates (both feature states): clippy --workspace --all-targets --all-features\nclean; clippy -p sparq-serve -p sparq-server (default) clean; rustdoc\n--all-features -D warnings clean; sparq-serve test {default, backup,\nresult-cache, backup+result-cache} pass; sparq-server --features backup\n--test backup (6 tests) pass; sparq-server default lib + protocol/updates/\nhardening + time-travel{,+backup} legs pass. typos + privacy gates clean.\n\nStill UNARMED — held under sq-0g6g for canonical perf validation of the\nopt-in backup-ON path (the default path is now byte-identical, so the\ndefault-path perf concern is resolved).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jeswrsolidserver@gmail.com>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\nCo-authored-by: Jesse Wright <jmwright.045@gmail.com>",
          "timestamp": "2026-06-20T13:33:59+01:00",
          "tree_id": "ac4e1553ee495a8a0fe94f2ec6c618ff6961cd8a",
          "url": "https://github.com/jeswr/sparq/commit/839014efb88b7517518d46a0c3a84b001e45e000"
        },
        "date": 1781960442521,
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
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3327.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4878.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 798.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13523.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 60993.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 168418.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 3490.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 44120.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8428.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 60915.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 162267,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 6594,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 41299.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29988.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 2020394.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6323.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3800.6,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3627.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7432.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 485869.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12944.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31786.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53398.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3782.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22435,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 140907.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 105793.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 186704.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36393.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6643.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13489.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 30017.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 2092074.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6101.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3785.7,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3650.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8767.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 489144.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 13119.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 33912.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 54692.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3821.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22685.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 21,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 144292.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 106354.8,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 177550.7,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 36692.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7320.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13216.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29876.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.4,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 2128115.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6376.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3932.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3719.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9611,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 483395.6,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13230.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 32223.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 54981.7,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3877.5,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22693.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 142703.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 105466.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 190219.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 36394.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6989.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13287,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 18.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 7266.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 17713.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16822.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16610.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 493084.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 18480,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 28016.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 314100.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 23776.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 26653.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 310320.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 11174.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 20244,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16733,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16825,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 547713.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 20623.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 25576.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 308543.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 24270.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 75.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 27442.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 309400.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 15895.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 21455,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16489.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16371.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 543307,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 22406.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24653.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 310899.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23516.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 131.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 26210.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 313326.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_count_us",
            "value": 60.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_count_us",
            "value": 31.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_count_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_count_us",
            "value": 101.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_count_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_count_us",
            "value": 18.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_count_us",
            "value": 32.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_count_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_count_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_count_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_materialize_us",
            "value": 959,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_materialize_us",
            "value": 26.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_materialize_us",
            "value": 27,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_materialize_us",
            "value": 120.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_materialize_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_materialize_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_materialize_us",
            "value": 14.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_materialize_us",
            "value": 107.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_materialize_us",
            "value": 32.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_materialize_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_materialize_us",
            "value": 15.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_materialize_us",
            "value": 23.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_json_us",
            "value": 1530.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_json_us",
            "value": 27.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_json_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_json_us",
            "value": 119.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_json_us",
            "value": 20.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_json_us",
            "value": 17.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_json_us",
            "value": 21.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_json_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_json_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_json_us",
            "value": 111.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_json_us",
            "value": 33.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_json_us",
            "value": 21.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_json_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_json_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 56.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 65.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 78.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 102.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 519.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 171.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 407.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 777.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 88.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 56.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 80.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 74.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 102.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 502.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 177.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 285.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 607.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 46.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 57.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 179.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 75.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 106.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 510.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 205.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 330,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 618.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 46,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 605.2,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.8,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 80.9,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 30.1,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2933.9,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4045,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 18.7,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22.6,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.072,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "sameas_size8_closure_s",
            "value": 0,
            "unit": "s"
          },
          {
            "name": "sameas_size8_query_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "sameas_size8_closure_triples",
            "value": 352,
            "unit": "triples"
          },
          {
            "name": "sameas_size32_closure_s",
            "value": 0.001,
            "unit": "s"
          },
          {
            "name": "sameas_size32_query_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "sameas_size32_closure_triples",
            "value": 4480,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 328.4,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 320.4,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13555,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6366.5,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 682213.2,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 138.8,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 85.8,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 107670.7,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 93.4,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3892.5,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.49778,
            "unit": "s"
          },
          {
            "name": "vectors_diskann_recall_at10",
            "value": 34,
            "unit": "milli"
          },
          {
            "name": "vectors_diskann_query_us",
            "value": 346.2,
            "unit": "us"
          },
          {
            "name": "vectors_hnsw_recall_at10",
            "value": 1,
            "unit": "milli"
          },
          {
            "name": "vectors_hnsw_query_us",
            "value": 488.3,
            "unit": "us"
          },
          {
            "name": "vectors_pq_recall_at10",
            "value": 22,
            "unit": "milli"
          },
          {
            "name": "vectors_pq_query_us",
            "value": 430.7,
            "unit": "us"
          },
          {
            "name": "vectors_build_s",
            "value": 41.213051,
            "unit": "s"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w0_rows",
            "value": 3,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_persistentdict_triples_per_s",
            "value": 2734492,
            "unit": "triples_per_s"
          },
          {
            "name": "snikmeta_triples",
            "value": 328,
            "unit": "count"
          },
          {
            "name": "snikmeta_terms",
            "value": 205,
            "unit": "count"
          },
          {
            "name": "snikmeta_distinct_predicates",
            "value": 23,
            "unit": "count"
          },
          {
            "name": "snikmeta_rdf_type_triples",
            "value": 49,
            "unit": "count"
          },
          {
            "name": "snikmeta_direct_eq_upstream",
            "value": 1,
            "unit": "count"
          },
          {
            "name": "hdt_load_s",
            "value": 0.042683,
            "unit": "s"
          },
          {
            "name": "hdt_vs_ntgz_load_s",
            "value": 3.4066,
            "unit": "ratio"
          },
          {
            "name": "zk_compose_filter_decimal_i3_f2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_gates",
            "value": 3113,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_hidden_issuer_d4_gates",
            "value": 16932,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_pok_gates",
            "value": 10334,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_set_d4_gates",
            "value": 10650,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb16_gates",
            "value": 7025,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb64_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb16_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb64_gates",
            "value": 18681,
            "unit": "gates"
          },
          {
            "name": "zk_compose_revoke_unset_d10_gates",
            "value": 899,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r4_gates",
            "value": 5991,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r8_gates",
            "value": 7038,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r4_gates",
            "value": 14923,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r8_gates",
            "value": 18850,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r4_gates",
            "value": 9254,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r8_gates",
            "value": 11261,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r4_gates",
            "value": 27054,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r8_gates",
            "value": 34821,
            "unit": "gates"
          },
          {
            "name": "zk_canon_bnode_1024_us",
            "value": 5919.026,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_256_us",
            "value": 1433.037,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_64_us",
            "value": 351.342,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_1024_us",
            "value": 4341.385,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_256_us",
            "value": 1050.687,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_64_us",
            "value": 245.972,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_1024_us",
            "value": 104062.309,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_256_us",
            "value": 26074.683,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_64_us",
            "value": 6458.41,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_1024_us",
            "value": 78775.68,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_256_us",
            "value": 19684.84,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_64_us",
            "value": 4904.507,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_1024_us",
            "value": 74013.355,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_256_us",
            "value": 18479.487,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_64_us",
            "value": 4588.032,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_hash40_us",
            "value": 202.867,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_permutation_us",
            "value": 14.293,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_traced_1000_us",
            "value": 5213.635,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_untraced_1000_us",
            "value": 1130.285,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_traced_1000_us",
            "value": 1770.976,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_untraced_1000_us",
            "value": 423.349,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_traced_1000_us",
            "value": 8590.658,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_untraced_1000_us",
            "value": 2550.364,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_traced_1000_us",
            "value": 610.985,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_untraced_1000_us",
            "value": 130.664,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_traced_1000_us",
            "value": 499.493,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_untraced_1000_us",
            "value": 80.103,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_traced_1000_us",
            "value": 1415.902,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_untraced_1000_us",
            "value": 264.911,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_traced_1000_us",
            "value": 4579.916,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_untraced_1000_us",
            "value": 1181.526,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_traced_1000_us",
            "value": 1362.745,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_untraced_1000_us",
            "value": 471.681,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_traced_100_us",
            "value": 331.731,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_untraced_100_us",
            "value": 114.282,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_traced_100_us",
            "value": 185.04,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_untraced_100_us",
            "value": 53.071,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_traced_100_us",
            "value": 658.599,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_untraced_100_us",
            "value": 169.955,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_traced_100_us",
            "value": 67.449,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_untraced_100_us",
            "value": 18.003,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_traced_100_us",
            "value": 53.776,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_untraced_100_us",
            "value": 11.062,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_traced_100_us",
            "value": 162.358,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_untraced_100_us",
            "value": 29.58,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_traced_100_us",
            "value": 315.233,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_untraced_100_us",
            "value": 122.205,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_traced_100_us",
            "value": 147.606,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_untraced_100_us",
            "value": 55.95,
            "unit": "us"
          },
          {
            "name": "solid_wac_named_graphs",
            "value": 1148,
            "unit": "count"
          },
          {
            "name": "solid_wac_quads",
            "value": 3060,
            "unit": "count"
          },
          {
            "name": "solid_wac_auth_triples",
            "value": 3783,
            "unit": "count"
          },
          {
            "name": "solid_acp_auth_triples",
            "value": 6355,
            "unit": "count"
          },
          {
            "name": "solid_alice_readable_graphs",
            "value": 800,
            "unit": "count"
          },
          {
            "name": "solid_full_dataset_rows",
            "value": 864,
            "unit": "count"
          },
          {
            "name": "solid_authorized_rows",
            "value": 599,
            "unit": "count"
          },
          {
            "name": "nlq_synth_triples",
            "value": 6000,
            "unit": "count"
          },
          {
            "name": "nlq_prompt_chars",
            "value": 1973,
            "unit": "chars"
          },
          {
            "name": "nlq_ask_repairs",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "nlq_ask_result_rows",
            "value": 2,
            "unit": "count"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.145,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1663825,
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
          "id": "bb4f88b7710e76cd53e7f70b4c0a716e46d8c04c",
          "message": "feat(site): /download page for GUI desktop + CLI/server binaries (sq-gl3cf) [OPUS-4.8] (#974)\n\nAdd a /download route linking the latest GitHub Release assets — the unsigned\n\"developer build\" desktop GUI installers (.dmg arm64/intel, .msi, .AppImage/.deb)\nplus the CLI/server binaries — with honest per-OS first-launch (Gatekeeper /\nSmartScreen / chmod) instructions and a no-release-yet fallback. Client-side OS\ndetection highlights the visitor's platform as progressive enhancement while\nalways listing every platform. Adds a discoverable \"Download\" top-bar nav tab.\n\nThe page is honest that the desktop bundles are NOT code-signed/notarized (signing\nis the separate needs:user bead sq-v286.8) and points only at the official\nReleases page. Asset links target the latest-release page (not per-asset deep\nlinks) because release.yml (sq-8n1c, #808) names each asset\n`sparq-gui-<tag>-<label>-<tauri-name>` with a dynamic tag — a static deep link\nwould be dead until a tag ships; gh release list is currently empty.\n\nRefs sq-gl3cf, sq-ixc3.\n\nCo-authored-by: Jesse Wright <jmwright.045@gmail.com>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-20T12:48:29Z",
          "tree_id": "d4096dd15f75b008a3c60667e3a91aa37318c527",
          "url": "https://github.com/jeswr/sparq/commit/bb4f88b7710e76cd53e7f70b4c0a716e46d8c04c"
        },
        "date": 1781961356453,
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
            "value": 3338.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4865.8,
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
            "value": 800.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13415.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 60861.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 164817.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4135.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 43173.3,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8246.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 59080.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 163788.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3133.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40565.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29731.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1519968.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6112,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3827.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3569.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7321.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 478529.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 16881.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31753.6,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53644.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3953,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22480.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 134461.2,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 103140,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 172701.7,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35694.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7764.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13279.4,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 5.2,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29641.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1532922.2,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6972.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3854.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3602.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8394.9,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 482839.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12685,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31325.1,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53952.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3773.5,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22140,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 138121.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 103168.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 172495,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 35735.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6844.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13061.6,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29855.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.8,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1542311.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6139.8,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3881.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3626.3,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9225.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 482724.6,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12829.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31289.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53222.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3892.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22380,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 142043.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 106300.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 175836.5,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 35660.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7131,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13051.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 7143.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16917.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16624.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16566.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 465657.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17801.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24801.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 301360.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 23342,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 25790.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 300662.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 10188.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 18689.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16423.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16272.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 513361.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 20988,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24884.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 306296.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 23350.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 66.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 25136,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 308065.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 14654.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20948.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16649.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16494.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 515340.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 20684.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24951.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 304941.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23423.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 132,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 24020.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 303581.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_count_us",
            "value": 62.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_count_us",
            "value": 31.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_count_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_count_us",
            "value": 99.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_count_us",
            "value": 18.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_count_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_count_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_count_us",
            "value": 33.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_count_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_count_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_materialize_us",
            "value": 924.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_materialize_us",
            "value": 26.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_materialize_us",
            "value": 26.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_materialize_us",
            "value": 116.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_materialize_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_materialize_us",
            "value": 16.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_materialize_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_materialize_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_materialize_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_materialize_us",
            "value": 110.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_materialize_us",
            "value": 31.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_materialize_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_materialize_us",
            "value": 14.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_materialize_us",
            "value": 23.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_materialize_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_json_us",
            "value": 1568.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_json_us",
            "value": 27.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_json_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_json_us",
            "value": 125.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_json_us",
            "value": 20.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_json_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_json_us",
            "value": 21.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_json_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_json_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_json_us",
            "value": 125.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_json_us",
            "value": 33.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_json_us",
            "value": 21.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_json_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_json_us",
            "value": 27.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_json_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 53.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 68.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 73.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 104.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 494.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 179.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 294.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 625.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 46.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 55.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 81.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 77.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 104.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 497.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 181.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 288.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 605,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 48.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 58.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 158.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 76.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 108.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 499.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 199,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 331.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 615.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 46.4,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 596,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.8,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 67,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 38.6,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2909.1,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4089.5,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 23.7,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.064,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "sameas_size8_closure_s",
            "value": 0,
            "unit": "s"
          },
          {
            "name": "sameas_size8_query_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "sameas_size8_closure_triples",
            "value": 352,
            "unit": "triples"
          },
          {
            "name": "sameas_size32_closure_s",
            "value": 0.001,
            "unit": "s"
          },
          {
            "name": "sameas_size32_query_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "sameas_size32_closure_triples",
            "value": 4480,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 334.3,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 314.6,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13482.2,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6473.2,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 652117.8,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 171.1,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 85.7,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 105794.1,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 96.3,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.6,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3909.5,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.496972,
            "unit": "s"
          },
          {
            "name": "vectors_diskann_recall_at10",
            "value": 34,
            "unit": "milli"
          },
          {
            "name": "vectors_diskann_query_us",
            "value": 346.6,
            "unit": "us"
          },
          {
            "name": "vectors_hnsw_recall_at10",
            "value": 2,
            "unit": "milli"
          },
          {
            "name": "vectors_hnsw_query_us",
            "value": 463.6,
            "unit": "us"
          },
          {
            "name": "vectors_pq_recall_at10",
            "value": 22,
            "unit": "milli"
          },
          {
            "name": "vectors_pq_query_us",
            "value": 424,
            "unit": "us"
          },
          {
            "name": "vectors_build_s",
            "value": 41.284967,
            "unit": "s"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w0_rows",
            "value": 3,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_persistentdict_triples_per_s",
            "value": 2717748,
            "unit": "triples_per_s"
          },
          {
            "name": "snikmeta_triples",
            "value": 328,
            "unit": "count"
          },
          {
            "name": "snikmeta_terms",
            "value": 205,
            "unit": "count"
          },
          {
            "name": "snikmeta_distinct_predicates",
            "value": 23,
            "unit": "count"
          },
          {
            "name": "snikmeta_rdf_type_triples",
            "value": 49,
            "unit": "count"
          },
          {
            "name": "snikmeta_direct_eq_upstream",
            "value": 1,
            "unit": "count"
          },
          {
            "name": "hdt_load_s",
            "value": 0.043155,
            "unit": "s"
          },
          {
            "name": "hdt_vs_ntgz_load_s",
            "value": 3.4212,
            "unit": "ratio"
          },
          {
            "name": "zk_compose_filter_decimal_i3_f2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_gates",
            "value": 3113,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_hidden_issuer_d4_gates",
            "value": 16932,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_pok_gates",
            "value": 10334,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_set_d4_gates",
            "value": 10650,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb16_gates",
            "value": 7025,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb64_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb16_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb64_gates",
            "value": 18681,
            "unit": "gates"
          },
          {
            "name": "zk_compose_revoke_unset_d10_gates",
            "value": 899,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r4_gates",
            "value": 5991,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r8_gates",
            "value": 7038,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r4_gates",
            "value": 14923,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r8_gates",
            "value": 18850,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r4_gates",
            "value": 9254,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r8_gates",
            "value": 11261,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r4_gates",
            "value": 27054,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r8_gates",
            "value": 34821,
            "unit": "gates"
          },
          {
            "name": "zk_canon_bnode_1024_us",
            "value": 5955.466,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_256_us",
            "value": 1443.934,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_64_us",
            "value": 344.743,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_1024_us",
            "value": 4408.75,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_256_us",
            "value": 1031.756,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_64_us",
            "value": 247.482,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_1024_us",
            "value": 104010.709,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_256_us",
            "value": 26048.462,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_64_us",
            "value": 6461.947,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_1024_us",
            "value": 78736.824,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_256_us",
            "value": 19774.607,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_64_us",
            "value": 4906.563,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_1024_us",
            "value": 74000.89,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_256_us",
            "value": 18485.033,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_64_us",
            "value": 4582.123,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_hash40_us",
            "value": 202.717,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_permutation_us",
            "value": 14.493,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_traced_1000_us",
            "value": 5248.553,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_untraced_1000_us",
            "value": 1137.868,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_traced_1000_us",
            "value": 1777.026,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_untraced_1000_us",
            "value": 424.332,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_traced_1000_us",
            "value": 8745.674,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_untraced_1000_us",
            "value": 2572.533,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_traced_1000_us",
            "value": 611.92,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_untraced_1000_us",
            "value": 129.54,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_traced_1000_us",
            "value": 505.217,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_untraced_1000_us",
            "value": 79.877,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_traced_1000_us",
            "value": 1409.018,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_untraced_1000_us",
            "value": 265.88,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_traced_1000_us",
            "value": 4728.9,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_untraced_1000_us",
            "value": 1097.771,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_traced_1000_us",
            "value": 1369.451,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_untraced_1000_us",
            "value": 440.675,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_traced_100_us",
            "value": 344.332,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_untraced_100_us",
            "value": 114.93,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_traced_100_us",
            "value": 186.113,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_untraced_100_us",
            "value": 52.744,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_traced_100_us",
            "value": 662.829,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_untraced_100_us",
            "value": 171.666,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_traced_100_us",
            "value": 68.248,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_untraced_100_us",
            "value": 17.744,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_traced_100_us",
            "value": 54.563,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_untraced_100_us",
            "value": 10.805,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_traced_100_us",
            "value": 163.53,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_untraced_100_us",
            "value": 30.288,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_traced_100_us",
            "value": 326.071,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_untraced_100_us",
            "value": 113.701,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_traced_100_us",
            "value": 146.996,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_untraced_100_us",
            "value": 53.279,
            "unit": "us"
          },
          {
            "name": "solid_wac_named_graphs",
            "value": 1148,
            "unit": "count"
          },
          {
            "name": "solid_wac_quads",
            "value": 3060,
            "unit": "count"
          },
          {
            "name": "solid_wac_auth_triples",
            "value": 3783,
            "unit": "count"
          },
          {
            "name": "solid_acp_auth_triples",
            "value": 6355,
            "unit": "count"
          },
          {
            "name": "solid_alice_readable_graphs",
            "value": 800,
            "unit": "count"
          },
          {
            "name": "solid_full_dataset_rows",
            "value": 864,
            "unit": "count"
          },
          {
            "name": "solid_authorized_rows",
            "value": 599,
            "unit": "count"
          },
          {
            "name": "nlq_synth_triples",
            "value": 6000,
            "unit": "count"
          },
          {
            "name": "nlq_prompt_chars",
            "value": 1973,
            "unit": "chars"
          },
          {
            "name": "nlq_ask_repairs",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "nlq_ask_result_rows",
            "value": 2,
            "unit": "count"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.145,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1663825,
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
          "id": "938ec5ab92d92d0f70080755843901fae6943622",
          "message": "feat(site): /try playground JSON-LD output modes — wire wasm Store.serializeCompact full 1.1 Compaction (sq-oy1f.7) [OPUS-4.8] (#976)\n\nThe /try REPL's CONSTRUCT / DESCRIBE result graph previously rendered only as\npretty Turtle / raw N-Triples. This adds a JSON-LD OUTPUT-FORMAT selector to the\ngraph result, serialising the SAME result triples through the wasm engine's own\nJSON-LD writer (never a TS reshaper):\n\n- \"JSON-LD (expanded)\" / \"(flattened)\" / \"(prefixes)\" drive `Store.serialize`'s\n  JSON-LD document forms (#900/#923).\n- \"JSON-LD (Compaction)\" drives `Store.serializeCompact` — the full W3C JSON-LD\n  1.1 Compaction Algorithm against a USER-SUPPLIED `@context` textarea (term\n  definitions, @vocab, type/language/@container coercion, @reverse, @id/@type\n  aliasing). This is sq-oy1f.7's specific ask (the consumer for sq-oy1f.5, #957).\n\nThe wasm `Store.serializeCompact(context, pretty, indent?)` binding (sq-oy1f.5,\n#957) and `Store.serialize` (#900/#923) were ALREADY exposed in the site's\n`build:wasm` bundle (`--features shacl,jsonld,serialize-rdf,scs`) — no crates\nchange was needed; the generated d.ts already carries both (check:wasm-types\nbyte-clean). sq-oy1f.3's REPL JSON-LD-output UI had not landed, so this PR also\nlands that broader capability (the expanded/flattened/prefix modes) as the\nfoundation the full-Compaction mode sits on.\n\nImplementation (site only):\n- `serializeGraphAsJsonLd(ntriples, mode, context?)` in src/lib/sparq-wasm.ts —\n  loads the result graph into an EPHEMERAL wasm store and serialises it in the\n  chosen JSON-LD form; the `\"compact\"` mode routes to `serializeCompact`. The\n  binding rejects a non-object `@context` with a clear error (surfaced inline).\n- `GraphResult` / `GraphFormatTabs` in repl.tsx — the format selector (same\n  role=\"tablist\" token pattern as ModeTabs / ResultViewTabs), the `@context`\n  textarea for the full-Compaction mode, lazy + memoised serialise on switch\n  (off the query path; a stale-result guard prevents out-of-order renders),\n  rendering via the read-only `JsonLdHighlight`.\n- e2e: a CONSTRUCT → JSON-LD (expanded) → JSON-LD (Compaction) walk in\n  repl-results.spec.ts, asserting each form renders a real JSON-LD document with\n  zero console errors (anchored on data-result-kind=\"graph\" /\n  data-graph-view=\"jsonld\"). Skips when the wasm bundle is absent (light CI lane).\n\nGates: Pages static export + Tauri build green (/try emitted), lint clean,\ntsc --noEmit clean, check:wasm-types byte-identical, e2e 3/3 pass.\n\nRefs sq-oy1f.7 (epic sq-oy1f), sq-oy1f.3.\n\nCo-authored-by: Jesse Wright <jmwright.045@gmail.com>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-20T13:04:05Z",
          "tree_id": "23f34b1bb1b645286f52ebe32690f84384395a7a",
          "url": "https://github.com/jeswr/sparq/commit/938ec5ab92d92d0f70080755843901fae6943622"
        },
        "date": 1781962679622,
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
            "value": 3356.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4902.1,
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
            "value": 795.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13169,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59945.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 169341.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4814.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 43941.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8161.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 60592.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 161248.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 4105.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39959.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 30034.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 16.7,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1539256.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6175.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3893.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3630.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7309.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 483543.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 13216.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31788.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53493.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3843.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 23520.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 129465.7,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 103290.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 174723.7,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35998.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6983.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13433.6,
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
            "value": 29733,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1623622.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6273.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3826.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3597.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8532,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 482120.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 13259.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31554.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53037,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3844.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22382.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.6,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 136283.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 102707.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 172225.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 35973.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7734,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13227.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29665,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1594231.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6202.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3921.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3689.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9694.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 482697.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13185.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 31761.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 55321.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3860.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22044.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 136929,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 105747.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 184313.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 35540.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7061.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13130,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 7114.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16756.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16383,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16425.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 468240.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17995.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24721.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 308173.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 23622.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 23826,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 306618,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 10153.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19247,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16515.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16569.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 507132.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 20507.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24687.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 304829.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 23038.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 63.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 24386.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 306780.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 14426.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20409.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16683.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16726.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 513767.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 20725.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24688.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 304956,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23405.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 130.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 24767.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 303794.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_count_us",
            "value": 65.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_count_us",
            "value": 33.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_count_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_count_us",
            "value": 118.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_count_us",
            "value": 18.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_count_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_count_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_count_us",
            "value": 32.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_count_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_count_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_count_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_materialize_us",
            "value": 939.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_materialize_us",
            "value": 25.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_materialize_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_materialize_us",
            "value": 104.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_materialize_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_materialize_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_materialize_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_materialize_us",
            "value": 121.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_materialize_us",
            "value": 31.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_materialize_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_materialize_us",
            "value": 14.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_materialize_us",
            "value": 23.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_C3_json_us",
            "value": 1548.1,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F2_json_us",
            "value": 27.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F3_json_us",
            "value": 27.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_F5_json_us",
            "value": 119.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L1_json_us",
            "value": 20.6,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L2_json_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L3_json_us",
            "value": 21.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L4_json_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_L5_json_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S1_json_us",
            "value": 111.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S2_json_us",
            "value": 33.2,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S3_json_us",
            "value": 21.4,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S4_json_us",
            "value": 16.9,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S5_json_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S6_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_sf1_S7_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 64,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 66.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 78,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 107.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 494.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 171.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 286.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 610.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 46,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 57.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 81.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 85.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 107.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 509.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 181.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 282.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 597.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 47.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 58.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 162.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 82.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 106.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 504,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 197.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 364.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 607.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 48.4,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 601.2,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.9,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 65.5,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 29.5,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.5,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2950.2,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4049.1,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 22.6,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.063,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "sameas_size8_closure_s",
            "value": 0,
            "unit": "s"
          },
          {
            "name": "sameas_size8_query_us",
            "value": 3.5,
            "unit": "us"
          },
          {
            "name": "sameas_size8_closure_triples",
            "value": 352,
            "unit": "triples"
          },
          {
            "name": "sameas_size32_closure_s",
            "value": 0.001,
            "unit": "s"
          },
          {
            "name": "sameas_size32_query_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "sameas_size32_closure_triples",
            "value": 4480,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 331.3,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 313.7,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13889.4,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6390.5,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 679268.2,
            "unit": "us"
          },
          {
            "name": "geo_within10km_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "geo_within50km_us",
            "value": 136.6,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k10_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "geo_nearest_k100_us",
            "value": 82.2,
            "unit": "us"
          },
          {
            "name": "geo_geof_within_us",
            "value": 111715.3,
            "unit": "us"
          },
          {
            "name": "geo_geo_compliance_pass_us",
            "value": 97,
            "unit": "us"
          },
          {
            "name": "geo_compliance_deficit",
            "value": 0,
            "unit": "fixtures"
          },
          {
            "name": "text_and_terms_us",
            "value": 13.4,
            "unit": "us"
          },
          {
            "name": "text_near_slop2_us",
            "value": 1.4,
            "unit": "us"
          },
          {
            "name": "text_or_terms_us",
            "value": 22.3,
            "unit": "us"
          },
          {
            "name": "text_phrase_us",
            "value": 1.3,
            "unit": "us"
          },
          {
            "name": "text_prefix4_us",
            "value": 3874,
            "unit": "us"
          },
          {
            "name": "fts_bytes_per_doc",
            "value": 371,
            "unit": "bytes"
          },
          {
            "name": "text_build_s",
            "value": 0.499321,
            "unit": "s"
          },
          {
            "name": "vectors_diskann_recall_at10",
            "value": 34,
            "unit": "milli"
          },
          {
            "name": "vectors_diskann_query_us",
            "value": 345.4,
            "unit": "us"
          },
          {
            "name": "vectors_hnsw_recall_at10",
            "value": 2,
            "unit": "milli"
          },
          {
            "name": "vectors_hnsw_query_us",
            "value": 453.9,
            "unit": "us"
          },
          {
            "name": "vectors_pq_recall_at10",
            "value": 22,
            "unit": "milli"
          },
          {
            "name": "vectors_pq_query_us",
            "value": 426.7,
            "unit": "us"
          },
          {
            "name": "vectors_build_s",
            "value": 41.060833,
            "unit": "s"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w0_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w2_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w3_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_avg_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_rebuild_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_pdict_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w1_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w3_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_sliding_sum_delta_w4_rows",
            "value": 1,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_rebuild_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_pdict_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_tumbling_groupby_join_delta_w2_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w0_rows",
            "value": 3,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_join_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w0_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w1_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w2_rows",
            "value": 2,
            "unit": "rows"
          },
          {
            "name": "rsp_srbench_groupby_state_w3_rows",
            "value": 0,
            "unit": "rows"
          },
          {
            "name": "rsp_persistentdict_triples_per_s",
            "value": 2703868,
            "unit": "triples_per_s"
          },
          {
            "name": "snikmeta_triples",
            "value": 328,
            "unit": "count"
          },
          {
            "name": "snikmeta_terms",
            "value": 205,
            "unit": "count"
          },
          {
            "name": "snikmeta_distinct_predicates",
            "value": 23,
            "unit": "count"
          },
          {
            "name": "snikmeta_rdf_type_triples",
            "value": 49,
            "unit": "count"
          },
          {
            "name": "snikmeta_direct_eq_upstream",
            "value": 1,
            "unit": "count"
          },
          {
            "name": "hdt_load_s",
            "value": 0.043425,
            "unit": "s"
          },
          {
            "name": "hdt_vs_ntgz_load_s",
            "value": 3.3783,
            "unit": "ratio"
          },
          {
            "name": "zk_compose_filter_decimal_i3_f2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_gates",
            "value": 3113,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_f64_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d1_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d3_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d2_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_filter_signed_int_d4_gates",
            "value": 17416,
            "unit": "gates"
          },
          {
            "name": "zk_compose_hidden_issuer_d4_gates",
            "value": 16932,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_pok_gates",
            "value": 10334,
            "unit": "gates"
          },
          {
            "name": "zk_compose_holder_set_d4_gates",
            "value": 10650,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb16_gates",
            "value": 7025,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na16_nb64_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb16_gates",
            "value": 12885,
            "unit": "gates"
          },
          {
            "name": "zk_compose_join_eq_na64_nb64_gates",
            "value": 18681,
            "unit": "gates"
          },
          {
            "name": "zk_compose_revoke_unset_d10_gates",
            "value": 899,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r4_gates",
            "value": 5991,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n16_r8_gates",
            "value": 7038,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r4_gates",
            "value": 14923,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k1_n64_r8_gates",
            "value": 18850,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r4_gates",
            "value": 9254,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n16_r8_gates",
            "value": 11261,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r4_gates",
            "value": 27054,
            "unit": "gates"
          },
          {
            "name": "zk_compose_scan_k2_n64_r8_gates",
            "value": 34821,
            "unit": "gates"
          },
          {
            "name": "zk_canon_bnode_1024_us",
            "value": 5944.102,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_256_us",
            "value": 1491.716,
            "unit": "us"
          },
          {
            "name": "zk_canon_bnode_64_us",
            "value": 347.197,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_1024_us",
            "value": 4409.913,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_256_us",
            "value": 1043.39,
            "unit": "us"
          },
          {
            "name": "zk_canon_iri_64_us",
            "value": 246.493,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_1024_us",
            "value": 104040.339,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_256_us",
            "value": 25980.006,
            "unit": "us"
          },
          {
            "name": "zk_commit_bnode_64_us",
            "value": 6477.378,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_1024_us",
            "value": 78778.328,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_256_us",
            "value": 19701.942,
            "unit": "us"
          },
          {
            "name": "zk_commit_iri_64_us",
            "value": 4919.899,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_1024_us",
            "value": 74673.774,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_256_us",
            "value": 18491.544,
            "unit": "us"
          },
          {
            "name": "zk_commit_leaves_fold_64_us",
            "value": 4585.608,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_hash40_us",
            "value": 202.977,
            "unit": "us"
          },
          {
            "name": "zk_poseidon2_permutation_us",
            "value": 14.316,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_traced_1000_us",
            "value": 5226.876,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_chain_untraced_1000_us",
            "value": 1134.82,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_traced_1000_us",
            "value": 1762.008,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_star_untraced_1000_us",
            "value": 423.252,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_traced_1000_us",
            "value": 8701.49,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_bgp_triangle_untraced_1000_us",
            "value": 2565.016,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_traced_1000_us",
            "value": 607.465,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_count_untraced_1000_us",
            "value": 130.083,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_traced_1000_us",
            "value": 500.701,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_distinct_untraced_1000_us",
            "value": 81.036,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_traced_1000_us",
            "value": 1396.232,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_filter_untraced_1000_us",
            "value": 264.244,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_traced_1000_us",
            "value": 3202.765,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_optional_untraced_1000_us",
            "value": 1095.002,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_traced_1000_us",
            "value": 1399.261,
            "unit": "us"
          },
          {
            "name": "zk_trace_1000entities_union_untraced_1000_us",
            "value": 434.265,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_traced_100_us",
            "value": 331.672,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_chain_untraced_100_us",
            "value": 115.779,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_traced_100_us",
            "value": 190.27,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_star_untraced_100_us",
            "value": 52.543,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_traced_100_us",
            "value": 665.395,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_bgp_triangle_untraced_100_us",
            "value": 171.906,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_traced_100_us",
            "value": 67.266,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_count_untraced_100_us",
            "value": 17.728,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_traced_100_us",
            "value": 54.397,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_distinct_untraced_100_us",
            "value": 10.994,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_traced_100_us",
            "value": 162.635,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_filter_untraced_100_us",
            "value": 30.019,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_traced_100_us",
            "value": 321.274,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_optional_untraced_100_us",
            "value": 113.725,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_traced_100_us",
            "value": 146.08,
            "unit": "us"
          },
          {
            "name": "zk_trace_100entities_union_untraced_100_us",
            "value": 52.036,
            "unit": "us"
          },
          {
            "name": "solid_wac_named_graphs",
            "value": 1148,
            "unit": "count"
          },
          {
            "name": "solid_wac_quads",
            "value": 3060,
            "unit": "count"
          },
          {
            "name": "solid_wac_auth_triples",
            "value": 3783,
            "unit": "count"
          },
          {
            "name": "solid_acp_auth_triples",
            "value": 6355,
            "unit": "count"
          },
          {
            "name": "solid_alice_readable_graphs",
            "value": 800,
            "unit": "count"
          },
          {
            "name": "solid_full_dataset_rows",
            "value": 864,
            "unit": "count"
          },
          {
            "name": "solid_authorized_rows",
            "value": 599,
            "unit": "count"
          },
          {
            "name": "nlq_synth_triples",
            "value": 6000,
            "unit": "count"
          },
          {
            "name": "nlq_prompt_chars",
            "value": 1973,
            "unit": "chars"
          },
          {
            "name": "nlq_ask_repairs",
            "value": 0,
            "unit": "count"
          },
          {
            "name": "nlq_ask_result_rows",
            "value": 2,
            "unit": "count"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.146,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1663825,
            "unit": "bytes"
          }
        ]
      }
    ]
  }
}