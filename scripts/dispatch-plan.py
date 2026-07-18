#!/usr/bin/env python3
# [OPUS-4.8] Issue-native orchestration: the pure dispatch PLANNER (Phase 3-A) — dry-run / read-only.
"""dispatch-plan.py — compose the readiness engine + route resolver into a dispatch plan.

This is the PURE, read-only planner. It walks the conflict-free, priority-ordered ready frontier
produced by `ready-issues.compute_ready`, resolves each issue's route via `route-resolve.resolve`
against `orchestration/routing.toml`, reserves the issue's first package, and emits a plan row per
issue: {number, priority, package, role, model_chain, agent, escalate}.

It is DELIBERATELY dry-run: it does NOT claim an account and does NOT trigger a worker. Those are the
two credential-gated seams (see `claim_and_dispatch`) that require the private account registry's
claim step and the automation identity — both pending the maintainer's broker/token-placement
sign-off. Live mode (`--dry-run`) stops at "would dispatch"; it never crosses that boundary.

Fail-closed and honest (per the Phase-1/2 review posture): if an issue's route cannot be resolved to
a concrete role, the plan row is flagged with role=None/agent=None rather than guessed. `plan_dispatch`
is pure (no network, no side effects) and unit-tested by `--self-test`.
"""
import argparse
import os
import sys

# The foundation scripts live alongside this one; import their pure functions (they hyphenate their
# filenames, so load by path rather than `import ready-issues`).
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import importlib.util


