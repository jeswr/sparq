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
  * none of its PACKAGES (`area:<crate>`) is already taken by an active open PR, an in-progress
    issue, or an earlier-selected ready issue. Human-parked artifacts (`needs:user`,
    `review:needs-user`, `status:blocked`) reserve nothing. A no-package / cross-cutting issue
    reserves a **global partition** that serializes it against ALL other work
    (shared lockfiles/CI/workspace configs).

The snapshot uses real cursor pagination (`gh api --paginate`) with an explicit fail-closed
ceiling. Open blockers are the UNION of GitHub's NATIVE issue dependencies and the legacy
validated `Blocked-by: #NN` body markers — see `open_blocker_count`. Pure `compute_ready()` is
unit-tested; the CLI wraps it over the paginated fetch.
"""
import argparse
import json
import os
import re
import subprocess
import sys

GATE_LABELS = ("needs:", "trust:untrusted")
# [OPUS-5] `status:in-progress-review` was MISSING here while the registry's dispatch.yml keeps
# such rows in `ready_input` precisely because it believes compute_ready() will (a) never select
# them and (b) still let them RESERVE their package. Neither held: the label is not a gate label,
# so an in-progress-review issue that also carried `status:ready` was SELECTABLE, and the reserve
# branch below only fired on `status:in-progress`, so its crate was left free for a second
# dispatch. Both halves fail OPEN into a double-dispatch; see IN_FLIGHT_STATUS below.
BUSY_STATUS = {"status:in-progress", "status:in-progress-review", "status:blocked",
               "status:deferred", "status:untriaged"}
# The statuses that make an OPEN ISSUE in-flight work occupying its area (not merely excluded).
IN_FLIGHT_STATUS = {"status:in-progress", "status:in-progress-review"}
# [GPT-5.6] Parked is not in-flight: these terminal, human-owned snapshot artifacts cannot
# advance autonomously, so they must not reserve an area indefinitely. Removing the label in a
# later snapshot restores occupancy immediately; there is no remembered park state.
PARKED_AREA_LABELS = {"needs:user", "review:needs-user", "status:blocked"}
# [OPUS-4.8] an epic is a tracking umbrella (its children are the work) — never dispatchable, even
# with a full ready label-set + zero blockers. Excluded here so a worker never "implements" an epic.
NON_DISPATCHABLE = "kind:epic"
GLOBAL = "__global__"  # the cross-cutting partition (serializes against everything)
_PRIO = re.compile(r"^priority:P([0-4])$")   # only P0..P4 are valid
_PKG = re.compile(r"^area:(.+)$")
_ROLE = re.compile(r"^role:.+$")

# ---------------------------------------------------------------------------
# Partition-key algebra (sparq#4336) — CONTAINMENT-aware, TOTAL.
#
# [OPUS-5] The under-serialisation defect this replaces: `conflict()` compared partition keys by
# EXACT-STRING set overlap, so a key naming a region INSIDE a crate never overlapped its parent
# crate. Five such labels already exist in production — `sparq-server-http`, `sparq-core-nt-dict`,
# `sparq-core-store`, `sparq-engine-exec`, `sparq-conformance-floors` — so an `area:sparq-server`
# issue and an `area:sparq-server-http` issue entered the frontier in the SAME tick, and an
# `area:sparq-server-http` issue entered despite an open PR holding `area:sparq-server`. Two
# workers in one crate with no lock between them. Measured base rate for that pair sharing a file:
# 57.1% of same-crate 24h PR pairs (research/crate-region-parallelism.md §4). Textual collisions
# surface as merge conflicts; SEMANTIC ones (both compile, both pass, together broken) are
# invisible to git and reach the merge-group gate at the cost of a dequeue plus a batch bisect.
#
# Under-serialisation is the CORRUPTING direction. Over-serialisation only costs delay. So the
# mapping below is TOTAL and biased to over-reserve: every key resolves to the coarsest partition
# that could contain it, and a key we cannot place at all resolves to GLOBAL.
_SEP = "-"                       # the hierarchy separator inside an `area:` key
_PARTITION_MEMO = {}             # key -> path, for the default (workspace-derived) root set
_WORKSPACE_ROOTS = None          # lazily scanned; None = not yet read


def _repo_root():
    return os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def workspace_roots(repo_root=None):
    """The RECOGNISED partition roots, READ FROM THE REPOSITORY TREE — deliberately not a table.

    A name is a recognised root iff the workspace really contains it as a partition: a crate
    directory under `crates/`, or a top-level repository directory. The tree is the only authority
    that can tell `sparq-engine-serialize` (a REAL sibling crate, its own partition) apart from
    `sparq-engine-exec` (a REGION inside `sparq-engine`) — as strings the two are identical in
    shape, and a hand-written table of which is which is exactly what goes stale. Reading the tree
    means a crate added next month registers itself with no code change, and a region label
    invented next month resolves to its crate with no code change.

    The registry's `dispatch.yml` CLONES this repo and runs this script, so the same tree is
    present there; `--dump-partitions` exports the resolved mapping for a parity fixture.
    """
    global _WORKSPACE_ROOTS
    if repo_root is None and _WORKSPACE_ROOTS is not None:
        return _WORKSPACE_ROOTS
    base = repo_root if repo_root is not None else _repo_root()
    names = set()
    for parent in (base, os.path.join(base, "crates")):
        try:
            entries = os.listdir(parent)
        except OSError:
            continue
        names.update(e for e in entries
                     if not e.startswith(".") and os.path.isdir(os.path.join(parent, e)))
    if repo_root is None:
        _WORKSPACE_ROOTS = names
    return names


def _ancestors(key):
    """Every `-`-delimited ancestor prefix of `key`, LONGEST first (`key` itself included)."""
    segs = key.split(_SEP)
    return [_SEP.join(segs[:i]) for i in range(len(segs), 0, -1)]


def partition_path(key, roots=None):
    """TOTAL map from an `area:` key to the hierarchical PARTITION PATH it reserves.

    Resolution, longest-recognised-ancestor first, so the failure direction is STRUCTURAL rather
    than dependent on anyone remembering to register a label:

      1. `GLOBAL` (and any empty/degenerate key) -> `()`, the root of the hierarchy. `()` is a
         prefix of every path, so GLOBAL conflicts with everything — the existing fail-closed
         backstop, now expressed as containment instead of two special cases in `conflict()`.
      2. The longest `-`-ancestor of `key` that the WORKSPACE recognises -> `(that ancestor,)`.
         `sparq-core-store` and `sparq-core-nt-dict` both resolve to `sparq-core`, so they conflict
         with their parent crate AND with each other (same crate, same files — a sibling hole would
         be the identical defect one level down). `sparq-engine-serialize` IS a crate directory, so
         it resolves to ITSELF and does NOT collapse into `sparq-engine`: genuinely unrelated
         crates stay parallel and the frontier survives.
      3. Otherwise the key's HEAD segment -> `(head,)`. A key whose parent the workspace does not
         know still declares a parent in its own structure, and honouring it over-reserves. This
         is what makes an invented `area:upstream-noir` conflict with `area:upstream` with no code
         change, and it leaves every single-segment key (`upstream`, `cli`, `docs`, ...) exactly
         where it is today — those name nothing narrower, so they cannot be under-serialising.

    The path is currently never deeper than one element ON PURPOSE: a sub-region collapses INTO its
    container rather than becoming a child of it. research/crate-region-parallelism.md §8 rejects
    intra-crate region partitioning as a parallelism lever (14.5% ceiling), and a two-level path
    would reopen the sibling hole. The prefix-based predicate below is depth-agnostic, so a future
    measured, gated subpartition (that record's Phase 1) can add depth without touching it.
    """
    if roots is None:
        memo = _PARTITION_MEMO
        if key in memo:
            return memo[key]
        path = _resolve(key, workspace_roots())
        memo[key] = path
        return path
    return _resolve(key, roots)


def _resolve(key, roots):
    if not key or key == GLOBAL:
        return ()
    for ancestor in _ancestors(key):
        if ancestor in roots:
            return (ancestor,)
    head = key.split(_SEP)[0]
    return (head,) if head else ()


def keys_conflict(a, b, roots=None):
    """Whether two `area:` keys reserve overlapping work — CONTAINMENT, not string equality.

    True iff one partition path is a prefix of the other, i.e. one region contains the other.
    Reflexive, symmetric, and NOT transitive-closed beyond containment: `sparq-core` and
    `sparq-engine` remain independent.
    """
    pa, pb = partition_path(a, roots), partition_path(b, roots)
    return pa[:len(pb)] == pb or pb[:len(pa)] == pa
# --- open blockers: NATIVE GitHub dependencies UNIONED with the legacy body markers -------------
# [OPUS-5] Until this landed, BOTH readers of "is this issue blocked" (this file and the registry's
# dispatch.yml planner step) derived `open_blockers` ONLY by regexing `Blocked-by: #NN` out of the
# issue BODY. GitHub issue dependencies are generally available and the maintainer uses the native
# UI, so a dependency added that way had ZERO effect on dispatch. MEASURED 2026-07-26 over all 1368
# open sparq issues (cursor-paginated, len == GraphQL totalCount): 112 issues carry a native edge
# (170 edges) and 0 of the 427 issues created since 2026-07-20 carry either channel — the marker
# channel is effectively dead for new work while the native channel is the one being written.
#
# UNION, never replace. 3 open issues (#2833, #2975, #3817) carry a marker with NO native edge, and
# the fail-safe direction is one-way: `exclusion_reason` keys on `open_blockers > 0`, so MISSING an
# edge dispatches an issue that is genuinely blocked, while OVER-counting one only delays it.
_MARKER_BLOCKED_BY = re.compile(r"[Bb]locked-by:\s*#(\d+)")
# GitHub's REST list payload carries this per non-PR issue at no extra request. `blocked_by` counts
# only OPEN blockers (`total_blocked_by` counts closed ones too) — MEASURED against GraphQL
# `blockedBy` filtered to state=OPEN over all 1368 open issues: identical sets AND identical
# per-issue counts, and 16 issues have total_blocked_by > blocked_by (a closed blocker), confirming
# a satisfied dependency is already excluded. So a CLOSED blocker never holds an issue here.
NATIVE_SUMMARY = "issue_dependencies_summary"
# A summary that is PRESENT but malformed is a schema change we cannot interpret. Admitting it as
# "0 blockers" is the fail-OPEN direction (it dispatches work that may be blocked), so it counts as
# one unknown blocker instead: the issue is held, loudly, until a human looks.
MALFORMED_SUMMARY_BLOCKERS = 1


def native_open_blockers(issue, warn=None):
    """OPEN blockers from GitHub's NATIVE dependency edges, per `issue_dependencies_summary`.

    ABSENT summary -> 0 (the honest reading: this snapshot carries no native-dependency data, so
    only the marker channel can speak). That absence is NOT silent — `_fetch` raises a corpus-level
    alarm if NO issue in a whole snapshot carries the field, which is what a GitHub field rename
    would look like and is otherwise indistinguishable from "nothing is blocked".
    PRESENT-but-malformed -> MALFORMED_SUMMARY_BLOCKERS (fail closed, see above).
    """
    summary = issue.get(NATIVE_SUMMARY)
    if summary is None:
        return 0
    number = issue.get("number", "?")
    if not isinstance(summary, dict):
        if warn is not None:
            warn(f"#{number}: {NATIVE_SUMMARY} is not an object — holding the issue (fail-closed)")
        return MALFORMED_SUMMARY_BLOCKERS
    value = summary.get("blocked_by")
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        if warn is not None:
            warn(f"#{number}: {NATIVE_SUMMARY}.blocked_by is {value!r}, not a non-negative int — "
                 "holding the issue (fail-closed)")
        return MALFORMED_SUMMARY_BLOCKERS
    return value


def marker_open_blockers(body, open_numbers):
    """OPEN blockers from validated `Blocked-by: #NN` BODY markers (the legacy channel)."""
    return sum(1 for n in _MARKER_BLOCKED_BY.findall(body or "") if int(n) in set(open_numbers))


def open_blocker_count(issue, open_numbers, warn=None):
    """The UNION of both blocker channels, as the count `exclusion_reason` consumes.

    `max` is the exact union for the only decision that is made from it — an issue is held iff
    `native > 0 or marker_open > 0`, which is precisely `max(...) > 0`. It is a LOWER BOUND on the
    cardinality of the union of the two blocker SETS (the native channel reports a count, not
    numbers, so a native blocker distinct from every marker blocker cannot be added to the marker
    count without double-counting the overlap). Under-reporting the count can only understate the
    delay in `--diagnose`; it can never flip a held issue to ready. MEASURED over all 1368 open
    issues: 0 issues where the two channels disagree on the count, so today max == the true union.
    """
    return max(native_open_blockers(issue, warn), marker_open_blockers(issue.get("body"),
                                                                       open_numbers))


def labels_of(issue):
    return {lb["name"] if isinstance(lb, dict) else lb for lb in issue.get("labels", [])}


def valid_priority(labels):
    """Exactly one valid priority:P0..P4 → its int; zero or multiple or out-of-range → None."""
    ps = {int(m.group(1)) for lb in labels for m in [_PRIO.match(lb)] if m}
    return next(iter(ps)) if len(ps) == 1 else None


def declared_packages(labels):
    """The SET of area:<crate> packages ACTUALLY declared — empty when none are, no global fallback.

    [OPUS-5] Separated from `packages_of` because the empty→GLOBAL fallback is only sound for a
    CANDIDATE (a no-area issue is cross-cutting work that must serialize against everything).
    Applying it to an in-flight artifact inverts the meaning: it turns "we cannot attribute this
    to a crate" into "this seizes every crate". Nothing labels PRs with `area:` — all 60 open
    sparq PRs carried none — so the old rule let a single unlabelled PR hold __global__ and
    reduce the whole frontier to zero. See `_reserving_packages`.
    """
    return {m.group(1) for lb in labels for m in [_PKG.match(lb)] if m}


def packages_of(labels):
    """The SET of all area:<crate> packages; empty → the serializing global partition.

    CANDIDATE-side rule (fail-closed): an unlabelled issue is treated as cross-cutting.
    """
    return declared_packages(labels) or {GLOBAL}


def _reserving_packages(labels):
    """OCCUPANCY-side rule: an in-flight artifact reserves ONLY the areas it declares.

    [OPUS-5] The deliberate asymmetry with `packages_of`. Fail-closed on a candidate costs one
    dispatch; fail-closed on an occupant costs the ENTIRE fleet, because an unattributable
    occupant would serialize every package at once. An occupant we cannot attribute therefore
    reserves nothing and is instead handled by the linked-issue suppression the registry planner
    applies (`Closes #N` / `sparq-agent/issue-N-*` heads) and, since reg#677, by folding it into
    its source issue's unit — see `unit_reservations`.

    [OPUS-5 2026-07-27] The original justification said "nothing in the pipeline applies `area:`
    labels to PRs". That is NO LONGER TRUE — `scripts/pr-area-labels.py` (pr-area-label.yml) now
    derives them from changed paths, and 109 of 123 open PRs carry one. The RULE is unchanged and
    still right: the deriver is itself fail-closed (it emits no label on unresolved or
    cross-cutting paths), so 9 open PRs still declare nothing, and making those 9 seize
    `__global__` would reproduce the measured whole-fleet stall. Only the stated reason needed
    correcting; leaving a false premise in place is how a correct rule gets "fixed" back.
    """
    return declared_packages(labels)


def has_role(labels):
    return any(_ROLE.match(lb) for lb in labels)


def is_gated(labels):
    return any(lb == g or lb.startswith(g) for lb in labels for g in GATE_LABELS)


def is_busy(labels):
    return bool(labels & BUSY_STATUS)


def is_parked(labels):
    return bool(labels & PARKED_AREA_LABELS)


def occupies_area(artifact):
    """Whether an otherwise in-flight PR/issue occupies its areas in this snapshot."""
    return not is_parked(labels_of(artifact))


def _artifact_name(artifact):
    if artifact is None:
        return "preseeded occupancy"
    kind = "pr" if "pull_request" in artifact else "issue"
    return f"{kind}#{artifact.get('number', '?')}"


def exclusion_reason(labels, open_blockers=0):
    """Why this label-set is NOT an enumerable ready candidate, or None when it is.

    Label/state gates ONLY — package serialization is a separate, capacity-shaped question
    answered by compute_ready(). Shared by ready_candidates() and the --diagnose taxonomy so the
    two can never drift apart.
    """
    if "status:ready" not in labels:
        return "no status:ready attestation"
    if NON_DISPATCHABLE in labels:
        return f"{NON_DISPATCHABLE} tracking umbrella"
    gates = sorted(lb for lb in labels for g in GATE_LABELS if lb == g or lb.startswith(g))
    if gates:
        return f"gated by {gates[0]}"
    busy = sorted(labels & BUSY_STATUS)
    if busy:
        return f"busy: {busy[0]}"
    parked = sorted(labels & PARKED_AREA_LABELS)
    if parked:
        return f"parked: {parked[0]}"
    if valid_priority(labels) is None:
        return "no single valid priority:P0..P4"
    if not has_role(labels):
        return "no role:* label"
    if int(open_blockers) > 0:
        return f"{int(open_blockers)} open blocker(s)"
    return None


def ready_candidates(issues, log=None):
    """The DRAINABLE backlog: every open issue whose LABELS make it dispatchable.

    [OPUS-5] Distinct from compute_ready(), which additionally serializes to one issue per
    package and therefore answers a CONCURRENCY question, not a "how much work is available"
    question. Conflating the two makes a healthy 200-item backlog behind a 4-wide frontier
    indistinguishable from an empty one. Returns [(priority, number, issue, packages)].

    `log`, when supplied, receives one attributable line per issue that carries the
    `status:ready` attestation and was nonetheless dropped — a silent `continue` here is what
    lets a label-regressed issue leave the frontier forever with zero signal. Issues without
    the attestation are NOT logged (they are not candidates and would flood the log).
    """
    out = []
    for it in issues:
        if str(it.get("state", "OPEN")).upper() != "OPEN":
            continue
        if "pull_request" in it:             # PRs reserve work; they are never issue candidates
            continue
        L = labels_of(it)
        reason = exclusion_reason(L, it.get("open_blockers", 0))
        if reason is not None:
            if log is not None and "status:ready" in L:
                log(f"defer #{it.get('number', '?')}: {reason}")
            continue
        out.append((valid_priority(L), it.get("number", 0), it, packages_of(L)))
    return out


def roleless_ready(issues):
    """The SILENT-INVISIBILITY class: open, attested, un-gated, un-blocked — and yet NO `role:*`.

    [OPUS-5] Ported from the registry planner (its issue #225, where 117 accumulated unnoticed).
    `ready_candidates` drops these BEFORE any plan row exists, so they appear in no plan and in
    no diagnostic and drain never. The fail-closed drop is CORRECT — a role is never guessed —
    but its silence was not, so callers report this count loudly. Pure; returns sorted numbers.
    """
    numbers = []
    for it in issues:
        if str(it.get("state", "OPEN")).upper() != "OPEN" or "pull_request" in it:
            continue
        L = labels_of(it)
        if "status:ready" not in L or NON_DISPATCHABLE in L:
            continue
        if is_gated(L) or is_busy(L) or is_parked(L):
            continue
        if int(it.get("open_blockers", 0)) > 0:
            continue
        if not has_role(L):
            numbers.append(it.get("number", 0))
    return sorted(numbers)


def _own_reservation(row):
    """What ONE snapshot row reserves BY ITSELF, under the existing per-half occupancy rules.

    Extracted verbatim from `compute_ready`'s occupancy loop so `unit_reservations` can union
    member reservations WITHOUT restating (and so drifting from) the rule. Every active open PR is
    in flight, drafts included; an open issue occupies only while `status:in-progress*`; the shared
    parked predicate vetoes both. A row that reserves nothing returns the empty set.
    """
    if str(row.get("state", "OPEN")).upper() != "OPEN" or not occupies_area(row):
        return set()
    labels = labels_of(row)
    if "pull_request" in row or labels & IN_FLIGHT_STATUS:
        return _reserving_packages(labels)
    return set()


def unit_reservations(issues, source_links=None):
    """Occupancy as ONE reservation per UNIT OF WORK — a PR together with the issues it closes.

    [OPUS-5] A worker PR and its source issue are the SAME unit of work, and each was reserving
    independently: MEASURED on the live sparq snapshot (2026-07-27, 1473 open issues / 123 open
    PRs) 65 occupying PRs plus 46 in-flight issues produced 158 reservations over 49 distinct
    partition keys — 20 of those reservations duplicates of a key the unit's other half already
    held. `source_links` (a PR-number -> source-issue-number map from `source_issue_links`) folds
    each pair into one reservation of the UNION, attributed to the PR.

    Two things this deliberately is NOT, both refuted by measurement rather than by argument:

    * It is NOT a frontier lever. `conflict()` tests membership in the SET of held keys, so a
      second occupant on an already-held key is a no-op. Deduplicating 158 -> 138 reservations
      leaves the held set at 49 keys and the live frontier unmoved (3 -> 3). Anything that DOES
      widen the frontier here widens it by RELEASING a key, and releasing is the corrupting
      direction — see the next point.
    * It is NOT permission to drop the issue half. MEASURED over the 94 open PRs with at least one
      open linked source issue: 31 pairs have PR ⊋ issue, 18 identical, 6 have a source issue with
      NO `area:` at all — but 13 have PR ⊊ issue and 26 are INCOMPARABLE (each side declares a key
      the other lacks). So in 39/94 = 41% of pairs the PR's file-derived key set is NOT a superset
      and dropping the issue's reservation would free a key the unit really occupies. The union is
      the only rule that is safe in both directions.

    MONOTONE BY CONSTRUCTION: a unit reserves `⋃ _own_reservation(member)`, so its reservation is a
    superset of every member's own — the dedup can never under-serialise relative to today, whoever
    the members are. Registry CLAIM's extra `areas |= issue_areas or {GLOBAL_PACKAGE}` fail-closed
    step is NOT adopted here: applied to this population it drives the live frontier to 0
    (measured) — the same whole-fleet seizure `_reserving_packages` documents and exists to
    prevent.

    `source_links=None` (the default, and what the registry's `dispatch.yml` passes today) yields
    exactly the legacy per-row reservations, in the legacy order — see
    `test_unit_reservations_without_links_is_identical_to_the_legacy_loop`.
    """
    links = source_links or {}
    by_number = {}
    for row in issues:
        by_number.setdefault(row.get("number"), row)

    def sources_of(pr_number):
        for number in sorted(links.get(pr_number) or ()):
            row = by_number.get(number)
            if row is not None and "pull_request" not in row:
                yield number, row

    consumed = {number for row in issues if "pull_request" in row
                for number, _ in sources_of(row.get("number"))}
    out = []
    for row in issues:                      # input order preserved: attribution is order-sensitive
        number = row.get("number")
        if "pull_request" not in row:
            if number in consumed:          # already reserved as part of its PR's unit
                continue
            areas = _own_reservation(row)
        else:
            areas = set(_own_reservation(row))
            for _number, source in sources_of(number):
                areas |= _own_reservation(source)
        if areas:
            out.append((areas, row))
    return out


def compute_ready(issues, in_progress_packages=None, conflict_log=None, source_links=None):
    """Conflict-free, priority-ordered, FAIL-CLOSED CONCURRENCY FRONTIER.

    This is the one-per-package concurrency WIDTH, not the size of the drainable backlog — use
    `ready_candidates()` for the latter. compute_ready() ⊆ ready_candidates() always.

    `conflict_log`, when supplied, receives one attribution line per conflict-excluded candidate;
    the live default writes those diagnostics to stderr without polluting the frontier rows.

    `source_links`, when supplied, makes a PR and the issues it closes reserve ONCE, as the union
    of both halves — see `unit_reservations`. Omitted, occupancy is byte-identical to the legacy
    per-row loop, so the registry's existing `compute_ready(ready_input)` call is unaffected and
    the two repositories may merge in either order.
    """
    blockers = {}

    def reserve(pkgs, artifact):
        for pkg in sorted(pkgs):
            blockers.setdefault(pkg, []).append(artifact)

    def conflict(pkgs):
        """The held key that CONTAINS-or-is-contained-by one of `pkgs`, or None.

        [OPUS-5] sparq#4336: was exact-string set overlap (`pkgs & blockers.keys()`) plus two
        hand-written GLOBAL special cases. Exact-string overlap under-serialised every sub-crate
        key against its parent crate; the GLOBAL cases are now just the `()` path being a prefix
        of everything, so there is ONE rule instead of three. Attribution reports the RAW held
        label (not its resolved partition) so a conflict line still names the artifact's own key;
        the coarsest holder wins, then alphabetical, so the message stays deterministic.
        """
        held = [key for key in blockers if any(keys_conflict(key, p) for p in pkgs)]
        if not held:
            return None
        area = min(held, key=lambda key: (len(partition_path(key)), key))
        return area, blockers[area][0]

    for pkg in sorted(set(in_progress_packages or ())):
        reserve({pkg}, None)
    # [GPT-5.6] Every active open PR is in flight (drafts included); open issues occupy only while
    # status:in-progress. The shared parked predicate is applied before either can reserve areas.
    # [OPUS-5] ...and a PR plus the issues it closes are ONE unit reserving the union ONCE.
    for areas, artifact in unit_reservations(issues, source_links):
        reserve(areas, artifact)
    cands = ready_candidates(issues)
    cands.sort(key=lambda c: (c[0], c[1]))   # priority then number (deterministic)
    ready = []
    for _p, _n, it, pkgs in cands:
        held = conflict(pkgs)
        if held is not None:
            area, blocker = held
            message = (f"conflict #{it.get('number', '?')}: area {area} held by "
                       f"{_artifact_name(blocker)}")
            if conflict_log is None:
                print(message, file=sys.stderr)
            else:
                conflict_log(message)
            continue
        reserve(pkgs, it)
        ready.append(it)
    return ready


def _self_test():
    def iss(n, labels, blk=0, state="OPEN"):
        return {"number": n, "state": state, "labels": labels, "open_blockers": blk}

    R = ["status:ready", "role:impl"]

    def quiet(_message):
        pass

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

    ready = compute_ready(F, conflict_log=quiet)
    # eligible: 1,2,3,12 (+11 global). 4 gated, 5 blocked, 6 closed, 7 untrusted, 8 no-ready,
    # 9 ambiguous-prio, 10 in-progress, 13 epic (kind:epic → excluded despite a P0 ready label-set).
    # Order by prio: #2(P0 core) -> #3(P1 engine) -> #12(P1 hdt) -> #11(P4 global). core taken by #2
    # so #1(P2 core) excluded. #11 global: only selectable if nothing taken -> excluded.
    check("ready order", [i["number"] for i in ready], [2, 3, 12])
    check("existing readiness fixtures unchanged-green", [i["number"] for i in ready], [2, 3, 12])
    # a P0 epic with an otherwise-perfect ready label-set must NOT dispatch (tracking umbrella):
    check("epic excluded", 13 in [i["number"] for i in ready], False)
    # a lone global issue with an empty board is selectable:
    check("lone global", [i["number"] for i in compute_ready(
        [iss(11, R + ["priority:P4"])], conflict_log=quiet)], [11])
    # global blocks everything else:
    g = compute_ready(
        [iss(11, R + ["priority:P0"]), iss(12, R + ["priority:P1", "area:sparq-hdt"])],
        conflict_log=quiet)
    check("global serializes", [i["number"] for i in g], [11])

    # [GPT-5.6] Parked-occupancy tripwires. These are end-to-end through compute_ready so deleting
    # the predicate, broadening it to every draft, or removing attribution makes --self-test red.
    def pr(n, labels, draft=True):
        return {"number": n, "state": "OPEN", "labels": labels,
                "pull_request": {}, "draft": draft}

    waiting = iss(20, R + ["priority:P1", "area:sparq-store"])
    parked = pr(70, ["area:sparq-store", "needs:user"])
    check("needs:user-parked draft PR does not block ready issue",
          [i["number"] for i in compute_ready([parked, waiting], conflict_log=quiet)], [20])
    unparked = {**parked, "labels": ["area:sparq-store"]}
    check("park-label removal restores snapshot occupancy",
          compute_ready([unparked, waiting], conflict_log=quiet), [])

    active = pr(71, ["area:sparq-store", "review:changes"])
    active_logs = []
    check("non-parked draft PR still blocks",
          compute_ready([active, waiting], conflict_log=active_logs.append), [])
    check("conflict log names the blocking artifact",
          active_logs, ["conflict #20: area sparq-store held by pr#71"])
    per_exclusion_logs = []
    global_then_two = compute_ready([
        iss(30, R + ["priority:P0"]),
        iss(31, R + ["priority:P1", "area:sparq-core"]),
        iss(32, R + ["priority:P2", "area:sparq-engine"]),
    ], conflict_log=per_exclusion_logs.append)
    check("one conflict log per excluded candidate",
          ([it["number"] for it in global_then_two], per_exclusion_logs),
          ([30], ["conflict #31: area __global__ held by issue#30",
                  "conflict #32: area __global__ held by issue#30"]))

    in_progress = iss(72, ["status:in-progress", "area:sparq-store"])
    check("status:in-progress issue still blocks",
          compute_ready([in_progress, waiting], conflict_log=quiet), [])
    check("all terminal park labels remove occupancy",
          [[it["number"] for it in compute_ready(
              [pr(73 + i, ["area:sparq-store", label]), waiting], conflict_log=quiet)]
           for i, label in enumerate(sorted(PARKED_AREA_LABELS))], [[20], [20], [20]])
    # [OPUS-5] sparq#4336 CONTAINMENT fixtures. The registry's dispatch.yml runs THIS --self-test,
    # so these are the assertions that gate the fleet's own copy of the key algebra.
    check("sub-crate key resolves to its crate", partition_path("sparq-server-http"),
          ("sparq-server",))
    check("real sibling crate resolves to itself", partition_path("sparq-engine-serialize"),
          ("sparq-engine-serialize",))
    check("invented sub-crate key resolves to its crate", partition_path("sparq-server-zzz"),
          ("sparq-server",))
    check("degenerate key falls all the way to global", partition_path(""), ())
    check("unrelated crates do not conflict",
          keys_conflict("sparq-core", "sparq-engine"), False)
    check("sibling regions of one crate conflict",
          keys_conflict("sparq-core-store", "sparq-core-nt-dict"), True)
    check("parent+child enter the frontier together? (must not)",
          [i["number"] for i in compute_ready(
              [iss(80, R + ["priority:P1", "area:sparq-server"]),
               iss(81, R + ["priority:P1", "area:sparq-server-http"])], conflict_log=quiet)], [80])
    check("child+parent, reversed order (must not)",
          [i["number"] for i in compute_ready(
              [iss(80, R + ["priority:P1", "area:sparq-server-http"]),
               iss(81, R + ["priority:P1", "area:sparq-server"])], conflict_log=quiet)], [80])
    check("open PR on the parent crate blocks a sub-crate issue",
          compute_ready([pr(82, ["area:sparq-server"]),
                         iss(83, R + ["priority:P1", "area:sparq-server-http"])],
                        conflict_log=quiet), [])
    check("unknown key under an unknown parent still over-reserves",
          keys_conflict("upstream", "upstream-noir"), True)
    check("single-segment unknown key keeps its own partition",
          partition_path("deps"), ("deps",))
    # ---------------------------------------------------------------------------------------
    # NATIVE dependency edges (the maintainer's own triage action). Every row below is written to
    # go RED if the native read is deleted, if the union is turned into a replacement, or if the
    # closed-blocker exemption is broken. They run END-TO-END through compute_ready wherever the
    # decision (dispatch / hold) is what matters — a pure-count assertion alone would survive
    # `_fetch` never calling open_blocker_count at all, so the `_fetch`-shaped rows are included.
    # ---------------------------------------------------------------------------------------
    def raw_issue(n, labels, body="", summary=None, pr=False):
        """A row in the SHAPE `_fetch` receives from `gh api repos/../issues`."""
        row = {"number": n, "state": "open", "labels": [{"name": lb} for lb in labels],
               "body": body}
        if summary is not None:
            row[NATIVE_SUMMARY] = summary
        if pr:
            row["pull_request"] = {}
        return row

    def summary(open_blockers, total=None):
        return {"blocked_by": open_blockers, "blocking": 0,
                "total_blocked_by": open_blockers if total is None else total, "total_blocking": 0}

    ready_labels = R + ["priority:P1", "area:sparq-core"]
    # (1) THE REGRESSION THIS EXISTS FOR: a native edge and NO body marker must hold the issue.
    native_only = raw_issue(40, ready_labels, body="no marker here", summary=summary(1))
    check("native blocked_by with NO body marker excludes from ready",
          [it["number"] for it in compute_ready(_fetch_rows([native_only]), conflict_log=quiet)],
          [])
    check("...and the same issue with the native edge cleared IS ready",
          [it["number"] for it in compute_ready(
              _fetch_rows([raw_issue(40, ready_labels, body="no marker here",
                                     summary=summary(0))]), conflict_log=quiet)],
          [40])
    # (2) the LEGACY channel must keep working — union, never replace. #2833/#2975/#3817 are live
    # marker-only rows, so a replacement would silently drop them.
    blocker = raw_issue(41, ["role:impl"])
    marker_only = raw_issue(42, ready_labels, body="Blocked-by: #41", summary=summary(0))
    check("marker-only edge (native says zero) still excludes from ready",
          [it["number"] for it in compute_ready(_fetch_rows([blocker, marker_only]),
                                                conflict_log=quiet)], [])
    # (3) a CLOSED blocker must NOT hold the child on either channel. Native: `blocked_by` counts
    # only open blockers while `total_blocked_by` counts the closed one (MEASURED on 16 live
    # issues, e.g. #3264 blocked_by=0 total_blocked_by=1). Marker: #43 is absent from the open set.
    closed_blocked = raw_issue(44, ready_labels, body="Blocked-by: #43", summary=summary(0, total=2))
    check("issue whose ONLY blocker is CLOSED is NOT excluded",
          [it["number"] for it in compute_ready(_fetch_rows([closed_blocked]), conflict_log=quiet)],
          [44])
    # (4) union arithmetic, at the count level, on every channel combination.
    check("open_blocker_count unions both channels (never replaces either)",
          [open_blocker_count(raw_issue(1, [], body=b, summary=s), {41})
           for b, s in (("", None), ("", summary(0)), ("", summary(3)),
                        ("Blocked-by: #41", summary(0)), ("Blocked-by: #41", summary(3)),
                        ("Blocked-by: #99", summary(0)))],
          [0, 0, 3, 1, 3, 0])
    # (5) a PRESENT-but-malformed summary must FAIL CLOSED (hold), never admit.
    warnings = []
    check("malformed native summary holds the issue and says so",
          ([native_open_blockers(raw_issue(45, [], summary=s), warnings.append)
            for s in ({"blocked_by": -1}, {"blocked_by": "1"}, {"blocked_by": True},
                      {"blocked_by": None}, ["not", "a", "dict"])],
           len(warnings)),
          ([MALFORMED_SUMMARY_BLOCKERS] * 5, 5))
    check("...and it is the FRONTIER that holds, not just the count",
          [it["number"] for it in compute_ready(
              _fetch_rows([raw_issue(45, ready_labels, summary={"blocked_by": "1"})]),
              conflict_log=quiet)], [])
    # (6) the DARK-CHANNEL alarm: absent-on-every-issue is a schema regression, not a quiet repo.
    check("native-channel-dark snapshot raises the alarm",
          [("DARK" in line, NATIVE_SUMMARY in line) for line in native_channel_alarm(
              [raw_issue(50, []), raw_issue(51, []), raw_issue(52, [], pr=True)])],
          [(True, True)])
    check("one issue carrying the summary is enough to keep the channel LIT",
          native_channel_alarm([raw_issue(50, []), raw_issue(51, [], summary=summary(0))]), [])
    check("a PR-only snapshot never raises the dark alarm",
          native_channel_alarm([raw_issue(52, [], pr=True)]), [])
    check("valid_priority single", valid_priority({"priority:P0"}), 0)
    check("valid_priority ambiguous", valid_priority({"priority:P1", "priority:P2"}), None)
    check("valid_priority out-of-range", valid_priority({"priority:P7"}), None)
    check("packages multi", packages_of({"area:a", "area:b"}), {"a", "b"})
    check("packages none->global", packages_of({"role:impl"}), {GLOBAL})
    check("untriaged is busy", is_busy({"status:untriaged"}), True)
    # paginated-snapshot flattening: multi-page merge, PR rows retained for occupancy, junk tolerated
    check("flatten pages retains PRs", _flatten_pages(
        [[{"number": 1}, {"number": 2, "pull_request": {}}], [{"number": 3}], "junk", [None]]),
        [{"number": 1}, {"number": 2, "pull_request": {}}, {"number": 3}])
    print("ready-issues self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


def _flatten_pages(pages):
    """Flatten `gh api --paginate --slurp` output, retaining PRs as occupancy artifacts."""
    return [i for page in pages for i in (page if isinstance(page, list) else [])
            if isinstance(i, dict)]


_LINK_HEAD = re.compile(r"^sparq-agent/issue-([1-9][0-9]*)-")
_CLOSES = re.compile(r"(?i)\b(?:close[sd]?|fix(?:e[sd])?|resolve[sd]?)\s+#([1-9][0-9]*)\b")
TRUSTED_ASSOCIATIONS = {"OWNER", "MEMBER", "COLLABORATOR"}


def source_issue_links(pulls, repo):
    """PR number -> the set of issue numbers that PR is the WORKING ARTIFACT for.

    [OPUS-5] The pairing behind `unit_reservations`, and the single definition of PR->issue
    linkage in this file: `linked_issue_numbers` is now its union, so the suppression set and the
    occupancy pairing can never disagree about what "covered by an open PR" means. Two rules that
    each re-derive linkage is exactly the mint-vs-adopt drift that lets one layer free a key the
    other still holds.

    A fork PR must never link an issue: only a same-repo `sparq-agent/issue-N-*` head is
    pipeline-owned provenance (a fork's head branch text is attacker-controlled), and a closing
    keyword in a body counts only from a trusted author association. PRs with no linkage are
    absent from the map rather than present-and-empty, so `unit_reservations` treats them as
    single-member units.
    """
    links = {}
    for pull in pulls:
        head = pull.get("head") or {}
        ref = head.get("ref") or ""
        body = pull.get("body") or ""
        same_repo = ((head.get("repo") or {}).get("full_name") == repo)
        app_pr = same_repo and _LINK_HEAD.match(ref) is not None
        found = set()
        if app_pr:
            found.update(int(n) for n in _LINK_HEAD.findall(ref))
        if app_pr or str(pull.get("author_association", "")).upper() in TRUSTED_ASSOCIATIONS:
            found.update(int(n) for n in _CLOSES.findall(body))
        if found:
            links.setdefault(pull.get("number"), set()).update(found)
    return links


def linked_issue_numbers(pulls, repo):
    """Issues already covered by an open PR — the suppression the REGISTRY planner applies.

    [OPUS-5] Mirrors dispatch.yml's `linked_issue_numbers` so the local CLI previews the same
    frontier the orchestrator dispatches. Derived from `source_issue_links` so the suppression
    set is, by construction, exactly the set of issues that are somebody's unit-of-work member.
    """
    return set().union(set(), *source_issue_links(pulls, repo).values())


def _fetch_pulls(repo):
    out = subprocess.run(
        ["gh", "api", "--paginate", "--slurp", f"repos/{repo}/pulls?state=open&per_page=100"],
        capture_output=True, text=True, check=True).stdout
    return [p for page in json.loads(out or "[]") if isinstance(page, list)
            for p in page if isinstance(p, dict)]


def _fetch(repo, ceiling=10000):
    """Open-issue snapshot via REAL cursor pagination (`gh api --paginate` follows Link headers),
    replacing the old single-page `--limit 1000` fetch that FAILED CLOSED at exactly 1000 open
    issues — the full bd migration (~900 beads on top of organic issues) crosses that. The explicit
    ceiling still fails closed on a runaway snapshot."""
    out = subprocess.run(
        ["gh", "api", "--paginate", "--slurp",
         f"repos/{repo}/issues?state=open&per_page=100"],
        capture_output=True, text=True, check=True).stdout
    pages = json.loads(out or "[]")
    raw = _flatten_pages(pages)
    if len(raw) >= ceiling:
        raise SystemExit(f"refusing: fetched {len(raw)} >= ceiling {ceiling} — snapshot looks "
                         "runaway (fail-closed). Raise the ceiling deliberately if the backlog "
                         "is really that large.")
    issues = _fetch_rows(raw, warn=lambda m: print(f"::warning::{m}", file=sys.stderr))
    for line in native_channel_alarm(raw):
        print(line, file=sys.stderr)
    return issues


def _fetch_rows(raw, warn=None):
    """The PURE half of `_fetch`: GitHub issue payloads -> readiness-engine rows.

    Split out so `--self-test` exercises the REAL row builder. Asserting on `open_blocker_count`
    alone would stay green with `_fetch` never calling it — which is exactly the shape of the bug
    being fixed (a correct blocker rule that no dispatcher consulted).
    """
    open_numbers = {i["number"] for i in raw if "pull_request" not in i}
    issues = []
    for i in raw:
        row = {"number": i["number"], "state": i["state"], "labels": i["labels"],
               "open_blockers": 0}
        if "pull_request" in i:
            row["pull_request"] = i["pull_request"]
            row["draft"] = i.get("draft")
        else:
            row["open_blockers"] = open_blocker_count(i, open_numbers, warn)
        issues.append(row)
    return issues


def native_channel_alarm(raw):
    """The GUARD against the native blocker channel going DARK without anyone noticing.

    `native_open_blockers` reads an ABSENT `issue_dependencies_summary` as 0 — correct for an old
    snapshot, and indistinguishable from "GitHub renamed the field" if nobody checks. MEASURED
    2026-07-26: the field is present on 1368/1368 open sparq ISSUES and 0/104 PR rows, so "no
    non-PR row carries it" is a schema regression, not a quiet repo. Returns the lines to print
    (pure, so the check itself is testable rather than a side effect nobody exercises).
    """
    non_pr = [i for i in raw if isinstance(i, dict) and "pull_request" not in i]
    if not non_pr:
        return []
    if any(isinstance(i.get(NATIVE_SUMMARY), dict) for i in non_pr):
        return []
    return [f"::warning::NATIVE BLOCKER CHANNEL IS DARK: none of {len(non_pr)} open issues carries "
            f"`{NATIVE_SUMMARY}`. Native GitHub dependencies are being IGNORED and only "
            "`Blocked-by: #NN` body markers can hold an issue — a maintainer's native dependency "
            "edits have no effect on dispatch until this is fixed."]


def dispatchable_view(issues, linked=()):
    """The rows the ORCHESTRATOR feeds to compute_ready — the single source of the local preview.

    [OPUS-5] Mirrors dispatch.yml's `ready_input` comprehension exactly:

        ready_input = [row for row in readiness_input
                       if "status:in-progress" in row["labels"]
                       or "status:in-progress-review" in row["labels"]
                       or (row["number"] not in linked and trusted(...))]

    The `or` matters and a plain `number not in linked` is WRONG for the dominant live shape.
    A `status:in-progress-review` issue is normally covered by its OWN worker PR, so it is
    always in `linked`; dropping it frees the crate it is actively occupying and the next tick
    dispatches a SECOND worker onto it. dispatch.yml keeps those rows deliberately — "In-progress
    rows are KEPT as inputs: compute_ready never selects them (they are busy), but they must
    still RESERVE their package in its taken-seeding". Executed counterexample, in-review #100 on
    sparq-core covered by its own PR plus attested #200 on sparq-core: the `not in linked` rule
    yields [200], the orchestrator yields [].

    Residual, deliberate divergence: dispatch.yml ALSO requires `trusted(issue, bots)`, which
    needs the issue AUTHOR and the registry's per-repo trusted-bot list. The local snapshot
    carries neither, so the local preview is an UPPER BOUND on the orchestrator frontier — it
    may show a row the registry will drop as untrusted. `trust:untrusted` (the label the triage
    pipeline applies) is still excluded here via GATE_LABELS. Parity is claimed on the
    linked/in-flight axis only.
    """
    linked = set(linked)
    return [it for it in issues
            if it.get("number") not in linked or labels_of(it) & IN_FLIGHT_STATUS]


def occupancy_parity(issues, source_links=None):
    """The PR-HALF-STRIPPED occupancy divergence, as a re-runnable measurement.

    [OPUS-5] `dispatchable_view` claims local/orchestrator parity on the linked/in-flight axis, and
    on the CANDIDATE axis it holds. On the OCCUPANCY axis it does not: the registry's dispatch.yml
    builds its readiness input as `[issue for issue in snapshot("issues") if "pull_request" not in
    issue]`, so PLAN never sees a PR row and reserves the ISSUE half of every unit only. MEASURED on
    the live snapshot (2026-07-27): the local CLI holds 49 partition keys and emits a frontier of 3;
    the orchestrator holds 37 and emits 9 — and 7 of those 9 rows land in a key an open PR already
    holds. They are not double-dispatched (registry CLAIM re-derives busy areas from the pulls
    snapshot and drops them), but they are dropped AFTER `compute_ready` committed the frontier, so
    each one burned a partition with no backfill — the registry's own issue #113 shape.

    Returns (pr_aware_keys, issue_only_keys, unheld) where `unheld` is the set of keys the
    issue-only view fails to hold. Pure, so `--diagnose` and the test suite read the same number.
    """
    def keys(rows, links):
        held = set()
        for areas, _artifact in unit_reservations(rows, links):
            held |= areas
        return held
    issue_only = [row for row in issues if "pull_request" not in row]
    pr_aware = keys(issues, source_links)
    stripped = keys(issue_only, source_links)
    return pr_aware, stripped, pr_aware - stripped


def diagnose(issues, linked=(), source_links=None):
    """Re-runnable VISIBILITY taxonomy: why the open backlog is not on the frontier.

    Returns (counts, roleless, candidates, frontier, units). Every open issue lands in exactly one
    bucket, so the buckets sum to the open-issue count and no class can hide.

    [OPUS-5] `units` is `unit_reservations(visible, source_links)` — the OCCUPANCY accounting, which
    is the only thing `source_links` can change here. The held KEY SET is provably invariant under
    folding (a unit reserves the union of exactly the members' own reservations, so the union over
    all units equals the union over all members), hence `frontier` is identical with and without
    `source_links` and cannot witness the fold. Returning `units` is what makes the parameter
    observable at this layer — without it the argument would be an equivalent mutant, green under
    deletion, which is precisely how a call site rots.
    """
    linked = set(linked)
    counts, open_issues = {}, []
    for it in issues:
        if str(it.get("state", "OPEN")).upper() != "OPEN" or "pull_request" in it:
            continue
        open_issues.append(it)
        if it.get("number") in linked and not (labels_of(it) & IN_FLIGHT_STATUS):
            reason = "covered by an open linked PR"
        else:
            reason = exclusion_reason(labels_of(it), it.get("open_blockers", 0)) or "ENUMERABLE"
        counts[reason] = counts.get(reason, 0) + 1
    visible = dispatchable_view(issues, linked)
    # `source_links` is deliberately NOT passed to compute_ready here: this call discards the
    # conflict log, and the frontier is PROVABLY invariant under folding (see the docstring and
    # `test_the_held_key_set_is_invariant_under_folding`). Passing it would be an argument no test
    # could ever kill — an equivalent mutant, green under deletion, i.e. a call site that rots.
    return (counts, roleless_ready(open_issues), ready_candidates(visible),
            compute_ready(visible, conflict_log=lambda _m: None),
            unit_reservations(visible, source_links))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default="sparq-org/sparq")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--diagnose", action="store_true",
                    help="print the full visibility taxonomy instead of just the frontier")
    ap.add_argument("--dump-partitions", action="store_true",
                    help="print the recognised partition roots (and the resolution of any KEYs) "
                         "as JSON — the machine-readable contract the registry's dispatch.yml "
                         "mirror must agree with; offline, no API calls")
    ap.add_argument("keys", nargs="*", metavar="KEY",
                    help="area: keys to resolve with --dump-partitions")
    args = ap.parse_args()
    if args.self_test:
        return _self_test()
    if args.dump_partitions:
        json.dump({"roots": sorted(workspace_roots()),
                   "resolved": {k: list(partition_path(k)) for k in args.keys}},
                  sys.stdout, indent=2, sort_keys=True)
        print()
        return 0
    issues = _fetch(args.repo)
    source_links = source_issue_links(_fetch_pulls(args.repo), args.repo)
    linked = set().union(set(), *source_links.values())
    visible = dispatchable_view(issues, linked)
    if args.diagnose:
        counts, roleless, cands, frontier, units = diagnose(issues, linked, source_links)
        total = sum(counts.values())
        print(f"open issues: {total}")
        for reason, n in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0])):
            print(f"  {n:5d}  {100 * n / total:5.1f}%  {reason}")
        print(f"\ndrainable backlog (ready_candidates): {len(cands)}")
        print(f"concurrency frontier (compute_ready): {len(frontier)}")
        print(f"unit occupancy: {len(units)} unit(s), "
              f"{sum(len(a) for a, _ in units)} reservation(s) over "
              f"{len(set().union(set(), *[a for a, _ in units]))} partition key(s)")
        _pr_aware, issue_only, unheld = occupancy_parity(visible, source_links)
        if unheld:
            print(f"\n::warning:: ORCHESTRATOR OCCUPANCY GAP: dispatch.yml strips PR rows from its "
                  f"readiness input, so PLAN holds {len(issue_only)} of {len(_pr_aware)} partition "
                  f"key(s). {len(unheld)} key(s) held here by an open PR are FREE there: "
                  f"{', '.join(sorted(unheld)[:20])}")
        if roleless:
            print(f"\n::warning:: {len(roleless)} attested issue(s) carry NO role:* label and are "
                  f"INVISIBLE to dispatch: {', '.join('#%d' % n for n in roleless[:20])}")
        return 0
    for it in compute_ready(visible, source_links=source_links):
        L = labels_of(it)
        print(f"P{valid_priority(L)}  #{it['number']:5}  {sorted(packages_of(L))}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
