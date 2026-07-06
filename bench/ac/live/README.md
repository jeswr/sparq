<!-- internal-stub -->
# ac-bench-live

> `publish = false` — dev-only live driver for the AC-query benchmark (bead sq-kvvcl).

This is the **live** counterpart to `bench/ac/` (the oracle-only harness, bead sq-i6du2.7).
It links `sparq-solid` and runs the real WAC/ACP authorization engine.

## What it does

- **W2 live**: builds a `PodStore` from the generator's intent table, materializes the auth
  view (WAC or ACP), and runs `query_as` for each expected decision. Hard fail-closed gate:
  any over-share (oracle=Deny but engine returns rows) exits non-zero immediately.
- **W4 concurrent**: wraps the `PodStore` in `Arc` and spawns N threads that concurrently
  call `decide_batch` + `query_as` via the `&self` read-side, asserting consistency.
- **Anti-vacuity**: a `PodStore` with no policy must deny all queries.

## Honest scope boundary

Under-share (oracle=Allow, engine=Deny) is logged as advisory, not a failure. The
`sparq-acbench` oracle evaluates all matching intents; the real WAC engine uses "nearest
ACL document wins." This divergence is tracked as bead sq-kvvcl.2. The security-critical
check (over-share) is fully fail-closed.

## Running

```
cd bench/ac && bash run.sh --smoke
```

Or invoke directly: `cargo run -- --smoke` (from `bench/ac/live/`).

## License

MIT — see the workspace root `LICENSE` file.
