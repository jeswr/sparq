#!/usr/bin/env python3
# [OPUS-4.8] Routing resolver — the consumer contract for orchestration/routing.toml (review D2).
"""route-resolve.py — resolve an issue's labels to (model_chain, agent, escalate).

Implements the PRECEDENCE the review flagged as unspecified: **security-label override > explicit
role > [defaults]**, FIRST MATCH WINS. `match_labels` rules match if any listed keyword is a
SUBSTRING of any issue label (so `zk` matches `area:sparq-zk`). Because the table lists security
rules first, an `impl` issue that also touches `area:sparq-zk` routes to Opus (soundness), not Fable.
"""
import sys

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib


def validate_routing(doc):
    """Structural invariants a routing table must satisfy before ANY resolution — enforced in
    resolve() so a violating table fails LOUDLY at PLAN time instead of silently routing.
    Maintainer rule 2026-07-18: the cheap tiers `sonnet` and `terra` author NOTHING but docs —
    they may appear only in the model_chain of a route whose role == "docs" (never in defaults,
    a match_labels security rule, or any other role's chain)."""
    docs_only = {"sonnet", "terra"}
    offenders = []
    if docs_only & set(doc.get("defaults", {}).get("model_chain", [])):
        offenders.append("defaults")
    for r in doc.get("route", []):
        if docs_only & set(r.get("model_chain", [])) and r.get("role") != "docs":
            offenders.append(r.get("role") or ",".join(r.get("match_labels", [])) or "<unnamed>")
    if offenders:
        raise ValueError("routing violates the docs-only rule for sonnet/terra (maintainer "
                         "2026-07-18) in: " + "; ".join(offenders))


def resolve(labels, doc):
    """Return (model_chain, agent, escalate). `labels`: iterable of the issue's labels."""
    validate_routing(doc)
    labels = set(labels)

    def role_of(lbs):
        for lb in lbs:
            if lb.startswith("role:"):
                return lb[5:]
        return None

    role = role_of(labels)
    for r in doc.get("route", []):
        kws = r.get("match_labels")
        if kws:  # security-label rule: any keyword is a substring of any label
            if any(k in lb for lb in labels for k in kws):
                return r["model_chain"], r["agent"], bool(r.get("escalate"))
        elif "role" in r and role is not None and r["role"] == role:
            return r["model_chain"], r["agent"], bool(r.get("escalate"))
    d = doc.get("defaults", {})
    return d.get("model_chain", []), d.get("agent"), False


def _self_test():
    doc = tomllib.load(open("orchestration/routing.toml", "rb"))
    ok = True

    def chk(n, got, want):
        nonlocal ok
        good = got == want
        ok = ok and good
        print(f"  {'ok  ' if good else 'FAIL'} {n}: {got} (want {want})")

    # impl + a security surface (area:sparq-zk) -> security rule wins over role -> Opus-5-led
    # (opus-4.8 tail fallback, 2026-07-24), escalate
    mc, ag, esc = resolve(["role:impl", "area:sparq-zk"], doc)
    chk("impl+zk -> opus5-led/escalate", (mc, ag, esc), (["opus5", "opus"], "sparq-reviewer", True))
    # plain impl -> sol-led chain (maintainer directive 2026-07-18)
    mc, ag, esc = resolve(["role:impl", "area:sparq-core"], doc)
    chk("impl -> sol-led", (mc, ag, esc), (["sol", "opus5", "fable", "opus"], "sparq-rust-impl", False))
    # docs -> haiku-led
    chk("docs -> haiku", resolve(["role:docs", "area:x"], doc)[0][0], "haiku")
    # [FABLE-5] UI ownership: site -> terra-led (GPT-5.6 codex, the original dashboard builder)
    chk("site -> sol-led (GPT owns UI; terra is docs-only)", resolve(["role:site", "area:site"], doc)[0], ["sol", "opus5", "fable", "opus"])
    # [FABLE-5] frontier-tier infra authorship (standing rule 2026-07-17): ci -> sol-first
    # (opus5 the primary anthropic tier since 2026-07-24, fable its tail fallback), FRONTIER-ONLY
    # chain — no sub-frontier model (sonnet/haiku) anywhere in it, so exhaustion
    # DEFERS at the registry claim step (retried next tick) instead of degrading tier.
    mc, ag, esc = resolve(["role:ci", "area:ci"], doc)
    chk("ci -> frontier-only sol-first", (mc, ag, esc), (["sol", "opus5", "fable"], "sparq-ci-infra", False))
    chk("ci chain has no sub-frontier tier", sorted(set(mc) & {"sonnet", "haiku"}), [])
    # no role -> defaults (sol-led, 2026-07-18)
    chk("no role -> defaults", resolve(["area:sparq-core"], doc)[0][0], "sol")
    # perf -> sol-led with the fable/opus fallbacks (new order pinned)
    chk("perf -> sol-led", resolve(["role:perf", "area:sparq-engine"], doc)[0], ["sol", "opus5", "fable", "opus"])
    # review role -> opus + escalate
    chk("review -> opus/escalate", resolve(["role:review"], doc)[1:], ("sparq-reviewer", True))
    print("route-resolve self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


def main():
    if "--self-test" in sys.argv:
        return _self_test()
    if len(sys.argv) > 1:
        doc = tomllib.load(open("orchestration/routing.toml", "rb"))
        mc, ag, esc = resolve(sys.argv[1].split(","), doc)
        print(f"model_chain={mc} agent={ag} escalate={esc}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
