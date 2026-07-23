#!/usr/bin/env python3
# [FABLE-5] sq-6vshe.5 (PR #2521 review round 4) — structural guard for the `wasm`
# rust-cache family: cache-prime.yml's `prime-wasm` job must mirror ci.yml's `wasm`
# job cargo closure EXACTLY.
#
# Why: both jobs share `shared-key: "wasm"`, and Swatinem/rust-cache prunes workspace
# members before saving — the cache carries DEPENDENCY artifacts only. Cargo resolves
# third-party dep features PER invocation state, so a (build|clippy, package,
# feature-set) state the target job compiles but the primer omits restores cold even
# when a sibling state was primed. The historical bug this pins: the primer built
# only sparq-wasm default + shacl,jsonld while the target job also compiled separate
# shacl / jsonld / policy / explain-json states and four more wasm crates.
#
# Rule enforced: the set of cargo build/clippy invocation states (mode, package,
# feature-set, --all-targets) against --target wasm32-unknown-unknown must be EQUAL
# between the two jobs. Missing primer states leave the fresh-key/stable-bump hole
# open; stale primer extras burn runner time and cache budget on artifacts the real
# job never restores. `-D warnings` (everything after a bare ` -- `) is ignored —
# the primer deliberately drops it (cache builder, not a second lint gate).
# wasm-pack / cargo-tree / non-wasm32 lines are out of scope.
#
# Deterministic: YAML parse only — no network, no build. Needs PyYAML (hash-pinned
# in .github/requirements/docs-quality.txt where this runs).

import argparse
import re
import sys

CI_WORKFLOW = ".github/workflows/ci.yml"
PRIME_WORKFLOW = ".github/workflows/cache-prime.yml"
TARGET_JOB = "wasm"
PRIME_JOB = "prime-wasm"
WASM_TARGET = "wasm32-unknown-unknown"

_CARGO_RE = re.compile(r"\bcargo\s+(build|clippy)\b")


def cargo_wasm_states(job):
    """Extract the set of (mode, package, features, all_targets) states for every
    `cargo build`/`cargo clippy` invocation against the wasm32 target in a job."""
    states = set()
    for step in job.get("steps") or []:
        run = step.get("run")
        if not isinstance(run, str):
            continue
        # Join shell line-continuations so a wrapped invocation parses whole.
        for line in run.replace("\\\n", " ").splitlines():
            m = _CARGO_RE.search(line)
            if not m:
                continue
            # Drop trailing-args after a bare ` -- ` (e.g. `-- -D warnings`).
            args = line[m.end():].split(" -- ")[0].split()
            pkg = None
            feats = frozenset()
            target = None
            all_targets = False
            i = 0
            while i < len(args):
                tok = args[i]
                if tok in ("-p", "--package") and i + 1 < len(args):
                    i += 1
                    pkg = args[i]
                elif tok == "--features" and i + 1 < len(args):
                    i += 1
                    feats = frozenset(args[i].split(","))
                elif tok == "--target" and i + 1 < len(args):
                    i += 1
                    target = args[i]
                elif tok == "--all-targets":
                    all_targets = True
                i += 1
            if target != WASM_TARGET:
                continue
            states.add((m.group(1), pkg, feats, all_targets))
    return states


def fmt_state(state):
    mode, pkg, feats, all_targets = state
    parts = ["cargo", mode, "-p", str(pkg)]
    if all_targets:
        parts.append("--all-targets")
    if feats:
        parts.append("--features " + ",".join(sorted(feats)))
    return " ".join(parts)


def load_job(yaml, path, job_id):
    with open(path, "r", encoding="utf-8") as fh:
        doc = yaml.safe_load(fh)
    job = (doc.get("jobs") or {}).get(job_id)
    if job is None:
        print(
            "check-cache-prime-wasm: job '%s' not found in %s — if it was renamed, "
            "update this check's constants" % (job_id, path),
            file=sys.stderr,
        )
        sys.exit(1)
    return job


def compare(target_states, prime_states):
    """Return (missing, extra): target states the primer omits / primer-only states.
    Sorted by rendered form — frozenset's `<` is subset (a partial order), so sorting
    the raw tuples would be nondeterministic."""
    return (sorted(target_states - prime_states, key=fmt_state),
            sorted(prime_states - target_states, key=fmt_state))


