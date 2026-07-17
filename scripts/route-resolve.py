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


def resolve(labels, doc):
    """Return (model_chain, agent, escalate). `labels`: iterable of the issue's labels."""
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

    # impl + a security surface (area:sparq-zk) -> security rule wins over role -> Opus, escalate
    mc, ag, esc = resolve(["role:impl", "area:sparq-zk"], doc)
    chk("impl+zk -> opus/escalate", (mc, ag, esc), (["opus"], "sparq-reviewer", True))
    # plain impl -> Fable-led chain
    mc, ag, esc = resolve(["role:impl", "area:sparq-core"], doc)
    chk("impl -> fable-led", (mc[0], ag, esc), ("fable", "sparq-rust-impl", False))
    # docs -> haiku-led
    chk("docs -> haiku", resolve(["role:docs", "area:x"], doc)[0][0], "haiku")
    # [FABLE-5] UI ownership: site -> terra-led (GPT-5.6 codex, the original dashboard builder)
    chk("site -> terra-led", resolve(["role:site", "area:site"], doc)[0], ["terra", "fable", "sonnet"])
    # no role -> defaults (fable-led)
    chk("no role -> defaults", resolve(["area:sparq-core"], doc)[0][0], "fable")
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
