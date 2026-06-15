window.BENCHMARK_DATA = {
  "lastUpdate": 1781555500322,
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
          "id": "a814261c2d17d8830bc75390ed79f22ba05db006",
          "message": "test(sparq-solid): DENIED update leaves store byte-identical (fail-closed-before-apply) [OPUS-4.8] (#175)\n\n* test(sparq-solid): DENIED update leaves store byte-identical (fail-closed-before-apply) [OPUS-4.8]\n\nsq-3jtd.2 (parent sq-3jtd, from PSS). Adds regression tests asserting a\nDENIED SPARQL Update mutates NOTHING — strengthening the existing per-graph\ncount checks to a WHOLE-STORE canonical-equality snapshot (default + every\nnamed graph, sorted S/P/O).\n\nKey case: a ';'-separated multi-operation body where ONE op is unauthorized\n(authorized INSERT into team2 + unauthorized INSERT/DELETE into priv0) — the\nWHOLE body is refused and the authorized op does NOT partially apply. Plus a\npositive control: the same multi-op shape with both targets authorized DOES\napply, proving the deny tests exercise a real allow/deny boundary.\n\nInvariant HELD: update_inner runs update::check over the whole parsed update\nand returns Err BEFORE ever calling sparq_engine::update_in_place, so denial is\nstructurally fail-closed-before-apply. All 40 sparq-solid tests pass; clippy\nclean; added lines fmt-clean under rustfmt.toml (max_width=100).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* test(solid): use sort_unstable for snapshot equality ordering [OPUS-4.8]\n\nThe per-graph triple snapshot and overall store snapshot are only compared\nfor equality, so a deterministic (not stable) order suffices. sort_unstable\nis faster for large snapshots. Addresses PR #175 review nits.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T15:22:37Z",
          "tree_id": "5f934f404855ddbfe65016a038cd616cebb4e510",
          "url": "https://github.com/jeswr/sparq/commit/a814261c2d17d8830bc75390ed79f22ba05db006"
        },
        "date": 1781537163591,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.33,
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
            "value": 3.0449,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 2.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 1603.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 3104.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 426.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 9232.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 45219.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 107884.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 1357.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 2.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 31573.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 4603.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 38745.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 98824.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3218.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 25207.8,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 2.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 16403.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1263779.1,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 3356.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 1925.8,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 1827.7,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 3634.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 257498.7,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 8878.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 16889.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 30984.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 2180.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 12499.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 93336,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 64432.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 103544.4,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 18734.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 3684.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 7301.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 16379.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1271410.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 3390.1,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 1969.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 1857.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 4373.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 255878.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 6466.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 17136.8,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 31162.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 2011.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 13407.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 96796.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 65885.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 107202.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 3.7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 19007.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 3539.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 7139.6,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 2.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 17466.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1300137.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 3410,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 1964.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 1931.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 5279.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 259190.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 6988.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 17397.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 32851,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 2194,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 14474.8,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 105733,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 67028.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 103636.5,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 18970.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 4021.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 7425.4,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 3553.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 9260.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 8876.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 8777.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 259472.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 10441.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 14821.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 171293.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 15020.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 3.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 14169.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 175455,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 5131.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 9995.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 8989.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 8894.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 294353.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 10304.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 14162.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 163244.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 14111.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 43.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 12793.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 160414,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 7808.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 10781.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 8898.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 8792.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 281365.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 10740.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 13683.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 160669,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 12752.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 64.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 12730.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 161859.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 33.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 19,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 58.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 3.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 944.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 14.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 74.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 152.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 23.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 16.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 784.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 15.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 69,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 61.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 31.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 39,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 41.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 55.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 257.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 94,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 152,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 337.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 26.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 31.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 43.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 40.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 60.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 259,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 98.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 155.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 339.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 25.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 34.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 87.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 41.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 57.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 256.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 108.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 182.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 332.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 328.4,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 2.7,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 41.3,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 1585.5,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 2371.1,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.004,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 2.4,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.041,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.093,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1588686,
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
          "id": "9603de6d5fcdc660b323c7d9e4c775189be6ded7",
          "message": "feat(mpc): bounded property-path over disclosed-key regime (sq-py8h.1) (#174)\n\n* feat(mpc): bounded property-path over disclosed-key regime (sq-py8h.1)\n\n[OPUS-4.8] Increment #1 of the bounded property-path design\n(research/mpc-bounded-property-path-design.md §6 step 1, §2.1 DISCLOSED\nregime): evaluate a bounded path `?a (p){m,k} ?b` whose endpoints AND\nintermediate join keys are disclosed global IRIs.\n\nConstruction (crypto-free): unroll the bounded path statically on the\nPUBLIC bound k into a finite set of fixed-length BGP chains; evaluate each\nexactly-`ℓ` chain as a left-to-right fold of the existing DisclosedKeyJoin\nthrough fresh intermediate vars (the §2.1 core — the same shape as the\njoin crate's differential_three_holder_chain_equals_union test); UNION the\nchains over `ℓ in m..=k`; add the length-0 reflexive identity pairs for\n`{0,k}`; DEDUP the endpoint pairs to a set (the realized length is never\ndisclosed). Alternation expands each hop over its alternatives. All of it\nruns OUTSIDE the cryptographic core — NO secret sharing, NO MPC round.\n\nSupported forms: sequence p1/p2/.../pk; exact {k}; range {m,k}; reflexive\n{0,k} (and the {0,1} = p? special case); per-hop alternation (p1|p2|...).\n\nRegime boundary (stated, not hidden): DISCLOSED-KEY ONLY. The HIDDEN-\nintermediate regime (secret ?z_i kept secret-shared via secure_equal /\ndegree_reduce / oblivious_set_output) is the separate cryptographic\ndeliverable sq-py8h.2, which this bead blocks. Disclosed-key support is\nNOT hidden-path support.\n\nSemantic boundary: implements the bounded {m,k} form exactly (a SPARQL 1.1\nconstruct), NOT the unbounded +/* closure — a pair connected only by a\nchain longer than k is correctly absent.\n\nDifferential tests (13, all passing): the federated disclosed-key unroll\n== the clear-text engine's eval_path over the UNION store, for a plain\nsequence, exact {k}, range {m,k} (incl. min=2 excluding length-1),\nreflexive {0,k} (identity pairs asserted exactly once), p? ({0,1}), and\nalternation (exact-2 + range). Plus multi-length dedup, a crypto-free\nround-count==0 assertion (CommCounter untouched — nothing in the call\ngraph can record a round), and Protocol-error soundness cases.\n\nRefs sq-py8h.1, parent sq-py8h, design doc §6 step 1.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(sparq-mpc): address Copilot review on bounded property-path (sq-py8h.1)\n\n[OPUS-4.8] Resolve 5 Copilot review threads on PR #174 (disclosed-key\nbounded property-path unroll). All crypto-free; the 13 differential tests +\ncrypto-free (mult_rounds==0) assertion still pass; 2 new differential tests added.\n\n- DisclosedEdges::from_holder_edges now REJECTS non-IRI endpoints (blank nodes\n  / literals) and rows of width != 2 at ingest. Soundness: blank-node labels are\n  document-scoped, so a disclosed-key equi-join treating equal labels across\n  holders as equal terms would be a false cross-holder join; literals are out of\n  the global-IRI key regime. Fail fast with Protocol error (design §2.1 DISCLOSED).\n- project_endpoints now FAILS FAST (Result<_, MpcError::Protocol>) when an\n  endpoint var is missing from the schema instead of silently returning an empty\n  relation — a missing endpoint is an internal unroll-invariant violation, not a\n  \"no results\" condition; masking it would undermine soundness reasoning.\n- dedup_endpoint_pairs computes each row's canonical key ONCE (O(n) allocations)\n  and sorts on the precomputed key, instead of re-format!-ing both operands on\n  every comparison (O(n log n) allocations).\n- alternation_chains drops the explosive `chains.len() * alternatives.len()`\n  with_capacity (== alternatives^length, which can overflow usize / request a\n  huge allocation → panic on a public API surface); grows the Vec amortised.\n- Added 2 differential tests for PathForm::Sequence with a bounded\n  repetition / alternation-repetition element (the per-part-dedup branch of\n  eval_sequence): federated unroll == clear-text eval_path, mid multiplicity\n  deduped to single endpoint pairs.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(mpc): resolve Copilot threads on bounded-path — unroll guard, factual {m,n} doc, fail-fast endpoints, non-vacuous crypto-free test [OPUS-4.8]\n\nsq-py8h.1 — substantive fixes for the 4 unresolved Copilot review threads on\nPR #174 (disclosed-key bounded property path).\n\n1. Unroll-size DoS guard. `(step){m,k}` over an a-way alternation unrolls to\n   Σ_{ℓ=m..=k} a^ℓ fixed chains — exponential in the PUBLIC bound k. Added a\n   documented `MAX_UNROLL_CHAINS` cap (2^20) and a closed-form, checked-arithmetic\n   `projected_chain_count` that `eval_repetition` evaluates BEFORE any allocation,\n   returning `MpcError::Protocol` for an over-cap path (no panic/OOM/overflow).\n   New tests: over-cap rejection + projected-count closed-form/overflow.\n\n2. Factual {m,n} doc fix. The module doc claimed \"SPARQL 1.1 dropped the {m,n}\n   quantifier\". Corrected to the accurate history: the {n}/{m,n} counting\n   quantifiers were in the SPARQL 1.1 working DRAFTS but removed before the final\n   W3C Recommendation (no consensus on counting semantics); the final Rec carries\n   only */+/? (engine's eval_path has only ZeroOrMore/OneOrMore/ZeroOrOne, no\n   {m,n} variant), and sparq supports {m,k} as a bounded extension.\n\n3. Fail-fast endpoints. `pair_set` no longer masks an unbound endpoint as \"\" via\n   `unwrap_or_default()` (which hid bugs and risked false set collisions); it now\n   `expect`s both endpoint columns bound with a clear message.\n\n4. Non-vacuous crypto-free test. `CommCounter` is a standalone bench-harness\n   object not threaded through `eval_bounded_path_disclosed`, so the old\n   `assert_eq!(counter.mult_rounds, 0)` on an untouched counter was VACUOUS (0\n   regardless). Rewrote with a NEGATIVE CONTROL: first drive `record_mult`/\n   `record_open` (the same APIs the secure path uses) into a witness counter and\n   assert it goes non-zero — proving the `== 0` claim is falsifiable — then assert\n   a separate counter stays 0 across a full evaluation of every bounded form. The\n   crypto-free property is now genuinely proven, not assumed.\n\ncargo build + nextest (235 passed) + clippy -D warnings all green.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T15:25:43Z",
          "tree_id": "ee256b481b7f5b1236558c9560e65a6e8e64e531",
          "url": "https://github.com/jeswr/sparq/commit/9603de6d5fcdc660b323c7d9e4c775189be6ded7"
        },
        "date": 1781537365291,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.555,
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
            "value": 3.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3089,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4446.9,
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
            "value": 751.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12304,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 56457.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 150236.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4598.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 40711.8,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8943.9,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 60911.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 161760.5,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3254.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 40335.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29440.1,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1666784.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6404.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3782.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3490.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7348.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 511493.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12592.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 32978.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 54061.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3645.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22099.9,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 140164.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 96089.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 170298.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.6,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35585.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6520.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12920.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29442.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1844998.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6503.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3737.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3393.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9068.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 512530.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12491.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 32239.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 54397.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3686.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21838,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 139768.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 96898.1,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 171602.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 34531.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6746.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13172.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29805.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1870070.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6473.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3951.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3445.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9202.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 510702.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12472.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 32235.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53821.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3880.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21840.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 140793.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 95577.8,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 169619.4,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 34636.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6888,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12803.6,
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
            "value": 6310.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15425.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 15113.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14793.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 437706.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15460.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22031.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 286379.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 20888.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22140.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 282886.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9157.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 17571.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14908.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14738.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 472707.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16117.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22132.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 284829.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20348.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 68.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22803.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 284628.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13773.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20406.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 15249,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14926.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 480874.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16762.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22291.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 285658.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20252.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 144.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22637,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 284620.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 62.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 103.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18.2,
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
            "value": 38.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 15.3,
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
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 885.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 27.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 109,
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
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 126.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 30.6,
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
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1553,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 29.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 130.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 23.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.8,
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
            "value": 127,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 22.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 29.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 58.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 72.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 78.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 104,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 465.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 164.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 264.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 542.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 48.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 62.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 82.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 81.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 105.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 461.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 177,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 277,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 549.4,
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
            "value": 64.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 171.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 97,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 110.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 479.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 190.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 303.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 543.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 48.3,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 591.1,
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
            "value": 63,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 28.3,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2696.7,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3869.7,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 23.3,
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
            "value": 0.061,
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
            "name": "rdfs_infer_s",
            "value": 0.144,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1588686,
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
          "id": "e56f8515072502c8e076c2ebf9941c9d7d079cdd",
          "message": "feat(zk): ZK join manifest schema — CircuitId/ProofInputs::JoinEq + JoinEdge (sq-fi03) (#178)\n\n* feat(zk): ZK join manifest schema — CircuitId/ProofInputs::JoinEq + JoinEdge [OPUS-4.8]\n\nAdd the manifest-level schema for the hidden cross-credential JOIN (sq-fi03,\nstep 3 of sq-bwwl), following research/zk-hidden-join-design.md §2.2/§3.1-§3.2.\nSteps 1-2 (host join_value_commitment + the join_eq_na16_nb16 Noir member)\nlanded on main via PR #170; this wires their host-side schema.\n\nThree schema additions, mirroring the existing scan/filter members EXACTLY:\n\n1. CircuitId::JoinEq { n_a, n_b } — the (n_a, n_b) graph-size buckets name the\n   compiled member `join_eq_na{n_a}_nb{n_b}` (package()), exactly as\n   Scan { k, n, r } names scan_k…; derive_join_eq_id + JOIN_EQ_N_BUCKETS in\n   build.rs mirror derive_scan_id (v1 compiles the single 16x16 member).\n\n2. ProofInputs::JoinEq { id, commit_a, commit_b, join_commitment, slot_a,\n   slot_b } — the typed public inputs, EXACTLY the join_eq member's `pub`\n   params after the binding-carried `challenge`, in the same order:\n   [challenge, commit_a, commit_b, join_commitment, slot_a, slot_b]\n   (cross-referenced to zk/compose/join_eq_na16_nb16/src/main.nr). The join\n   VALUE + blinder are PRIVATE — never serialized. circuit_id() gains the arm.\n\n3. JoinEdge { scan_a, graph_a, scan_b, graph_b, join_proof } + a\n   join_edges: Vec<JoinEdge> field on ProofManifest (serde default → legacy\n   manifests parse) — the hidden-key analogue of BindingEdge: ties two scans'\n   commitments to a join_eq sub-proof, disclosing the graph linkage but not the\n   joined value.\n\nExhaustive-match wiring (workspace builds clean): circuit_id(), derive_id, and\nreconstruct_public_inputs gain JoinEq arms. reconstruct_public_inputs emits the\naudit-#1 public-input byte-layout (pure serialization matching the Noir order) —\nit bypasses no check. prover_toml_for's JoinEq arm is unimplemented! (NOT a\nsilent stub): it is UNREACHABLE here (nothing builds JoinEq proving inputs until\nstep 4) and the function signature cannot carry the join member's private\nwitnesses; sq-sfsi replaces it. No reachable panic introduced.\n\nSCOPE BOUNDARY (honest): SCHEMA + types ONLY. The bind_joins verifier gate\n(commitment-equality to the scan proofs + canonical-vk + the UnboundJoin query\nbinding + JoinPolicy) is sq-sfsi (step 4), now unblocked. NOT implemented here.\n\nTests (manifest::join_schema_tests): serde round-trip of ProofInputs::JoinEq +\nJoinEdge + CircuitId::JoinEq, package()/enumeration, additive-default parse of a\njoin-less manifest, and a field-ordering pin (JOIN_EQ_PUBLIC_INPUT_LAYOUT)\ncross-referencing the Noir main. 219/219 sparq-zk-compose nextest pass; clippy\n--all-targets -D warnings clean; workspace builds.\n\nRefs sq-fi03 (parent sq-bwwl step 3).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(zk-compose): prover_toml_for returns Err for JoinEq; correct JoinEdge canonicalisation doc [OPUS-4.8]\n\nPR #178 review threads:\n\n1. prover_toml_for is a pub fn that panicked via unimplemented! on\n   ProofInputs::JoinEq, a downstream-crash footgun (e.g. a CLI loading a\n   manifest containing join_eq). It now returns Result<_, ProverTomlError>\n   with a recoverable JoinEqUnsupported variant; the join proving path stays\n   deferred to sq-sfsi. All call sites updated; added a test asserting the\n   JoinEq arm returns Err (not panics).\n\n2. The JoinEdge doc claimed the manifest canonicalises join_edges by sorting\n   on (scan_a, graph_a, slot_a) — but no sorting exists and JoinEdge has no\n   slot field. binding_edges are likewise unsorted, so the honest fix is to\n   correct the doc (declaration order today; canonicalisation deferred) rather\n   than claim behaviour that isn't there. Tracked by sq-y2wy.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T15:45:21Z",
          "tree_id": "0fddff8b4e27cad2fea5efc16dd67c6decc25e51",
          "url": "https://github.com/jeswr/sparq/commit/e56f8515072502c8e076c2ebf9941c9d7d079cdd"
        },
        "date": 1781538547155,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.57,
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
            "value": 5.2419,
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
            "value": 3090.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4459.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 750.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13234.5,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 62396.3,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 165643.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 6614.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 44827.2,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9328.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 66836,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 170951.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 6295.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 43145.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29665,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 27.4,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 2199344.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6617.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3748,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3460.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7564.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 511161.9,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12455.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 35084.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 54737.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3794.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22467.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 152876.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 108330.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 193333,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 37726.3,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6885.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13387.3,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29931.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 2320388,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6482.8,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3822.7,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3526.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9413.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 515321.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12606.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 35055.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 54400.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3895.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 23473.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 14.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 164257.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 116277.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 213037,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 40076.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7036,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13451,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 30749.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.3,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 3073093.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6621.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3917.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3506.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9797.9,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 519799.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13101.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 35057.6,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 54401.9,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3981.5,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 23338.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 166325.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 110111.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 196928.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.5,
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
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 38123.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6943.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13397.4,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6378.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16140.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 15879.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 15478,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 466878.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 16451.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 23646.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 294541.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 21250.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 24292.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 289510.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 10361.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 18049.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14795.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14975.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 504789.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18939.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 23468.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 286537.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 21191.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 72,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23432.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 291969.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13805.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20972.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 15182.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 15244.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 510765.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 17461.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22509.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 286922.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 21129.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 135.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22961.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 290756.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 61.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 33.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 100.8,
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
            "value": 38.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 14.4,
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
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 864,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 27.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 113.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.6,
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
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 122.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 32.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 23.2,
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
            "value": 1527,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 29.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 30.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 134.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 21.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 136.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 22.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 13,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 57.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 74.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 79.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 112.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 473.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 170.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 271.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 563.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 48.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 63.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 84.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 80.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 131.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 463.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 173.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 278.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 551.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 48.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 61.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 190.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 81.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 112.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 482.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 200.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 324.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 548.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47.7,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 597.5,
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
            "value": 67.6,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 31.2,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2723.5,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3918.5,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 19.4,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 26,
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
            "value": 0.068,
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
            "name": "rdfs_infer_s",
            "value": 0.148,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1588686,
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
          "id": "81998a51fa8586c1ace411c15d824b3d5d66cdd7",
          "message": "feat(bench): SHACL validation benchmark suite (sq-7iai) (#179)\n\n* feat(bench): SHACL validation benchmark suite (sq-7iai)\n\n[OPUS-4.8] Replicate the LUBM/Deep-Taxonomy benchmark template onto SHACL, per\nresearch/capability-benchmark-program.md §3.1. The cleanest competitor surface:\nJena-SHACL / pySHACL / rdf-validate-shacl run the identical (data, shapes) pair.\n\nG1 runner choice: a crate EXAMPLE (crates/sparq-shacl/examples/bench_shacl.rs), NOT a\nsparq-cli subcommand — sparq-shacl is the isolated SHACL crate (not a sparq-cli\ndependency), so a CLI arm would break that isolation; the example is the natural surface\nand matches research/coverage-and-benchmark-plan.md §1.1. It emits, per shapes/*.ttl,\n`name\\tviolations\\tvalidate_us\\tconforms\\tfocus_nodes\\tload_us`.\n\n- bench/shacl/: gen.sh (thin wrapper over bench/lubm/gen.sh — reuses the LUBM(1) ABox as\n  the data substrate), 5 committed shape graphs (cardinality, datatype_range,\n  class_nodekind, node_paths, sparql_constraint) each with a fixed invalid-focus fraction\n  so violation counts are deterministic constants, expected.tsv, self-asserting run.sh,\n  README.md.\n- DETERMINISTIC gate (self-asserted in run.sh, exit 1 on drift): per-workload violations\n  (report.results.len()), conforms (0/1), focus_nodes. Derived by RUNNING sparq-shacl on\n  the pinned corpus + independently cross-checked vs the raw ABox (149/125/1874/1802/3738).\n  The shacl_w3c_pass ratchet (98/98, only-tightens) lives in sparq-shacl/tests/w3c_core.rs\n  (BASELINE_PASS). TIMING is ADVISORY (shacl_<workload>_validate_us, trend-only, NOT in\n  perf-gate.py; this box is non-canonical).\n- New crate API: sparq_shacl::count_focus_nodes (public, reuses GraphView target\n  primitives) + a unit test.\n- Guarded ci-bench.sh hook (reuses the javac/rapper guard since it reuses the LUBM corpus;\n  builds the example on demand).\n- Dashboard FEATURED_SUITES + GROUP_ORDER 'SHACL validation' row; gen-metric-labels.py\n  SHACL block (gap G2: stems(subdir, ext) generalised so *.ttl shapes enumerate);\n  metric-labels.json regenerated.\n- Registry: benchmarks.toml shacl-validate-bench + CATALOG.md rows. Competitors:\n  jena-shacl added (report-cli); pyshacl/rdf-validate-shacl already wired (sq-eifd #171) —\n  engines/values EMPTY in git (gather-only).\n\nSelf-gate verified: run.sh passes; a deliberate shape perturbation (maxCount 1->5) makes\nit exit 1 with a correctness error. cargo build (workspace), nextest -p sparq-shacl\n(108 pass), clippy (workspace --all-targets -D warnings), dashboard smoke + label drift\nall clean.\n\nsq-7iai\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(bench-shacl): always rebuild example + reject iters=0 (PR #179 review)\n\n[OPUS-4.8] Resolve two Copilot threads on the SHACL validation bench suite:\n\n- ci-bench.sh SHACL hook: replace the file-exists guard (which could run a STALE\n  cached target/release/examples/bench_shacl after rust-cache restores target/)\n  with an unconditional `cargo build --release -p sparq-shacl --example\n  bench_shacl` — cargo's own staleness detection makes a no-op rebuild cheap.\n  Also stop swallowing the build error with `|| true`: a real compile failure now\n  fails the gate (exit 1) instead of silently skipping the SHACL correctness check\n  on main.\n- bench_shacl.rs: reject iters=0 at parse time (exit 2 with a clear message). 0\n  made both `0..iters` loops skip, leaving load_us/validate_us at INFINITY and\n  tripping `data.expect(\"iters >= 1\")`. The deterministic name\\tcount\\tus output\n  is now guaranteed finite.\n\nVerified: gate run.sh passes (5/5 match expected.tsv), iters=0 rejected cleanly,\nnextest 108/108, clippy -D warnings clean. NON-CANONICAL timing.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T15:49:09Z",
          "tree_id": "95b233e4742593478d4d038a89976345685354ea",
          "url": "https://github.com/jeswr/sparq/commit/81998a51fa8586c1ace411c15d824b3d5d66cdd7"
        },
        "date": 1781538793773,
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
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3339.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4849.8,
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
            "value": 822.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13114.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 60526.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 162225.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2770.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42716.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8268.2,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 59252.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 161281.6,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2545.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38972.4,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29953.8,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 16.4,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 2143225.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 9.2,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6426.8,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3768.5,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3578,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7209.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 480224.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12932.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31078.4,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 54196.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3825.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22518.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 145258.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 107178.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 175401.8,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 36200.2,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6961.5,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13131.7,
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
            "value": 30015.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 2296700.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6580.7,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3803,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3537.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9095.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 481483,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 13170.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31105.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 54147.6,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3784.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22431.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 146516.2,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 105734.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 178823.7,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.3,
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
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 35757.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7123.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13261.2,
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
            "value": 30056.3,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 19.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 2247915.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6557,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3795.8,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3555.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9293.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 477149,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 13593.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 32419.5,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 54428.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 4161.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22663.8,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 13.3,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 145597,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 105379.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 178766.7,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 13.1,
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
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 38327.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6968.5,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13222.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6682.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 17034.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16500.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16670.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 453794.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17359.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24311.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 288095.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22969.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 23357.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 295997.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 14.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9498.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19724.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16807.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16572.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 504444.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18709.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24169.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 295636.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 23749.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 65.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23588.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 294639.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 16.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 14070.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 21683.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16599,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16447.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 518591.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 19046.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24702.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 291395,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23288.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 129.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 23640.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 300142.8,
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
            "value": 33.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 102,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 18.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 26.4,
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
            "value": 11.1,
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
            "value": 12.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 939.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 26.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 119.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 18.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 118.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 31.6,
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
            "value": 23.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1556.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 28.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 128.3,
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
            "value": 21,
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
            "value": 117.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 33.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 22.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 28.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 62.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 67.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 77.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 103.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 502.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 174.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 291.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 598.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 50.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 54.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 95.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 75,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 103.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 506.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 191.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 302.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 622.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 46.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 69.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 163.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 85.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 108,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 503.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 199.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 344.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 633.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 14.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 45.8,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 598.5,
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
            "value": 70.5,
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
            "value": 31.9,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2973.8,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4056.5,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 24.7,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.7,
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
            "value": 0.064,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 334.9,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 320,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13443.5,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6648,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 672689,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.145,
            "unit": "s"
          },
          {
            "name": "wasm_bundle_bytes",
            "value": 1588686,
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
          "id": "62e068dd91be32dc7589d91bc525a69495746edb",
          "message": "feat(engine): graph-IRI prefix range-scan index for prefix-scoped aggregates (sq-zz8z, gh-51) (#180)\n\n* feat(engine): graph-IRI prefix range-scan index for prefix-scoped aggregates (sq-zz8z, gh-51)\n\nPSS computes `usage(prefix)` as `SUM(?size) + COUNT(DISTINCT ?g)` with\n`FILTER(STRSTARTS(STR(?g), prefix))` over a `GRAPH ?g` enumeration of ALL named\ngraphs — O(graphs) per interactive call, and the no-cross-pod-leak boundary on a\nmulti-tenant box. Measurement (non-canonical, EC2 work box) showed this is in fact\nSUPER-linear and unusable past ~10k graphs, so an index is clearly worth it.\n\n[OPUS-4.8] Implements a graph-IRI prefix/range index instead of documenting the cost:\n\n- sparq-core: `Graph::for_named_graphs_with_prefix(prefix, f)` range-scans a cached,\n  lazily-built sorted permutation of the `named` indices (keyed by `named.len()`, the\n  only thing that changes when the SET of graph IRIs changes), so a prefix lookup is\n  O(log G + matches) instead of O(G). The positional `named` Vec (load-bearing for the\n  on-disk sub-tree + manifest) is NEVER reordered — the index is a separate side struct.\n  One shared `graph_name_str` defines `STR(name)` so the index ordering and a query's\n  `STRSTARTS(STR(?g), …)` cannot diverge.\n- sparq-engine: a `GRAPH ?g { … } FILTER(STRSTARTS(STR(?g), \"lit\"))` shape is recognised\n  (incl. inside a top-level `&&`) and the prefix pushed into the graph enumeration via the\n  range scan. The original FILTER still runs, so results are IDENTICAL — only the set of\n  graphs visited shrinks. Also fixes an O(G²) `union_bindings` fold in `GRAPH ?g` (the\n  per-graph relations now accumulate into one flat buffer in a stable `?g`-first schema).\n\nCorrectness: equivalence tests assert the indexed path == a plain-Rust STRSTARTS oracle ==\nthe core range-scan API across empty / no-match / exact-match / prefix-is-a-substring /\npercent-encoded-IRI edge cases; the PSS SUM+COUNT(DISTINCT) shape is verified prefix-scoped;\nsame-object cache coherence across add/remove is locked in sparq-core. STR(IRI)+STRSTARTS\nsemantics already matched the SPARQL spec / QLever (simple-literal comparison) — no fix\nneeded, no parity bead.\n\ncargo nextest (sparq-core 63, sparq-engine 227) + clippy -D warnings both clean.\nMeasured timings are advisory only and kept out of committed docs/tests (repo hygiene).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* perf: drop prefix-index lock before callback + O(1) row remap [OPUS-4.8]\n\nPR #180 review fixes (sq-zz8z):\n- sparq-core for_named_graphs_with_prefix: collect matching graph\n  indices under the graph_prefix_index lock, DROP the guard, THEN\n  invoke the caller-provided f. Holding the mutex across f needlessly\n  serialized concurrent readers and risked deadlock on re-entry. The\n  match snapshot is taken under the lock, so correctness is preserved.\n- sparq-engine GRAPH ?g row remap: precompute a schema-column ->\n  position-in-b.vars map ONCE per source binding-set, then map each\n  row with O(1) indexed lookups, instead of an O(vars) linear\n  position() search per (row, var) cell (an O(rows*vars^2) hotspot\n  on large GRAPH ?g scans). Results identical.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T15:55:25Z",
          "tree_id": "52e98c7c14fc84a08fbb46bc72a21a22261643a9",
          "url": "https://github.com/jeswr/sparq/commit/62e068dd91be32dc7589d91bc525a69495746edb"
        },
        "date": 1781539162695,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.558,
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
            "value": 3.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3075.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4370.5,
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
            "value": 689.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12714.8,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 57358.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 156243.8,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4146.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 48836.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 8064.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 62932.1,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 159550.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3882.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 39397.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29874.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.5,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1366560,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6201.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3679,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3403.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7219.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 519725.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12319.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 34045.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 52426.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3609.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22917.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 135574.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 91064.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 162445.3,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 11.1,
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
            "value": 7,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35942.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7653.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13244,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29823.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1550462.4,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6297.6,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3688.8,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3415,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8680,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 510933.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12702.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 34376.9,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53830.6,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3670.2,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 21649.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 137808.1,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 89949.5,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 158747.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.6,
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
            "value": 35426.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6616.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13201.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 30244.2,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 23.7,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1260209.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6243.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3694.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3346.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8228.9,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 505050.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12078.1,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 34236.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 52672.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3717.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22230.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 142807.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 96603.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 166698.7,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_json_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 38695.5,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7323.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 14505.8,
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
            "value": 6293,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15843.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 15286.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 15135.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 454942.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 16318.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22722.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 297327.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 21675.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22317.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 289972.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 15.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9487.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 16912.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 15117.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14971.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 494478.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 17694.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 22347.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 288024.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 21947.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 58.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23278.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 292069.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13005.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 21115.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 15553,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 15137.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 510068.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 17852.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22637.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 289123.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 21594.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 114.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 23459.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 289432.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 56.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 29.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 115.8,
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
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 38.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 12.6,
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
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 874.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 27.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 28.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 108,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 17.5,
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
            "value": 30.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 24.3,
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
            "value": 1401.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 30.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 129.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 18.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 20.4,
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
            "value": 130.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 33.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 28,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 55.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 69.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 77.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 101.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 478.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 164.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 266,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 555.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 51.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 60.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 81.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 79.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 106.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 476.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 176.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 276.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 558.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 49.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 58,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 149,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 80.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 115.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 467.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 185.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 297.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 554.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 47.9,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 595.9,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 67.9,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 30.3,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29.6,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2699.1,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3985.7,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 19,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 23.6,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 19.7,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.007,
            "unit": "s"
          },
          {
            "name": "deeptax_d1000_query_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_triples",
            "value": 2001,
            "unit": "triples"
          },
          {
            "name": "deeptax_d10000_closure_s",
            "value": 0.068,
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
            "value": 393.2,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 343.2,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13221.9,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 7137.8,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 791972.9,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.167,
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
          "id": "f37bab4b5c7576d4c1bb9937ee915c192a6ebb1c",
          "message": "test(sparq-solid): differential — resolve_var_graphs set == engine write set (sq-3jtd.1) (#181)\n\n* test(sparq-solid): differential — resolve_var_graphs set == engine write set (sq-3jtd.1) [OPUS-4.8]\n\nAdds a differential test module to crates/sparq-solid/src/update.rs guarding\nthe load-bearing write-path security invariant: for a DELETE/INSERT…WHERE with\na GRAPH ?var slot, PodStore's precise `resolve_var_graphs` target set must EQUAL\nthe set of named graphs `sparq_engine::update_in_place` actually writes — no\nover-approximation (false denials) and crucially no under-approximation (a write\nescaping authorization = security hole).\n\nThe engine's real write set is observed via `update_in_place_capturing`, whose\n`UpdateEffect::Delta { slot, .. }` records every graph slot the engine touched;\nthe precise set comes straight from `resolve_var_graphs`. Cases mirror PSS's\nsetAclPointer/putContainer (gh-47):\n\n- OPTIONAL binds (pointer rewrite): precise == engine write set {r1,r2}.\n- OPTIONAL unbound for some rows (DELETE quad dropped when ?p unbound): precise\n  == engine write set; no phantom graph, no escalation.\n- OPTIONAL empty binding: precise == engine write set == ∅ (no-op, no wildcard).\n- WITH-clause variant: resolve_var_graphs deliberately bails to the conservative\n  all-graphs wildcard (sq-cnor); test asserts the wildcard is a SOUND superset of\n  the engine's real write set (no under-approx hole) and documents it as a strict\n  over-approximation (the sq-cnor precision gap).\n\nInvariant HELD across every case — no divergence, no security bug, no bead filed.\nThis is the differential GUARD so sq-cnor's USING/WITH precision work cannot drift\nfrom the engine's build_using semantics.\n\nRefs sq-3jtd.1, parent sq-3jtd, gh-47.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* test: strengthen writeset differential — blank-node slots + WITH default-graph assert [OPUS-4.8]\n\nPR #181 review fixes (sq-3jtd.1):\n- engine_write_set: include blank-node graph slots in the named write\n  set (keyed _:label) instead of silently dropping them — a graph name\n  is a NamedNode OR BlankNode per RDF, so dropping the blank-node case\n  could mask a real engine write and defeat the differential guard.\n  Replace the silent Some(_) catch-all with an explicit panic on any\n  malformed (literal/triple-term) graph slot so regressions fail loudly.\n- with_clause test: capture and assert touched_default. Every template\n  quad is explicitly GRAPH ?g-scoped, so WITH's default-graph re-scope\n  must NOT produce a default-graph write; assert !default rather than\n  discarding the signal (a regression routing writes to the WITH default\n  graph would otherwise slip past the subset check).\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T15:57:31Z",
          "tree_id": "69948303b0e913e4e7d06b27ce894cba23abf6fd",
          "url": "https://github.com/jeswr/sparq/commit/f37bab4b5c7576d4c1bb9937ee915c192a6ebb1c"
        },
        "date": 1781539411954,
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
            "value": 3258.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4771.6,
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
            "value": 778.7,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 13304.7,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 59235.4,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 164492,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 4098.7,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.8,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 42294.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7267.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 57679.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 149142.1,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2238.2,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38478.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29862.4,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 16.1,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1587234.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6157.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3727.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3573.2,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7247.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 475636.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 13231.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 31345.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 54508.9,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 4144.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22511.7,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 147229.3,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 108461.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 190130,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.6,
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
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 37278.9,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 7203.1,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 13458.9,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29765.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1489746.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6192.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3803.2,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3526.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 8339.9,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 476560,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12676.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30857.5,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 54191.2,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3787.1,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22372.4,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 13.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 139189.2,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 103680.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 174503.2,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 37005.8,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6892.9,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13089.6,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 29752.8,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18.7,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1675445.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6256.9,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3780.6,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3499.1,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8721.6,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 475638.2,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12822.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 32022.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 54828.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3689.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22198.3,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 144505.8,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 103508.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 173812.3,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9.9,
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
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 36482.1,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7014.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13206.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6697.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 16785.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16258.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16204.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 450863.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17347.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 23550.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 295179,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22484.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 22692,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 286143.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9159.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19272.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16635.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16405.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 492592,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18354.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24084.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 292357.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 22914.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 62.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 22620.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 289787.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 12938.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20077.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16262.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16148.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 496937.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 18658.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24056.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 295357.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23004.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 93.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22446.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 295805.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 52,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 31.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 28,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 110.7,
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
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 31.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 14.5,
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
            "value": 936.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 25.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 26.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 107.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 14.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 108.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 31.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1356.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 27.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 27.5,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 119.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 19.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 111.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 34,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 16.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 54.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 65.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 75.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 100.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 489.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 170.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 278.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 602.2,
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
            "value": 58.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 77.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 81.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 101.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 506.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 179.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 284.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 603.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 45.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 58.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 147,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 75,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 114,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 483.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 189.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 321.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 6.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 610.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 45.6,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 616.3,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.7,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 66.6,
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
            "value": 2919.6,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4040.4,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 17.8,
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
            "value": 354.1,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 303.6,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13119.5,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6556.7,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 666741,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.149,
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
          "id": "b3aaeca193ee291954ff9bd9d7f8fa730da0a454",
          "message": "chore(beads): re-export issues.jsonl mirror (session bead closes + new beads) (#182)\n\nSyncs the committed bead mirror with the dolt source of truth after a large\nmerge train: ~14 PRs landed + their beads closed, plus new beads filed by\nsub-agents (sq-sfsi/sq-y2wy/sq-kep2/sq-zn0x/sq-nx0s/sq-0x65/sq-pjhc/sq-6te5/\nsq-yqi1 etc.). Mirror-only; no dolt edits.\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T16:08:09Z",
          "tree_id": "99d17ff358a942ed96291a808923fbb97c01bee4",
          "url": "https://github.com/jeswr/sparq/commit/b3aaeca193ee291954ff9bd9d7f8fa730da0a454"
        },
        "date": 1781539950089,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "load_s",
            "value": 0.598,
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
            "value": 5.7044,
            "unit": "ns/byte"
          },
          {
            "name": "store_bytes_per_triple_small",
            "value": 88,
            "unit": "bytes"
          },
          {
            "name": "q02_type_person_count_us",
            "value": 3.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3101.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4832,
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
            "value": 766,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 16643.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 71606.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 191185.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2835.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 52026.1,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 9865.6,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 66744.5,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 171605.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3684.1,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 6.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 48576.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.2,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 32395.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 2063242.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 7655.8,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 4459.7,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 4454.4,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 9291.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 539288.5,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 14880.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 44982.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 67684.4,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 4694.9,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 23335,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 10.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 152279.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 108557.2,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 200834.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 38340.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 8451.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 14330.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.1,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 32000.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 18.6,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 2154275.3,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 8085.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 4773,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 4334.9,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 10451.4,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 531766.3,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 14958.4,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 44721.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 66753.6,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 4623.5,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 24428.8,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 151393.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 105360,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 191857.1,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 38832.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7861.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 14255.2,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 32941.7,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 2241233.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 7856.3,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 5111.3,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 4690.8,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 10878.8,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 544823.9,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 15160.6,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 45601,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 67166.5,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 4783.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 23619.6,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 152439.3,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 109822.3,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 196835.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 11.2,
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
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 39213,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 8110.6,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 14539.6,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 5950.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 17086,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 16178.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 16786.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 543694.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 17017.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 22548.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 324473.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22883.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 20677.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 333139.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9340.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 19765.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16921.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16734.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 605288.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 17686.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 21828.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 328090.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 22946.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 51.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 20003.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 318083.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 14.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 11644.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 20624.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 16708.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16180.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 594720.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 18578,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 23362.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 326408.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 23906.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 81.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 20666.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 328447.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 53.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 30.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 27.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 104.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 15.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 37.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 13.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.4,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 824.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 24.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 26.2,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 109.6,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_materialize_us",
            "value": 15.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_materialize_us",
            "value": 13.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_materialize_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_materialize_us",
            "value": 9.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 134.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 28.5,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 15.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 14,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 20.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 9.8,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1192.4,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 26.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 26.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 141.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 17.2,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 15,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 8,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 137.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 30.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 19.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 24.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 53.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 61.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 73.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 102.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 421.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 164.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 249.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 506.9,
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
            "value": 75.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 109.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 108.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 139.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 423.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 164.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 253.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 518.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 9.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 43.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 73.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 155.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 76.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 105,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 426.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 177.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 305.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 511.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 43.7,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 12.2,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 522.6,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 17.7,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 4.3,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 84.4,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 32.7,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.6,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 31.1,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2434,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3670.5,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 23.9,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 28.6,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.007,
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
            "value": 0.075,
            "unit": "s"
          },
          {
            "name": "deeptax_d10000_query_us",
            "value": 5.3,
            "unit": "us"
          },
          {
            "name": "deeptax_d10000_closure_triples",
            "value": 20001,
            "unit": "triples"
          },
          {
            "name": "shacl_cardinality_validate_us",
            "value": 344.9,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 314.6,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 12862.2,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6509,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 678104.6,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.187,
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
          "id": "396d71d9d18a92ea0a7ab756383f5559b84ab0a4",
          "message": "feat(zk-compose): bind_joins verifier gate for hidden cross-credential JOIN (sq-sfsi) (#183)\n\n* feat(zk-compose): bind_joins verifier gate for hidden cross-credential JOIN (sq-sfsi) [OPUS-4.8]\n\nStep 4 of sq-bwwl: the verifier gate that enforces ProofManifest::join_edges\n(the hidden-key analogue of binding_edges). For each declared JoinEdge it\nenforces three properties (a manifest whose join proof/commitments/binding do\nnot line up is REJECTED):\n\n1. Commitment-matching (anti-A2, design §2.3/§4.2): the join_eq proof's public\n   commit_a/commit_b must byte-equal the two referenced SCAN sub-proofs' bound\n   commitments[graph_a]/commitments[graph_b] — so a join can't bind rows from\n   unrelated/forged scans. The scan commitments are audit-#1 byte-bound into the\n   scan proofs and issuer-attested+in-K (bind_issuer_attestations), so the join\n   is over two genuine attested credentials. (JoinCommitmentMismatch)\n2. Canonical VK (anti-A1): the join_eq proof is verified against the verifier-\n   recomputed canonical vk (CircuitId::JoinEq) in verify_manifest's per-sub-proof\n   loop (audit-#2) over the audit-#1-reconstructed public inputs — unchanged\n   discipline; bind_joins is the structural gate that ties those bound inputs to\n   the scans + query.\n3. Slot binding (§4.4): the join_eq's public slot_a/slot_b must equal the\n   query-derived slots a SHARED variable occupies across the two answered\n   patterns (variable_slots) — rejects a join over the wrong column AND a spurious\n   join over an unrelated scan pair. (JoinSlotMismatch)\n\nWired into prefilter_manifest_structure after bind_issuer_attestations.\n\nHonest scope boundary: bind_joins validates DECLARED hidden joins; it does NOT\ndemand a hidden join for every query cross-scan shared variable — that is\ndischarged by the disclosed-row path (recheck/join_obligations), the hidden\nJoinEdge being the opt-in privacy alternative. A dropped hidden edge falls back\nto the disclosed path (not a hole); demanding a hidden join would wrongly break\ndisclosed joins (the e2e attribution/distinct-salts suite). Multi-way (N-way)\nchains and the join_eq PROVING path / FULL-bb accept are deferred (sq-r2s8).\n\nTests (join_gates.rs): structural ACCEPT (honest cross-scan join passes) +\nreject matrix — commit_a/commit_b not matching the referenced scans\n(cross-scan forgery), edge pointing at the wrong scan, wrong slot_a/slot_b,\nspurious/wrong-slot edge, dangling proof/graph index, kind mismatches; plus a\ndisclosed-path fallback scope assertion. The FULL-bb accept is #[ignore]'d\n(join_eq proving deferred from sq-fi03) — sq-r2s8 tracks build_join +\nprover_toml_for JoinEq + un-ignoring it. verifier_errors.rs covers the 4 new\nCheckError Display arms.\n\ncargo nextest -p sparq-zk-compose: 232 passed, 30 skipped; clippy clean.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n* fix(zk-compose): bind_joins multi-scan-per-pattern soundness (sq-sfsi) [OPUS-4.8]\n\nbind_joins used scan_for_pattern via position(..) (FIRST match), then required\nthe join edge's scan_a/scan_b to equal that first-match scan. A query pattern\ncan LEGITIMATELY be answered by more than one scan (the same triple pattern\nsatisfied by two credentials) — the disclosed-row path (bind_query_correctness)\nalready treats this as a first-class config (.any + per-scan FILTER loop). The\nfirst-match logic would (a) reject a valid join whose edge points at a non-first\nscan answering the pattern, or (b) let sub_proofs ordering decide which scan the\nslot binding validates against (soundness gap).\n\nReplace scan_for_pattern (first-match) with pattern_answered_by_scan(pi, idx), a\nmembership test against the SPECIFIC edge.scan_a/edge.scan_b. Anti-forgery\nbinding preserved: the edge must still reference a scan that genuinely answers\nthe right pattern at the right slot — only the \"which scan\" resolution changed\nfrom positional to the referenced index. UNION is out-of-fragment so the\nmulti-credential case is the reachable one.\n\nTests: multi_scan_join_edge_points_at_second_scan_passes (edge -> 2nd scan\nanswering pattern A; rejected pre-fix), multi_scan_join_edge_points_at_non_-\nanswering_scan_rejected (edge scan_a -> a scan not answering pattern A =>\nJoinSlotMismatch; binding not weakened). All 234 sparq-zk-compose tests pass;\nclippy -D warnings clean.\n\nCo-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T16:24:29Z",
          "tree_id": "2faadcaac3190763402286be3db0d75f352e7a1e",
          "url": "https://github.com/jeswr/sparq/commit/396d71d9d18a92ea0a7ab756383f5559b84ab0a4"
        },
        "date": 1781540913433,
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
            "value": 3075.6,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4395.3,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 697.4,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12306.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 55268,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 146628.2,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2549.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 39373.5,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7838.3,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 56153.2,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 152843.7,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 3121.8,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.9,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38044.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 4,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29364,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 15.2,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 1309583.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.7,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6255,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3709.4,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3391.7,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7190.7,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 503959.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 12340.5,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 30889.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 51808.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3941.7,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 21079.2,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 11.5,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 123002.5,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 91055.8,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 154901.5,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_count_us",
            "value": 6.9,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_count_us",
            "value": 8.1,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 34756,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6631.7,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12545.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.6,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 4.6,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 28534.6,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 16.4,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 1177220.5,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6318.4,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3723.9,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 3438.6,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 7999,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 527212.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12130.3,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 31191.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 51522.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3766.6,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22094.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.9,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 122303.7,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 90390.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 153470.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.9,
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
            "value": 34039,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 6365.4,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 12607.5,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_json_us",
            "value": 4.2,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_json_us",
            "value": 28416.5,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 1185860.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 8.3,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6425.5,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3695.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3390.5,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 8150.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 510614.4,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 11956.8,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30203.3,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 51515.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3615.5,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 21421.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 13.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 119784.4,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 90461.7,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 153959.3,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_json_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_json_us",
            "value": 12.3,
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
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 34651.6,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 6199.3,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 12484.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 8.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6848.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 15192.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 14784.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 14802.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 410862,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 15151.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 21872,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 281959.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 21123.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 21485.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 283339.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 5.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 8487.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 15974.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 14789.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 14586.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 461198.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 16275.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 21775,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 283854.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 20786.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 54.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 21301.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 281248,
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
            "value": 11458.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 17644.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 14930.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 14953.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 454563.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 16386,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 22303.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 283492.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 20637.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 107.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 22684.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 283659.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.5,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 48.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 32.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 31.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 102.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_count_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_count_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_count_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.3,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 37.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_count_us",
            "value": 14.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_count_us",
            "value": 13.3,
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
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 10.3,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 862.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 27,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 112.7,
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
            "value": 11.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_materialize_us",
            "value": 123.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 32.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 23.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 11.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.9,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1310,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 28.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 124.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 19.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 20.1,
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
            "value": 125.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 33.1,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 21.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 17.1,
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
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 56.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 68.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 80,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 104.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 480.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 164.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 267.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 554.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 48.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 57.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 82.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 89,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 107.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 495.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 172.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 269.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 549.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 49.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 57.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 150.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 82.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 110.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 477.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 192.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 299.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 7.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 545.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 48.5,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 580.8,
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
            "value": 65.7,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 27.9,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 29,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2660,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 3865.9,
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
            "value": 22.8,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 19.3,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
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
            "value": 354.5,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 333.7,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13116.8,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6953.4,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 764177.7,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.148,
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
          "id": "4afca70a897d3eb113bd3f5ddc453366155f6771",
          "message": "feat(sparq-mpc): in-protocol range proof of the secret-shared sum [OPUS-4.8] (#185)\n\nsq-nx0s — close the last unenforced seam in disclose_threshold_verdict.\nPreviously the sum-magnitude bound (sum < 2^DECOMP_VALUE_BITS) was a CALLER\nPRECONDITION only: an out-of-range secret-shared sum silently produced a WRONG\nverdict because a sharing's magnitude can't be read off the shares.\n\nAdd verify_sum_in_range: after the in-MPC bit-decomposition, PROVE — without\nreconstructing the sum — that sum is in [0, 2^DECOMP_VALUE_BITS) via two secret\nzero-tests over the recovered shared bits:\n  (1) recompose: sum == Σ b_k·2^k  (no field wrap, fits L=60 bits)\n  (2) magnitude: every bit >= DECOMP_VALUE_BITS is zero (sum < 2^20)\nClause (1) is the soundness load-bearer against field wraparound — a\nmagnitude-only check would wrongly accept large wrapping sums whose recovered\nlow bits look small. On violation: fail-closed MpcError::Protocol (abort), not\na silent wrong verdict.\n\nPrivacy preserved: each zero-test (secret_is_zero, the same masked v·r\nequality-to-zero primitive HiddenValueJoin::secure_equal uses) opens ONLY a\nuniform mask product — zero in range, uniform-nonzero otherwise — so the sum is\nstill never reconstructed.\n\nTests: in-range (0/1/mid/2^20-1) accepted+correct; out-of-range (2^20, 2^20+k,\n2^30, 2^59, 2^60-1, 2^60, near-p wrap, p/2) rejected fail-closed; boundary\n(2^20-1 accept / 2^20 reject); field-wrap soundness sweep over 40 seeds near p;\nprivacy (two different in-range sums disclose byte-identical partial, sum never\nappears); secret_is_zero primitive correctness. 241/241 nextest, clippy clean.\n\nHonest-majority, semi-honest (inherits the backend model; NOT malicious).\n\nCo-authored-by: Jesse Wright <jesse@jeswr.org>\nCo-authored-by: Claude Opus 4.8 (1M context) <noreply@anthropic.com>",
          "timestamp": "2026-06-15T16:28:39Z",
          "tree_id": "881b2246430c7bfa8eea0bf378f74a84217e5db7",
          "url": "https://github.com/jeswr/sparq/commit/4afca70a897d3eb113bd3f5ddc453366155f6771"
        },
        "date": 1781541161313,
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
            "value": 3.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_count_us",
            "value": 3303.9,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_count_us",
            "value": 4759.4,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_count_us",
            "value": 4.9,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_count_us",
            "value": 4.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_count_us",
            "value": 775.9,
            "unit": "us"
          },
          {
            "name": "q02_type_person_materialize_us",
            "value": 12920.4,
            "unit": "us"
          },
          {
            "name": "q03_star3_materialize_us",
            "value": 60486.7,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_materialize_us",
            "value": 162919.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_materialize_us",
            "value": 2601.4,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_materialize_us",
            "value": 4.7,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_materialize_us",
            "value": 41528.6,
            "unit": "us"
          },
          {
            "name": "q02_type_person_json_us",
            "value": 7100.1,
            "unit": "us"
          },
          {
            "name": "q03_star3_json_us",
            "value": 59041.8,
            "unit": "us"
          },
          {
            "name": "q04_follows_name_json_us",
            "value": 155940.9,
            "unit": "us"
          },
          {
            "name": "q06_filter_age_json_us",
            "value": 2324.5,
            "unit": "us"
          },
          {
            "name": "q09_count_edges_json_us",
            "value": 5.4,
            "unit": "us"
          },
          {
            "name": "q10_optional_age_json_us",
            "value": 38226.4,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_count_us",
            "value": 3.8,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_count_us",
            "value": 29348.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_count_us",
            "value": 16,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_count_us",
            "value": 2238293.6,
            "unit": "us"
          },
          {
            "name": "op_q05_union_count_us",
            "value": 8.8,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_count_us",
            "value": 6340.2,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_count_us",
            "value": 3816.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_count_us",
            "value": 3617,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_count_us",
            "value": 7223.2,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_count_us",
            "value": 481076.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_count_us",
            "value": 13251.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_count_us",
            "value": 30313.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_count_us",
            "value": 53294.1,
            "unit": "us"
          },
          {
            "name": "op_q14_values_count_us",
            "value": 3849.3,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_count_us",
            "value": 22533.1,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_count_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_count_us",
            "value": 135384.9,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_count_us",
            "value": 111261.9,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_count_us",
            "value": 176678.9,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_count_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_count_us",
            "value": 10.5,
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
            "value": 7,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_count_us",
            "value": 35983.7,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_count_us",
            "value": 6964.5,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_count_us",
            "value": 12981.7,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_count_us",
            "value": 9.1,
            "unit": "us"
          },
          {
            "name": "op_q01_bgp_materialize_us",
            "value": 5,
            "unit": "us"
          },
          {
            "name": "op_q02_star3_materialize_us",
            "value": 29894.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_materialize_us",
            "value": 17.8,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_materialize_us",
            "value": 2399226.9,
            "unit": "us"
          },
          {
            "name": "op_q05_union_materialize_us",
            "value": 8.5,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_materialize_us",
            "value": 6553,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_materialize_us",
            "value": 3829.1,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_materialize_us",
            "value": 5114,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_materialize_us",
            "value": 9773.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_materialize_us",
            "value": 486792.8,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_materialize_us",
            "value": 12783.2,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_materialize_us",
            "value": 30898.2,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_materialize_us",
            "value": 53401.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_materialize_us",
            "value": 3774,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_materialize_us",
            "value": 22264.8,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_materialize_us",
            "value": 12.7,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_materialize_us",
            "value": 136852.2,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_materialize_us",
            "value": 103577.6,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_materialize_us",
            "value": 180750.6,
            "unit": "us"
          },
          {
            "name": "op_q20_path_opt_materialize_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "op_q21_path_seq_materialize_us",
            "value": 12.6,
            "unit": "us"
          },
          {
            "name": "op_q22_path_alt_materialize_us",
            "value": 7.7,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_materialize_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_materialize_us",
            "value": 7.8,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_materialize_us",
            "value": 37132.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_materialize_us",
            "value": 7465.8,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_materialize_us",
            "value": 13014.7,
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
            "value": 29844.9,
            "unit": "us"
          },
          {
            "name": "op_q03_chain_json_us",
            "value": 18,
            "unit": "us"
          },
          {
            "name": "op_q04_triangle_json_us",
            "value": 2575123.8,
            "unit": "us"
          },
          {
            "name": "op_q05_union_json_us",
            "value": 7.9,
            "unit": "us"
          },
          {
            "name": "op_q06_optional_json_us",
            "value": 6496,
            "unit": "us"
          },
          {
            "name": "op_q07_optional_notbound_json_us",
            "value": 3971,
            "unit": "us"
          },
          {
            "name": "op_q08_minus_json_us",
            "value": 3611.3,
            "unit": "us"
          },
          {
            "name": "op_q09_filter_numeric_json_us",
            "value": 9343.5,
            "unit": "us"
          },
          {
            "name": "op_q10_filter_string_json_us",
            "value": 484515.1,
            "unit": "us"
          },
          {
            "name": "op_q11_filter_in_json_us",
            "value": 12902.7,
            "unit": "us"
          },
          {
            "name": "op_q12_filter_exists_json_us",
            "value": 30846.7,
            "unit": "us"
          },
          {
            "name": "op_q13_bind_json_us",
            "value": 53541.3,
            "unit": "us"
          },
          {
            "name": "op_q14_values_json_us",
            "value": 3731.8,
            "unit": "us"
          },
          {
            "name": "op_q15_agg_group_having_json_us",
            "value": 22328.5,
            "unit": "us"
          },
          {
            "name": "op_q16_distinct_json_us",
            "value": 12.4,
            "unit": "us"
          },
          {
            "name": "op_q17_orderby_limit_offset_json_us",
            "value": 142075.6,
            "unit": "us"
          },
          {
            "name": "op_q18_path_plus_json_us",
            "value": 111704.4,
            "unit": "us"
          },
          {
            "name": "op_q19_path_star_json_us",
            "value": 207456.3,
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
            "value": 7.2,
            "unit": "us"
          },
          {
            "name": "op_q23_path_inverse_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q24_path_negated_pset_json_us",
            "value": 8.2,
            "unit": "us"
          },
          {
            "name": "op_q25_subquery_json_us",
            "value": 37311.4,
            "unit": "us"
          },
          {
            "name": "op_q26_ask_json_us",
            "value": 7100.2,
            "unit": "us"
          },
          {
            "name": "op_q27_construct_json_us",
            "value": 13040.1,
            "unit": "us"
          },
          {
            "name": "op_q28_describe_json_us",
            "value": 9,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_count_us",
            "value": 11.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_count_us",
            "value": 6747.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_count_us",
            "value": 17475.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_count_us",
            "value": 17070.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_count_us",
            "value": 17012.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_count_us",
            "value": 462520.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_count_us",
            "value": 18141.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_count_us",
            "value": 24917.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_count_us",
            "value": 288401.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_count_us",
            "value": 22572.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_count_us",
            "value": 4.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_count_us",
            "value": 24884.2,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_count_us",
            "value": 291521.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_count_us",
            "value": 6,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_materialize_us",
            "value": 13.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_materialize_us",
            "value": 9610.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_materialize_us",
            "value": 18918.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_materialize_us",
            "value": 16295.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_materialize_us",
            "value": 16196.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_materialize_us",
            "value": 497826.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_materialize_us",
            "value": 18479.7,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_materialize_us",
            "value": 24772.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_materialize_us",
            "value": 290391.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_materialize_us",
            "value": 23727.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_materialize_us",
            "value": 62.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_materialize_us",
            "value": 23372,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_materialize_us",
            "value": 288691.5,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_materialize_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q01_json_us",
            "value": 15.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q02_json_us",
            "value": 13175.4,
            "unit": "us"
          },
          {
            "name": "sp2b_q03a_json_us",
            "value": 21800.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q03b_json_us",
            "value": 17119.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q03c_json_us",
            "value": 16337.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q04_json_us",
            "value": 503998.1,
            "unit": "us"
          },
          {
            "name": "sp2b_q05b_json_us",
            "value": 19518.8,
            "unit": "us"
          },
          {
            "name": "sp2b_q07_json_us",
            "value": 24888.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q08_json_us",
            "value": 294629.9,
            "unit": "us"
          },
          {
            "name": "sp2b_q09_json_us",
            "value": 24284.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q10_json_us",
            "value": 94,
            "unit": "us"
          },
          {
            "name": "sp2b_q11_json_us",
            "value": 24713.3,
            "unit": "us"
          },
          {
            "name": "sp2b_q12b_json_us",
            "value": 300948.6,
            "unit": "us"
          },
          {
            "name": "sp2b_q12c_json_us",
            "value": 5.6,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_count_us",
            "value": 45,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_count_us",
            "value": 31.6,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_count_us",
            "value": 27.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_count_us",
            "value": 99.6,
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
            "value": 7.4,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_count_us",
            "value": 6.1,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_count_us",
            "value": 11.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_count_us",
            "value": 31,
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
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_count_us",
            "value": 12,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_count_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_count_us",
            "value": 9.9,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_materialize_us",
            "value": 959.8,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_materialize_us",
            "value": 25,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_materialize_us",
            "value": 27.1,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_materialize_us",
            "value": 104.8,
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
            "value": 112,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_materialize_us",
            "value": 31.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_materialize_us",
            "value": 17.4,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_materialize_us",
            "value": 15.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_materialize_us",
            "value": 23,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_materialize_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_materialize_us",
            "value": 10.7,
            "unit": "us"
          },
          {
            "name": "watdiv_C3_json_us",
            "value": 1357.9,
            "unit": "us"
          },
          {
            "name": "watdiv_F2_json_us",
            "value": 27.3,
            "unit": "us"
          },
          {
            "name": "watdiv_F3_json_us",
            "value": 27.7,
            "unit": "us"
          },
          {
            "name": "watdiv_F5_json_us",
            "value": 122.7,
            "unit": "us"
          },
          {
            "name": "watdiv_L1_json_us",
            "value": 20.5,
            "unit": "us"
          },
          {
            "name": "watdiv_L2_json_us",
            "value": 17,
            "unit": "us"
          },
          {
            "name": "watdiv_L3_json_us",
            "value": 19.8,
            "unit": "us"
          },
          {
            "name": "watdiv_L4_json_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "watdiv_L5_json_us",
            "value": 10.6,
            "unit": "us"
          },
          {
            "name": "watdiv_S1_json_us",
            "value": 112.9,
            "unit": "us"
          },
          {
            "name": "watdiv_S2_json_us",
            "value": 32.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S3_json_us",
            "value": 20.7,
            "unit": "us"
          },
          {
            "name": "watdiv_S4_json_us",
            "value": 16.8,
            "unit": "us"
          },
          {
            "name": "watdiv_S5_json_us",
            "value": 27.2,
            "unit": "us"
          },
          {
            "name": "watdiv_S6_json_us",
            "value": 12.3,
            "unit": "us"
          },
          {
            "name": "watdiv_S7_json_us",
            "value": 11.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_count_us",
            "value": 57.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_count_us",
            "value": 73.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_count_us",
            "value": 76,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_count_us",
            "value": 99.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_count_us",
            "value": 501.6,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_count_us",
            "value": 178.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_count_us",
            "value": 285.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_count_us",
            "value": 7.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_count_us",
            "value": 594.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_count_us",
            "value": 8.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_count_us",
            "value": 45.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_materialize_us",
            "value": 57.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_materialize_us",
            "value": 77.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_materialize_us",
            "value": 81,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_materialize_us",
            "value": 102.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_materialize_us",
            "value": 496.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_materialize_us",
            "value": 177.5,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_materialize_us",
            "value": 285.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_materialize_us",
            "value": 7,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_materialize_us",
            "value": 615.7,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_materialize_us",
            "value": 10.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_materialize_us",
            "value": 47,
            "unit": "us"
          },
          {
            "name": "bsbm_query01_json_us",
            "value": 58.9,
            "unit": "us"
          },
          {
            "name": "bsbm_query02_json_us",
            "value": 157.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query03_json_us",
            "value": 81.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query04_json_us",
            "value": 107,
            "unit": "us"
          },
          {
            "name": "bsbm_query05_json_us",
            "value": 504.2,
            "unit": "us"
          },
          {
            "name": "bsbm_query07_json_us",
            "value": 199.3,
            "unit": "us"
          },
          {
            "name": "bsbm_query08_json_us",
            "value": 326.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query09_json_us",
            "value": 6.8,
            "unit": "us"
          },
          {
            "name": "bsbm_query10_json_us",
            "value": 625.4,
            "unit": "us"
          },
          {
            "name": "bsbm_query11_json_us",
            "value": 12.1,
            "unit": "us"
          },
          {
            "name": "bsbm_query12_json_us",
            "value": 45.1,
            "unit": "us"
          },
          {
            "name": "lubm_q01_count_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "lubm_q02_count_us",
            "value": 618.4,
            "unit": "us"
          },
          {
            "name": "lubm_q03_count_us",
            "value": 14.7,
            "unit": "us"
          },
          {
            "name": "lubm_q14_count_us",
            "value": 5.1,
            "unit": "us"
          },
          {
            "name": "lubm_q04_count_us",
            "value": 68.3,
            "unit": "us"
          },
          {
            "name": "lubm_q05_count_us",
            "value": 28.9,
            "unit": "us"
          },
          {
            "name": "lubm_q06_count_us",
            "value": 6.5,
            "unit": "us"
          },
          {
            "name": "lubm_q07_count_us",
            "value": 30,
            "unit": "us"
          },
          {
            "name": "lubm_q08_count_us",
            "value": 2938.6,
            "unit": "us"
          },
          {
            "name": "lubm_q09_count_us",
            "value": 4073.1,
            "unit": "us"
          },
          {
            "name": "lubm_q10_count_us",
            "value": 18.8,
            "unit": "us"
          },
          {
            "name": "lubm_q11_count_us",
            "value": 10.8,
            "unit": "us"
          },
          {
            "name": "lubm_q12_count_us",
            "value": 24.1,
            "unit": "us"
          },
          {
            "name": "lubm_q13_count_us",
            "value": 18.4,
            "unit": "us"
          },
          {
            "name": "deeptax_d1000_closure_s",
            "value": 0.006,
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
            "value": 0.071,
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
            "value": 335.5,
            "unit": "us"
          },
          {
            "name": "shacl_class_nodekind_validate_us",
            "value": 314.4,
            "unit": "us"
          },
          {
            "name": "shacl_datatype_range_validate_us",
            "value": 13360.5,
            "unit": "us"
          },
          {
            "name": "shacl_node_paths_validate_us",
            "value": 6572.5,
            "unit": "us"
          },
          {
            "name": "shacl_sparql_constraint_validate_us",
            "value": 662378.3,
            "unit": "us"
          },
          {
            "name": "rdfs_infer_s",
            "value": 0.149,
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
      }
    ]
  }
}