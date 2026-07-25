#!/usr/bin/env python3
# [OPUS-4.8] Validate orchestration/routing.toml against its model catalog (review D2).
"""routing-validate.py — schema-check the routing table.

Fails if: a model in any chain (defaults or a route) is not in [models]; a chain is empty; a route
lacks an agent; a route has neither `role` nor `match_labels`; or a model catalog entry is missing a
required field. Run in CI to prevent a misspelled model/role or an empty chain from shipping.
"""
import sys

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib

CATALOG_REQUIRED = ("provider", "harness", "provider_model", "credential_format")
# [FABLE-5] STANDING RULE (maintainer 2026-07-17): CI/infrastructure work is authored by a
# FRONTIER-tier model only (fable / terra). Validation enforces the floor structurally: a role:ci
# chain containing a sub-frontier tier fails CI, so exhaustion can only ever DEFER, never degrade.
SUB_FRONTIER = ("sonnet", "haiku")


def validate(doc):
    errs = []
    models = doc.get("models", {})
    if not models:
        errs.append("no [models] catalog")
    for name, spec in models.items():
        for f in CATALOG_REQUIRED:
            if f not in spec:
                errs.append(f"model {name}: missing '{f}'")

    def check_chain(where, chain):
        if not chain:
            errs.append(f"{where}: empty model_chain")
        for m in chain:
            if m not in models:
                errs.append(f"{where}: model '{m}' not in [models] catalog")

    defaults = doc.get("defaults", {})
    check_chain("defaults", defaults.get("model_chain", []))
    if not defaults.get("agent"):
        errs.append("defaults: missing agent")
    for i, r in enumerate(doc.get("route", [])):
        where = f"route[{i}]"
        if "role" not in r and "match_labels" not in r:
            errs.append(f"{where}: needs `role` or `match_labels`")
        if not r.get("agent"):
            errs.append(f"{where}: missing agent")
        check_chain(where, r.get("model_chain", []))
        # [FABLE-5] frontier-tier floor for CI/infrastructure authorship (standing rule 2026-07-17)
        if r.get("role") == "ci":
            for m in r.get("model_chain", []):
                if m in SUB_FRONTIER:
                    errs.append(f"{where}: role:ci chain contains sub-frontier model '{m}' "
                                "(frontier-tier infra-authorship rule: fable/terra only)")
    return errs


def _self_test():
    good = {
        "models": {"fable": {"provider": "a", "harness": "claude", "provider_model": "x",
                             "credential_format": "y"}},
        "defaults": {"model_chain": ["fable"], "agent": "sparq-rust-impl"},
        "route": [{"role": "impl", "model_chain": ["fable"], "agent": "sparq-rust-impl"},
                  # a frontier-only ci chain is valid (frontier-tier infra-authorship rule)
                  {"role": "ci", "model_chain": ["fable"], "agent": "sparq-ci-infra"}],
    }
    bad = {
        "models": {"fable": {"provider": "a"},  # missing fields
                   "sonnet": {"provider": "a", "harness": "claude", "provider_model": "x",
                              "credential_format": "y"}},
        "defaults": {"model_chain": ["ghost"], "agent": ""},  # unknown model + no agent
        "route": [{"model_chain": [], "agent": "x"},  # empty chain + no role/labels
                  # sub-frontier model in a role:ci chain -> frontier-floor violation
                  {"role": "ci", "model_chain": ["sonnet"], "agent": "sparq-ci-infra"}],
    }
    bad_errs = validate(bad)
    ok = (not validate(good) and len(bad_errs) >= 5
          and any("sub-frontier" in e for e in bad_errs))
    print("good doc errors:", validate(good))
    print("bad doc errors :", validate(bad))
    print("routing-validate self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


def main():
    if "--self-test" in sys.argv:
        return _self_test()
    path = sys.argv[1] if len(sys.argv) > 1 else "orchestration/routing.toml"
    errs = validate(tomllib.load(open(path, "rb")))
    if errs:
        print(f"routing.toml INVALID ({len(errs)}):")
        for e in errs:
            print("  -", e)
        return 1
    print("routing.toml OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
