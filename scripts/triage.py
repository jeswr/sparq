#!/usr/bin/env python3
# [OPUS-4.8] Issue-native orchestration: static triage (assign role/priority/package + ready-state).
"""triage.py — the deterministic, no-LLM part of issue triage.

Given an issue's labels + type, decide the labels to ADD/REMOVE and whether it is triage-complete:
  * role     — from a `kind:*` label or the bead/issue type (feature/bug->impl, docs->docs, ...).
  * priority — kept if a valid single `priority:P0..P4` is present; otherwise triage is incomplete.
  * package  — the existing `area:<crate>` labels are the package (kept as-is); a no-package issue is
               cross-cutting (handled by the readiness engine's global partition), not incomplete.
  * ready    — an issue is `status:ready` iff it has a valid single priority AND a role AND is not
               gated/untrusted. Otherwise it becomes `status:untriaged` for the retriage cron / an
               LLM pass (a model call is only needed when role/priority cannot be derived statically).

Fail-closed: ambiguity or missing role/priority yields `status:untriaged`, never `status:ready`.
Pure `triage()` is unit-tested; the workflow applies the returned label delta.
"""
import re
import sys

ROLE_BY_KIND = {"docs": "docs", "design": "research", "research": "research", "perf": "perf",
                "test": "impl", "ci": "ci", "site": "site", "security": "soundness"}
ROLE_BY_TYPE = {"feature": "impl", "bug": "impl", "task": "impl", "chore": "ci",
                "spike": "research", "epic": "impl"}
SEC_KEYWORDS = ("zk", "mpc", "reasoner", "crypto", "auth", "e2ee")
_PRIO = re.compile(r"^priority:P([0-4])$")


def _valid_priority(labels):
    ps = {m.group(1) for lb in labels for m in [_PRIO.match(lb)] if m}
    return len(ps) == 1


def _role(labels, issue_type):
    # a security-surface keyword forces the soundness lane regardless of kind/type
    if any(k in lb for lb in labels for k in SEC_KEYWORDS):
        return "soundness"
    for lb in labels:
        if lb.startswith("kind:") and lb[5:] in ROLE_BY_KIND:
            return ROLE_BY_KIND[lb[5:]]
    return ROLE_BY_TYPE.get(issue_type)


def triage(labels, issue_type="task", trusted=True):
    """Return {add:set, remove:set, ready:bool, role:str|None}. If not trusted, triage is a no-op
    (the trust layer quarantines/notifies; content is never inspected here)."""
    labels = set(labels)
    if not trusted or "trust:untrusted" in labels:
        return {"add": set(), "remove": set(), "ready": False, "role": None}
    role = _role(labels, issue_type)
    add, remove = set(), set()
    if role:
        add.add(f"role:{role}")
    ready = bool(role) and _valid_priority(labels) and "needs:user" not in labels
    if ready:
        add.add("status:ready")
        remove.add("status:untriaged")
    else:
        add.add("status:untriaged")   # fail-closed: not dispatchable until complete
        remove.add("status:ready")
    return {"add": add - labels, "remove": remove & labels, "ready": ready, "role": role}


def _self_test():
    ok = True

    def chk(n, got, want):
        nonlocal ok
        good = got == want
        ok = ok and good
        print(f"  {'ok  ' if good else 'FAIL'} {n}: {got} (want {want})")

    # complete: priority + derivable role -> ready
    r = triage(["priority:P1", "area:sparq-core"], "feature")
    chk("feature ready", (r["ready"], "role:impl" in r["add"], "status:ready" in r["add"]), (True, True, True))
    # missing priority -> untriaged
    r = triage(["area:sparq-core"], "feature")
    chk("no priority -> untriaged", (r["ready"], "status:untriaged" in r["add"]), (False, True))
    # ambiguous priority -> untriaged
    r = triage(["priority:P1", "priority:P2"], "feature")
    chk("ambiguous priority", r["ready"], False)
    # docs kind -> role:docs
    chk("docs role", triage(["priority:P3", "kind:docs"], "task")["role"], "docs")
    # security keyword forces soundness
    chk("sec soundness", triage(["priority:P1", "area:sparq-zk"], "feature")["role"], "soundness")
    # needs:user -> not ready
    chk("needs:user gated", triage(["priority:P1", "kind:docs", "needs:user"], "task")["ready"], False)
    # untrusted -> no-op
    chk("untrusted no-op", triage(["priority:P1", "trust:untrusted"], "feature"), {"add": set(), "remove": set(), "ready": False, "role": None})
    print("triage self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


def main():
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--labels", default="", help="comma-separated current labels")
    ap.add_argument("--type", default="task")
    ap.add_argument("--untrusted", action="store_true")
    a = ap.parse_args()
    if a.self_test:
        return _self_test()
    labels = [x for x in a.labels.split(",") if x.strip()]
    r = triage(labels, a.type, trusted=not a.untrusted)
    print("ADD: " + " ".join(sorted(r["add"])))
    print("REMOVE: " + " ".join(sorted(r["remove"])))
    return 0


if __name__ == "__main__":
    sys.exit(main())
