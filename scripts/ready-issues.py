#!/usr/bin/env python3
# [OPUS-4.8] Issue-native orchestration: the readiness engine (replaces `bd ready` + push-frontier).
"""ready-issues.py — compute the dispatchable frontier from GitHub issues, FAIL-CLOSED.

Per the GPT-5.6 review (C1/S2), readiness requires POSITIVE, bot-attested state — never mere absence
of a quarantine label. An issue is READY iff, in priority order, ALL hold:
  * OPEN, and
  * carries `status:ready` (positive attestation the triage/trust pipeline set), and
  * carries exactly ONE valid `priority:P0..P4` (ambiguous/invalid priority → excluded), and
  * carries a `role:*` label, and
  * carries NO gate label (`needs:*`, `trust:untrusted`) and is NOT busy
    (`status:in-progress|blocked|deferred|untriaged`), and
  * has zero open blockers, and
  * none of its PACKAGES (`area:<crate>`) is already taken by an in-progress issue or an
    earlier-selected ready issue. A no-package / cross-cutting issue reserves a **global partition**
    that serializes it against ALL other work (shared lockfiles/CI/workspace configs).

Truncation fails closed (an incomplete snapshot is refused). Native GitHub dependencies + cursor
pagination are a follow-up (see FIXME); today blockers come from validated `Blocked-by: #NN` markers.
Pure `compute_ready()` is unit-tested; the CLI wraps it over `gh issue list`.
"""
import argparse
import json
import re
import subprocess
import sys

GATE_LABELS = ("needs:", "trust:untrusted")
BUSY_STATUS = {"status:in-progress", "status:blocked", "status:deferred", "status:untriaged"}
# [OPUS-4.8] an epic is a tracking umbrella (its children are the work) — never dispatchable, even
# with a full ready label-set + zero blockers. Excluded here so a worker never "implements" an epic.
NON_DISPATCHABLE = "kind:epic"
GLOBAL = "__global__"  # the cross-cutting partition (serializes against everything)
_PRIO = re.compile(r"^priority:P([0-4])$")   # only P0..P4 are valid
_PKG = re.compile(r"^area:(.+)$")
_ROLE = re.compile(r"^role:.+$")


def labels_of(issue):
    return {lb["name"] if isinstance(lb, dict) else lb for lb in issue.get("labels", [])}


def valid_priority(labels):
    """Exactly one valid priority:P0..P4 → its int; zero or multiple or out-of-range → None."""
    ps = {int(m.group(1)) for lb in labels for m in [_PRIO.match(lb)] if m}
    return next(iter(ps)) if len(ps) == 1 else None


def packages_of(labels):
    """The SET of all area:<crate> packages; empty → the serializing global partition."""
    pkgs = {m.group(1) for lb in labels for m in [_PKG.match(lb)] if m}
    return pkgs or {GLOBAL}


def has_role(labels):
    return any(_ROLE.match(lb) for lb in labels)


def is_gated(labels):
    return any(lb == g or lb.startswith(g) for lb in labels for g in GATE_LABELS)


def is_busy(labels):
    return bool(labels & BUSY_STATUS)


def compute_ready(issues, in_progress_packages=None):
    """Conflict-free, priority-ordered, FAIL-CLOSED ready frontier."""
    taken = set(in_progress_packages or ())
    # an OPEN in-progress issue reserves all its packages (derive conflict set from the issues too).
    for it in issues:
        if str(it.get("state", "OPEN")).upper() != "OPEN":
            continue
        L = labels_of(it)
        if "status:in-progress" in L:
            taken |= packages_of(L)
    cands = []
    for it in issues:
        if str(it.get("state", "OPEN")).upper() != "OPEN":
            continue
        L = labels_of(it)
        if "status:ready" not in L:          # positive attestation required
            continue
        if NON_DISPATCHABLE in L:            # epics are tracking umbrellas, not work items
            continue
        if is_gated(L) or is_busy(L):
            continue
        p = valid_priority(L)
        if p is None:                        # need exactly one valid priority
            continue
        if not has_role(L):                  # need a role
            continue
        if int(it.get("open_blockers", 0)) > 0:
            continue
        cands.append((p, it.get("number", 0), it, packages_of(L)))
    cands.sort(key=lambda c: (c[0], c[1]))   # priority then number (deterministic)
    ready = []
    for _p, _n, it, pkgs in cands:
        if GLOBAL in taken:                  # cross-cutting work in flight → nothing else co-runs
            break
        if pkgs & taken:                     # package conflict
            continue
        if GLOBAL in pkgs and taken:         # cross-cutting can't co-run with any package in flight
            continue
        taken |= pkgs
        ready.append(it)
    return ready