def _load(modname, filename):
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), filename)
    spec = importlib.util.spec_from_file_location(modname, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


_ready = _load("ready_issues", "ready-issues.py")
_route = _load("route_resolve", "route-resolve.py")

compute_ready = _ready.compute_ready
packages_of = _ready.packages_of
labels_of = _ready.labels_of
valid_priority = _ready.valid_priority
resolve = _route.resolve

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib


def _role_of(labels):
    """The issue's declared role from its `role:<r>` label, or None. Deterministic (sorted)."""
    for lb in sorted(labels):
        if lb.startswith("role:"):
            return lb[5:]
    return None


def plan_dispatch(ready_issues, routing_doc):
    """Compose the ready frontier + routing into a dispatch plan. PURE — no I/O, no side effects.

    `ready_issues` is the already priority-ordered, conflict-free output of `compute_ready` (each a
    dict with at least `number` + `labels`). Returns a list of plan rows, one per ready issue:
      {number, priority, package, role, model_chain, agent, escalate}
    `package` is the FIRST (sorted, deterministic) of the issue's packages — the partition the ready
    engine already reserved for it. If routing falls through to [defaults] because the issue carries
    no `role:` label, the row is flagged unresolved: role=None, agent=None (fail-closed, never guessed).
    """
    plan = []
    for it in ready_issues:
        labels = labels_of(it)
        role = _role_of(labels)
        package = sorted(packages_of(labels))[0]
        model_chain, agent, escalate = resolve(labels, routing_doc)
        if role is None:
            # No declared role → the resolver returned [defaults]; do NOT guess an agent/role.
            row = {
                "number": it.get("number", 0),
                "priority": valid_priority(labels),
                "package": package,
                "role": None,
                "model_chain": [],
                "agent": None,
                "escalate": False,
            }
        else:
            row = {
                "number": it.get("number", 0),
                "priority": valid_priority(labels),
                "package": package,
                "role": role,
                "model_chain": list(model_chain),
                "agent": agent,
                "escalate": bool(escalate),
            }
        plan.append(row)
    return plan


def claim_and_dispatch(entry):
    """CREDENTIAL-GATED SEAM — the broker/worker boundary. The planner NEVER calls this.

    Turning a plan row into a live worker requires TWO capabilities this public planner does not (and
    must not) hold:

      1. The account claim. `select-and-claim.py` lives in the PRIVATE `agent-account-registry` repo
         (`jeswr/agent-account-registry`). It consumes the row's `model_chain` to lease the first
         AVAILABLE account (respecting per-account limits, the lease, and cache affinity), returning a
         restored credential. That claim is gated on the maintainer's token-placement / broker
         sign-off.
      2. The worker trigger. Dispatching the claimed run is a GitHub Actions worker job that runs under
         the automation identity `REGISTRY_ADMIN_TOKEN` (the registry-admin secret), NOT any human PAT.

    Both are deliberately absent here so the planner stays public, pure, and safe to run anywhere.
    When the broker sign-off + automation identity land, THIS function is the single place to wire
    them; every caller upstream stops at "would dispatch".
    """
    raise NotImplementedError(
        "claim_and_dispatch is the credential-gated broker/worker seam: it needs the private "
        "agent-account-registry claim (select-and-claim.py, gated on the maintainer's "
        "token-placement/broker sign-off) and the worker Actions job under REGISTRY_ADMIN_TOKEN "
        "(the automation identity). The dry-run planner never calls it."
    )


# ---------------------------------------------------------------------------------------------------
# Self-test: fixtures exercising the composition end-to-end against the REAL routing.toml.
def _routing_doc():
    here = os.path.dirname(os.path.abspath(__file__))
    toml = os.path.join(os.path.dirname(here), "orchestration", "routing.toml")
    with open(toml, "rb") as fh:
        return tomllib.load(fh)


def _self_test():
    doc = _routing_doc()
    ok = True

    def chk(name, got, want):
        nonlocal ok
        good = got == want
        ok = ok and good
        print(f"  {'ok  ' if good else 'FAIL'} {name}: {got} (want {want})")

    def iss(n, labels, blk=0, state="OPEN"):
        return {"number": n, "state": state, "labels": labels, "open_blockers": blk}

    R = ["status:ready"]

    # --- Fixture: an impl issue → fable-led chain, no escalate ------------------------------------
    impl = compute_ready([iss(1, R + ["priority:P1", "role:impl", "area:sparq-core"])])
    p_impl = plan_dispatch(impl, doc)
    chk("impl -> single row", len(p_impl), 1)
    row = p_impl[0]
    chk("impl row", (row["role"], row["model_chain"][0], row["agent"], row["escalate"]),
        ("impl", "fable", "sparq-rust-impl", False))
    chk("impl package", row["package"], "sparq-core")
    chk("impl priority", row["priority"], 1)

    # --- Fixture: a security/zk issue → opus + escalate (security override beats role) -----------
    sec = compute_ready([iss(2, R + ["priority:P0", "role:impl", "area:sparq-zk"])])
    p_sec = plan_dispatch(sec, doc)
    row = p_sec[0]
    chk("zk -> opus", row["model_chain"], ["opus"])
    chk("zk -> reviewer/escalate", (row["agent"], row["escalate"]), ("sparq-reviewer", True))
    chk("zk role stays declared", row["role"], "impl")

    # --- Fixture: a docs issue → its route (haiku-led) -------------------------------------------
    docs = compute_ready([iss(3, R + ["priority:P2", "role:docs", "area:sparq-docs"])])
    p_docs = plan_dispatch(docs, doc)
    row = p_docs[0]
    chk("docs -> haiku", row["model_chain"][0], "haiku")
    chk("docs -> sparq-docs", row["agent"], "sparq-docs")

    # --- Fixture: a ci/infra issue → frontier-only chain (standing rule 2026-07-17) --------------
    # No sub-frontier model (sonnet/haiku) in the plan row's chain: the registry claim step can
    # only serve a frontier account or DEFER the item to the next tick — degradation to a cheaper
    # authoring tier is impossible by construction (see routing-validate's frontier-floor check).
    ci = compute_ready([iss(8, R + ["priority:P1", "role:ci", "area:ci"])])
    row = plan_dispatch(ci, doc)[0]
    chk("ci -> frontier-only row", (row["role"], row["model_chain"], row["agent"], row["escalate"]),
        ("ci", ["fable", "sol"], "sparq-ci-infra", False))
    chk("ci row has no sub-frontier tier", sorted(set(row["model_chain"]) & {"sonnet", "haiku"}), [])

    # --- Fixture: package-conflict pair → only the higher-priority one is planned ----------------
    pair = compute_ready([
        iss(4, R + ["priority:P2", "role:impl", "area:sparq-engine"]),   # lower priority
        iss(5, R + ["priority:P0", "role:impl", "area:sparq-engine"]),   # higher priority (wins)
    ])
    p_pair = plan_dispatch(pair, doc)
    chk("conflict -> one row", len(p_pair), 1)
    chk("conflict -> higher prio kept", p_pair[0]["number"], 5)

    # --- Fixture: an empty frontier → empty plan -------------------------------------------------
    chk("empty frontier -> empty plan", plan_dispatch([], doc), [])
    chk("no ready -> empty",
        plan_dispatch(compute_ready([iss(6, ["priority:P1", "role:impl", "area:x"])]), doc), [])

    # --- Fixture: an issue with NO declared role → fail-closed (role/agent None) -----------------
    # (compute_ready requires a role, so bypass it and feed plan_dispatch a roleless issue directly
    #  to prove the resolver-fallthrough is flagged, never guessed.)
    p_norole = plan_dispatch([iss(7, ["priority:P1", "area:sparq-core"])], doc)
    row = p_norole[0]
    chk("no-role -> flagged", (row["role"], row["agent"], row["model_chain"]), (None, None, []))

    print("dispatch-plan self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


# ---------------------------------------------------------------------------------------------------
# Dry-run CLI: fetch live ready issues (reusing the readiness engine's live path), plan, print.
def _print_table(plan):
    if not plan:
        print("(no ready issues — the dispatch plan is empty; nothing to dispatch)")
        return
    cols = ["number", "priority", "package", "role", "model_chain", "agent", "escalate"]

    def cell(row, c):
        v = row[c]
        if c == "number":
            return f"#{v}"
        if c == "priority":
            return f"P{v}" if v is not None else "P?"
        if c == "model_chain":
            return ">".join(v) if v else "-"
        return str(v) if v is not None else "-"

    widths = {c: max(len(c), *(len(cell(r, c)) for r in plan)) for c in cols}
    header = "  ".join(c.ljust(widths[c]) for c in cols)
    print(header)
    print("  ".join("-" * widths[c] for c in cols))
    for r in plan:
        print("  ".join(cell(r, c).ljust(widths[c]) for c in cols))
    print(f"\n{len(plan)} issue(s) would be dispatched (dry-run — no account claimed, no worker "
          "triggered).")


def main():
    ap = argparse.ArgumentParser(description="Pure dispatch planner (dry-run) for issue-native "
                                             "orchestration.")
    ap.add_argument("--repo", default="sparq-org/sparq")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--dry-run", action="store_true",
                    help="fetch live ready issues, build + print the plan (read-only; never "
                         "dispatches)")
    args = ap.parse_args()
    if args.self_test:
        return _self_test()
    if args.dry_run:
        issues = _ready._fetch(args.repo)          # reuse the readiness engine's live `gh` fetch
        ready = compute_ready(issues)
        plan = plan_dispatch(ready, _routing_doc())
        _print_table(plan)
        return 0
    ap.print_help()
    return 0


if __name__ == "__main__":
    sys.exit(main())
