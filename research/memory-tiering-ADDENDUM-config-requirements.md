# ADDENDUM from the orchestrator (user directive, 2026-06-12) — configurability + resource-awareness requirements

> For the memory-tiering agent: if you see this before writing your final report, fold these
> into the "proposed design" sections. If you've already finished, the orchestrator will hand
> this to the implementation wave instead — no need to redo measurements.

The user has confirmed index-dropping + usage-driven compression as direction, with two
binding requirements on the eventual design:

1. **Build-time configurability.** The adaptive behaviors (perm dropping, hot/cold compression
   tiering, access tracking) must be toggleable at build time — cargo features in the existing
   house style (cf. `parallel`, `mmap`, `cs-planner`, `compact-index`: additive, off-by-default
   for new behavior or on-by-default only when measured zero-cost when idle). "Sensible
   defaults": the static 6-perm raw behavior must remain the out-of-the-box default unless the
   measurements show the adaptive mode is genuinely free when not under pressure. Runtime
   config (thresholds, policies) layered on top of the build-time gate.

2. **Resource-awareness.** Tiering/dropping decisions must be able to account for AVAILABLE
   MEMORY and AVAILABLE DISK SPACE (dropping a perm to disk needs disk headroom; compression
   targets should derive from a memory budget). Both configurable with sensible defaults
   (e.g. default memory budget = fraction of detected system RAM; default disk floor =
   refuse to spill below X GB free). Relevant to your investigation NOW if cheap: note which
   platform APIs you'd use for detection (sysinfo crate? statvfs? macOS/Linux/wasm differences
   — wasm has neither, so the build-time gate must compile it out there), and whether the
   benchmarking harness should itself set explicit budgets for reproducibility (the user
   expects the budget mechanism to help with benchmarking).

These do not change the measurement plan — verdicts first, design second.
