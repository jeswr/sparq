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


# [OPUS-5] DEPRECATION REGISTER (maintainer directive 2026-07-26: "deprecate the use of fable and
# opus entirely in favour of opus5"). These are the aliases and the concrete provider ids that must
# never again occupy a routing position. Keeping the ids here — not only the aliases — is what makes
# the deprecation stick: re-adding `[models.fable]` under a DIFFERENT alias name still trips the
# provider-id half of the guard.
DEPRECATED_ALIASES = {"fable", "opus"}
DEPRECATED_PROVIDER_MODELS = {"claude-fable-5", "claude-opus-4-8"}

# [OPUS-5] Cheap anthropic tiers, deprecated FOR DOCS WRITING on 2026-07-26 ("deprecate sonnet and
# haiku for docs writing in favor of gpt 5.6 sol"). role=docs was their ONLY routing consumer, so
# after that directive they hold no routing position at all and this set is the whole rule. They are
# deliberately still in the [models] CATALOG and still serve non-routing, non-docs-writing roles in
# the harness agent configs (sparq-pkg-nl NL retrieval, sparq-verify-mechanical, sparq-rust-impl,
# sparq-issue-sweeper, sparq-context-monitor) — this guard governs the routing table only.
NOT_A_ROUTING_TARGET = {"haiku", "sonnet"}

# `terra` (the codex CLI default model) stays docs-only: it is sol's same-provider fallback in the
# docs chain and must not appear anywhere else.
DOCS_ONLY = {"terra"}


def _chains(doc):
    """Yield (where, chain) for every routing position in the table: defaults + every route."""
    yield "defaults", list(doc.get("defaults", {}).get("model_chain", []))
    for r in doc.get("route", []):
        where = ("role:" + r["role"]) if r.get("role") else \
            ("match_labels:" + ",".join(r.get("match_labels", []))) or "<unnamed>"
        yield where, list(r.get("model_chain", []))


