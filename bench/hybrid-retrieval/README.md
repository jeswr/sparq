<!-- internal-stub -->
<!-- [GPT-5.6] sq-ljp19: pinned bench-only RRF reference. -->
# Hybrid-retrieval RRF reference

This bench-only harness fuses pinned dense, sparse, and structural ranking lists
with deterministic weighted reciprocal-rank fusion. It does not invoke the
engine, a model, or the network.

Run the pinned result-equivalence and claim gates:

```sh
bash bench/hybrid-retrieval/run.sh
```

`analyze.py` truncates each arm to `top_k`, adds `weight / (rrf_k + rank)`, and
uses document ID as the stable final tie-breaker. Its report retains each arm's
rank provenance, overlap at k, fused-versus-arm rank deltas, and relevant-at-k
ablation totals. A configured `lift` claim fails unless fusion beats the best
single arm on the pinned relevance labels.