def check_repo(yaml):
    target = load_job(yaml, CI_WORKFLOW, TARGET_JOB)
    prime = load_job(yaml, PRIME_WORKFLOW, PRIME_JOB)
    target_states = cargo_wasm_states(target)
    prime_states = cargo_wasm_states(prime)
    if not target_states:
        print(
            "check-cache-prime-wasm: no wasm32 cargo invocations found in ci.yml job "
            "'%s' — parser or workflow drift, refusing to pass vacuously" % TARGET_JOB,
            file=sys.stderr,
        )
        return 1
    missing, extra = compare(target_states, prime_states)
    ok = True
    for state in missing:
        print(
            "check-cache-prime-wasm: MISSING from %s '%s': %s — this state's unique "
            "third-party deps restore COLD in ci.yml's wasm job"
            % (PRIME_WORKFLOW, PRIME_JOB, fmt_state(state)),
            file=sys.stderr,
        )
        ok = False
    for state in extra:
        print(
            "check-cache-prime-wasm: STALE extra in %s '%s': %s — ci.yml's wasm job "
            "never compiles this state; drop it (runner time + cache budget)"
            % (PRIME_WORKFLOW, PRIME_JOB, fmt_state(state)),
            file=sys.stderr,
        )
        ok = False
    if ok:
        print(
            "check-cache-prime-wasm: OK — prime-wasm mirrors all %d wasm32 cargo "
            "states of ci.yml's wasm job" % len(target_states)
        )
        return 0
    return 1


# ---- hermetic self-test (mirrors the coverage-gate.py --self-test pattern) ----

_SELF_TEST_TARGET = """
jobs:
  wasm:
    steps:
      - run: cargo build -p crate-a --target wasm32-unknown-unknown
      - run: cargo build -p crate-a --target wasm32-unknown-unknown --features x,y
      - run: cargo clippy -p crate-a --target wasm32-unknown-unknown --all-targets --features x -- -D warnings
      - run: cargo clippy -p crate-b --target wasm32-unknown-unknown -- -D warnings
      - run: cargo build -p crate-a --target x86_64-unknown-linux-gnu
      - run: wasm-pack test --node
      - working-directory: crates/crate-a
        run: |
          if cargo tree -p crate-a --target wasm32-unknown-unknown -e normal | grep -q z; then
            exit 1
          fi
"""

_SELF_TEST_PRIME = """
jobs:
  prime-wasm:
    steps:
      - run: |
          cargo build -p crate-a --target wasm32-unknown-unknown
          cargo build -p crate-a --target wasm32-unknown-unknown --features y,x
      - run: cargo clippy -p crate-a --target wasm32-unknown-unknown --all-targets --features x
      - run: cargo clippy -p crate-b --target wasm32-unknown-unknown
"""


def self_test(yaml):
    target = cargo_wasm_states(yaml.safe_load(_SELF_TEST_TARGET)["jobs"]["wasm"])
    prime = cargo_wasm_states(yaml.safe_load(_SELF_TEST_PRIME)["jobs"]["prime-wasm"])

    # Extraction: host-triple, wasm-pack, and cargo-tree lines are ignored; feature
    # order and `-- -D warnings` do not affect state identity.
    expect = {
        ("build", "crate-a", frozenset(), False),
        ("build", "crate-a", frozenset({"x", "y"}), False),
        ("clippy", "crate-a", frozenset({"x"}), True),
        ("clippy", "crate-b", frozenset(), False),
    }
    assert target == expect, "extraction mismatch: %r" % (target,)
    assert compare(target, prime) == ([], []), "equal closures must pass"

    # A target state the primer omits is reported as missing.
    dropped = prime - {("clippy", "crate-a", frozenset({"x"}), True)}
    missing, extra = compare(target, dropped)
    assert missing == [("clippy", "crate-a", frozenset({"x"}), True)] and extra == []

    # A primer-only state is reported as stale (all-targets is part of identity).
    padded = prime | {("clippy", "crate-b", frozenset(), True)}
    missing, extra = compare(target, padded)
    assert missing == [] and extra == [("clippy", "crate-b", frozenset(), True)]

    print("check-cache-prime-wasm: self-test OK (4 cases)")
    return 0


def main(argv):
    ap = argparse.ArgumentParser(
        description="Assert cache-prime's prime-wasm job mirrors ci.yml's wasm job "
        "cargo closure exactly."
    )
    ap.add_argument("--self-test", action="store_true",
                    help="run the hermetic parser/comparison self-test and exit")
    args = ap.parse_args(argv)
    try:
        import yaml
    except ImportError:
        print("check-cache-prime-wasm: PyYAML is required (pip install "
              "--require-hashes -r .github/requirements/docs-quality.txt)",
              file=sys.stderr)
        return 1
    if args.self_test:
        return self_test(yaml)
    return check_repo(yaml)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