def validate_routing(doc):
    """Structural invariants a routing table must satisfy before ANY resolution — enforced in
    resolve() so a violating table fails LOUDLY at PLAN time instead of silently routing.

    FAIL-CLOSED. Every rule here raises rather than dropping the offending alias, because the
    alternative — resolving anyway and letting the chain fall through — is exactly how a deprecated
    model keeps serving traffic after it is "removed".
    """
    models = set(doc.get("models", {}))
    errs = []

    # (1) DEPRECATION GUARD — fable / opus-4.8 may not occupy ANY routing position. This is the
    # regression guard for the 2026-07-26 directive: it is what turns the deprecation from a
    # one-time edit into an invariant.
    for name, spec in doc.get("models", {}).items():
        if name in DEPRECATED_ALIASES:
            errs.append(f"[models.{name}] is a DEPRECATED alias (maintainer 2026-07-26: fable and "
                        f"opus-4.8 are retired in favour of opus5)")
        pm = spec.get("provider_model")
        if pm in DEPRECATED_PROVIDER_MODELS:
            errs.append(f"[models.{name}] pins DEPRECATED provider_model '{pm}' (retired "
                        f"2026-07-26 in favour of claude-opus-5)")
    for where, chain in _chains(doc):
        for m in chain:
            if m in DEPRECATED_ALIASES:
                errs.append(f"{where}: model_chain names DEPRECATED alias '{m}' — use 'opus5'")

    # (2) FAIL CLOSED ON AN UNKNOWN MODEL. Previously only routing-validate.py (a separate CI step)
    # checked chain membership against the catalog, so a chain naming a model the catalog no longer
    # defines still RESOLVED — it just handed an unresolvable alias to the registry's account
    # selector, which found no account serving it and silently walked to the next rung. With
    # fable/opus now deleted from the catalog that failure mode is live, so refuse here instead.
    for where, chain in _chains(doc):
        for m in chain:
            if m not in models and m not in DEPRECATED_ALIASES:
                errs.append(f"{where}: model '{m}' is not in the [models] catalog — refusing to "
                            f"resolve (an unresolvable rung must fail, never fall through)")

    # (3) Cheap anthropic tiers hold no routing position (2026-07-26 docs-writing directive).
    for where, chain in _chains(doc):
        for m in chain:
            if m in NOT_A_ROUTING_TARGET:
                errs.append(f"{where}: model_chain names '{m}', which is not a routing target "
                            f"(docs writing moved to sol, maintainer 2026-07-26)")

    # (4) terra remains docs-only.
    if DOCS_ONLY & set(doc.get("defaults", {}).get("model_chain", [])):
        errs.append("defaults: names a docs-only model (" + ",".join(sorted(DOCS_ONLY)) + ")")
    for r in doc.get("route", []):
        if DOCS_ONLY & set(r.get("model_chain", [])) and r.get("role") != "docs":
            where = r.get("role") or ",".join(r.get("match_labels", [])) or "<unnamed>"
            errs.append(f"{where}: names a docs-only model outside role=docs")

    if errs:
        raise ValueError("routing table is invalid; refusing to resolve:\n  - " +
                         "\n  - ".join(errs))


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

    def raises(n, fn, needle):
        """Assert fn() raises ValueError whose message contains `needle`. A deprecation guard that
        cannot be shown to REJECT something is a comment, not a guard."""
        nonlocal ok
        try:
            fn()
        except ValueError as e:
            good = needle in str(e)
            ok = ok and good
            print(f"  {'ok  ' if good else 'FAIL'} {n}: raised {'with' if good else 'WITHOUT'} "
                  f"{needle!r}")
        else:
            ok = False
            print(f"  FAIL {n}: did NOT raise (expected {needle!r})")

    # impl + a security surface (area:sparq-zk) -> security rule wins over role -> Opus 5 alone
    # (the opus-4.8 tail fallback was deprecated 2026-07-26), escalate
    mc, ag, esc = resolve(["role:impl", "area:sparq-zk"], doc)
    chk("impl+zk -> opus5/escalate", (mc, ag, esc), (["opus5"], "sparq-reviewer", True))
    # plain impl -> sol-led chain (maintainer directive 2026-07-18)
    mc, ag, esc = resolve(["role:impl", "area:sparq-core"], doc)
    chk("impl -> opus5-led", (mc, ag, esc), (["opus5", "sol"], "sparq-rust-impl", False))
    # [OPUS-5] docs -> SOL-led (maintainer 2026-07-26: docs writing off haiku/sonnet onto gpt-5.6
    # sol). terra is sol's same-provider fallback; opus5 the cross-provider tail.
    chk("docs -> sol-led", resolve(["role:docs", "area:x"], doc)[0], ["sol", "terra", "opus5"])
    chk("docs chain has no cheap anthropic tier",
        sorted(set(resolve(["role:docs", "area:x"], doc)[0]) & NOT_A_ROUTING_TARGET), [])
    # [FABLE-5] UI ownership: site -> sol-led (GPT-5.6 codex, the original dashboard builder)
    # [OPUS-5] site is NOT the gui carve-out — it takes the opus5-first default (2026-07-26).
    chk("site -> opus5-led (site is outside the area:gui carve-out)",
        resolve(["role:site", "area:site"], doc)[0], ["opus5", "sol"])
    # [FABLE-5] frontier-tier infra authorship (standing rule 2026-07-17): ci -> sol-first, opus5
    # the SOLE anthropic tier since 2026-07-26. FRONTIER-ONLY chain — no sub-frontier model
    # (sonnet/haiku) anywhere in it, so exhaustion DEFERS at the registry claim step (retried next
    # tick) instead of degrading tier.
    mc, ag, esc = resolve(["role:ci", "area:ci"], doc)
    chk("ci -> frontier-only opus5-first", (mc, ag, esc), (["opus5", "sol"], "sparq-ci-infra", False))
    chk("ci chain has no sub-frontier tier", sorted(set(mc) & {"sonnet", "haiku"}), [])
    # no role -> defaults (sol-led, 2026-07-18)
    chk("no role -> defaults", resolve(["area:sparq-core"], doc)[0][0], "opus5")
    chk("perf -> opus5-led", resolve(["role:perf", "area:sparq-engine"], doc)[0], ["opus5", "sol"])

    # ---------------------------------------------------------------------------------------------
    # [OPUS-5] OPUS-5-FIRST DEFAULT + THE `area:gui` CARVE-OUT (maintainer 2026-07-26).
    # ---------------------------------------------------------------------------------------------
    # Every route where opus5 AND sol are both viable implementors must lead with opus5...
    for _role in ("impl", "site", "ci", "perf"):
        mc = resolve([f"role:{_role}"], doc)[0]
        chk(f"role:{_role} prefers opus5 over sol", mc[0], "opus5")
        chk(f"role:{_role} keeps sol reachable as a fallback (preference, not exclusion)",
            "sol" in mc, True)
    chk("defaults prefer opus5 over sol", resolve(["area:sparq-core"], doc)[0][0], "opus5")
    # ...EXCEPT role:gui, which keeps the sol lead (original-builder steer, task #331).
    gui = resolve(["role:gui", "area:gui"], doc)[0]
    chk("role:gui KEEPS sol first (the carve-out)", gui[0], "sol")
    chk("role:gui keeps opus5 reachable (GUI stays dispatchable in a sol outage)",
        "opus5" in gui, True)
    chk("role:gui routes to the site agent", resolve(["role:gui"], doc)[1], "sparq-site")
    # The carve-out is EXACTLY role:gui. role:site must NOT be swept into it — "GUI" reads
    # informally as covering the site surfaces, which is the likely future widening mistake.
    chk("role:site is NOT in the sol carve-out", resolve(["role:site"], doc)[0][0], "opus5")
    # Both directions terminate: neither class can become undispatchable.
    for _role in ("impl", "site", "ci", "perf", "gui"):
        mc = resolve([f"role:{_role}"], doc)[0]
        chk(f"role:{_role} chain is cross-provider (cannot be starved by one provider)",
            sorted({doc["models"][m]["provider"] for m in mc}), ["anthropic", "openai"])
    chk("research -> opus5 only", resolve(["role:research"], doc)[0], ["opus5"])
    # review role -> opus5 + escalate
    chk("review -> opus5/escalate", resolve(["role:review"], doc)[1:], ("sparq-reviewer", True))

    # ---------------------------------------------------------------------------------------------
    # [OPUS-5] THE DEPRECATION IS AN INVARIANT, NOT A ONE-TIME EDIT (maintainer 2026-07-26).
    # Every assertion below fails if the corresponding guard clause in validate_routing() is
    # deleted or weakened, and the LIVE-TABLE sweeps fail if a deprecated alias is reintroduced
    # into orchestration/routing.toml by any future edit.
    # ---------------------------------------------------------------------------------------------
    live_chains = {w: c for w, c in _chains(doc)}
    for where, chain in live_chains.items():
        chk(f"live table: {where} names no deprecated alias",
            sorted(set(chain) & DEPRECATED_ALIASES), [])
        chk(f"live table: {where} names no cheap anthropic tier",
            sorted(set(chain) & NOT_A_ROUTING_TARGET), [])
    chk("live catalog defines no deprecated alias",
        sorted(set(doc.get("models", {})) & DEPRECATED_ALIASES), [])
    chk("live catalog pins no deprecated provider id",
        sorted({s.get("provider_model") for s in doc.get("models", {}).values()}
               & DEPRECATED_PROVIDER_MODELS), [])
    chk("live catalog still defines opus5 -> claude-opus-5",
        doc.get("models", {}).get("opus5", {}).get("provider_model"), "claude-opus-5")
    chk("live catalog still defines sol -> gpt-5.6-sol",
        doc.get("models", {}).get("sol", {}).get("provider_model"), "gpt-5.6-sol")

    def _mutate(**kw):
        """A copy of the LIVE table with one field replaced — mutation-tests the guard against the
        real doc rather than a hand-built toy that could drift away from it."""
        import copy
        d = copy.deepcopy(doc)
        for k, v in kw.items():
            if k == "defaults_chain":
                d["defaults"]["model_chain"] = v
            elif k == "catalog":
                d["models"].update(v)
        return d

    raises("REJECTS a reintroduced `fable` rung in a chain",
           lambda: resolve(["role:impl"], _mutate(defaults_chain=["sol", "opus5", "fable"])),
           "DEPRECATED alias 'fable'")
    raises("REJECTS a reintroduced `opus` (4.8) rung in a chain",
           lambda: resolve(["role:impl"], _mutate(defaults_chain=["sol", "opus5", "opus"])),
           "DEPRECATED alias 'opus'")
    raises("REJECTS a re-added [models.fable] catalog entry",
           lambda: resolve(["role:impl"], _mutate(catalog={"fable": {
               "provider": "anthropic", "harness": "claude",
               "provider_model": "claude-fable-5", "credential_format": "claude-oauth-token"}})),
           "DEPRECATED alias")
    # the provider-id half: a retired model smuggled back under an INNOCENT alias name.
    raises("REJECTS claude-opus-4-8 smuggled back under a fresh alias",
           lambda: resolve(["role:impl"], _mutate(catalog={"legacy": {
               "provider": "anthropic", "harness": "claude",
               "provider_model": "claude-opus-4-8", "credential_format": "claude-oauth-token"}})),
           "DEPRECATED provider_model 'claude-opus-4-8'")
    raises("REJECTS claude-fable-5 smuggled back under a fresh alias",
           lambda: resolve(["role:impl"], _mutate(catalog={"legacy": {
               "provider": "anthropic", "harness": "claude",
               "provider_model": "claude-fable-5", "credential_format": "claude-oauth-token"}})),
           "DEPRECATED provider_model 'claude-fable-5'")
    raises("REJECTS haiku creeping back into a chain (docs writing moved to sol)",
           lambda: resolve(["role:impl"], _mutate(defaults_chain=["haiku", "sol"])),
           "not a routing target")
    raises("REJECTS sonnet creeping back into a chain",
           lambda: resolve(["role:impl"], _mutate(defaults_chain=["sonnet", "sol"])),
           "not a routing target")
    # FAIL CLOSED: an unresolvable rung must refuse, never fall through to the next one.
    raises("FAILS CLOSED on a model absent from the catalog",
           lambda: resolve(["role:impl"], _mutate(defaults_chain=["sol", "ghost"])),
           "not in the [models] catalog")
    raises("REJECTS terra outside role=docs",
           lambda: resolve(["role:impl"], _mutate(defaults_chain=["terra", "sol"])),
           "docs-only")

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
