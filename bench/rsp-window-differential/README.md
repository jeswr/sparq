<!-- [GPT-5.6] sq-no6iy — RSP window-content differential harness. -->
# RSP window-semantics differential

This standalone bench replays one fixed, timestamped SRBench-shaped stream through
the public `sparq-rsp` multi-window API. At every report timestamp it compares the
complete emitted solution window with a pinned RSP4J/YASPER event-time capture. Rows
are sorted into a canonical multiset before comparison; a row-value change therefore
fails even when the row count is unchanged.

Run `bench/rsp-window-differential/run.sh` from anywhere. Set
`RSP_WINDOW_ENVELOPE` to retain the JSON envelope. The runner records advisory
streaming-emit latency only after all content comparisons pass, and includes the
different time models as machine-readable envelope metadata.

## Honest gap table

| Surface | Verdict | Gap |
|---|---|---|
| Event-time tumbling window content | Compared | Complete canonical multisets are pinned per report timestamp. |
| Sliding and count windows | Not compared | The captured RSP4J replay covers the shared tumbling-window dialect only. |
| Aggregate output terms | Not compared | This fixture uses a projection join so datatype and binding equality are directly portable. |
| Raw throughput | Not comparable | sparq-rsp is clock-free; deployed RSP4J uses wall-clock service scheduling. |

The committed capture is an oracle fixture, not an RSP4J runtime dependency. The
existing `bench/rsp` gather harness documents how the pinned RSP4J/YASPER revision is
built and driven; this directory deliberately keeps the per-commit gate offline and
deterministic.