def _self_test():
    def iss(n, labels, blk=0, state="OPEN"):
        return {"number": n, "state": state, "labels": labels, "open_blockers": blk}

    R = ["status:ready", "role:impl"]
    F = [
        iss(1, R + ["priority:P2", "area:sparq-core"]),
        iss(2, R + ["priority:P0", "area:sparq-core"]),
        iss(3, R + ["priority:P1", "area:sparq-engine"]),
        iss(4, R + ["priority:P1", "area:sparq-engine", "needs:user"]),         # gated
        iss(5, R + ["priority:P1", "area:sparq-zk"], blk=2),                     # blocked
        iss(6, R + ["priority:P0", "area:sparq-hdt"], state="CLOSED"),           # closed
        iss(7, R + ["priority:P1", "trust:untrusted", "area:sparq-geo"]),        # untrusted
        iss(8, ["priority:P3", "role:impl", "area:sparq-text"]),                 # not status:ready
        iss(9, R + ["priority:P1", "priority:P2", "area:sparq-sim"]),            # ambiguous priority
        iss(10, R + ["priority:P1", "area:sparq-fedplan", "status:in-progress"]),# in-progress fedplan
        iss(11, R + ["priority:P4"]),                                            # no package -> global
        iss(12, R + ["priority:P1", "area:sparq-hdt"]),                          # hdt (free)
        iss(13, R + ["priority:P0", "area:sparq-text", "kind:epic"]),            # epic -> excluded
    ]
    ok = True

    def check(name, got, want):
        nonlocal ok
        good = got == want
        ok = ok and good
        print(f"  {'ok  ' if good else 'FAIL'} {name}: {got} (want {want})")

    ready = compute_ready(F)
    # eligible: 1,2,3,12 (+11 global). 4 gated, 5 blocked, 6 closed, 7 untrusted, 8 no-ready,
    # 9 ambiguous-prio, 10 in-progress, 13 epic (kind:epic → excluded despite a P0 ready label-set).
    # Order by prio: #2(P0 core) -> #3(P1 engine) -> #12(P1 hdt) -> #11(P4 global). core taken by #2
    # so #1(P2 core) excluded. #11 global: only selectable if nothing taken -> excluded.
    check("ready order", [i["number"] for i in ready], [2, 3, 12])
    # a P0 epic with an otherwise-perfect ready label-set must NOT dispatch (tracking umbrella):
    check("epic excluded", 13 in [i["number"] for i in ready], False)
    # a lone global issue with an empty board is selectable:
    check("lone global", [i["number"] for i in compute_ready([iss(11, R + ["priority:P4"])])], [11])
    # global blocks everything else:
    g = compute_ready([iss(11, R + ["priority:P0"]), iss(12, R + ["priority:P1", "area:sparq-hdt"])])
    check("global serializes", [i["number"] for i in g], [11])
    check("valid_priority single", valid_priority({"priority:P0"}), 0)
    check("valid_priority ambiguous", valid_priority({"priority:P1", "priority:P2"}), None)
    check("valid_priority out-of-range", valid_priority({"priority:P7"}), None)
    check("packages multi", packages_of({"area:a", "area:b"}), {"a", "b"})
    check("packages none->global", packages_of({"role:impl"}), {GLOBAL})
    check("untriaged is busy", is_busy({"status:untriaged"}), True)
    print("ready-issues self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


def _fetch(repo, limit=1000):
    out = subprocess.run(
        ["gh", "issue", "list", "-R", repo, "--state", "open", "--limit", str(limit),
         "--json", "number,labels,body,state"],
        capture_output=True, text=True, check=True).stdout
    raw = json.loads(out)
    if len(raw) >= limit:  # FIXME: replace with cursor pagination + native deps (GraphQL)
        raise SystemExit(f"refusing: fetched {len(raw)} >= limit {limit} — snapshot may be truncated "
                         "(fail-closed). Implement pagination before raising the limit.")
    open_numbers = {i["number"] for i in raw}
    issues = []
    for i in raw:
        blockers = re.findall(r"[Bb]locked-by:\s*#(\d+)", i.get("body") or "")
        open_blk = sum(1 for b in blockers if int(b) in open_numbers)
        issues.append({"number": i["number"], "state": i["state"],
                       "labels": i["labels"], "open_blockers": open_blk})
    return issues


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default="sparq-org/sparq")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return _self_test()
    for it in compute_ready(_fetch(args.repo)):
        L = labels_of(it)
        print(f"P{valid_priority(L)}  #{it['number']:5}  {sorted(packages_of(L))}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
