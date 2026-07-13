#!/usr/bin/env python3
# [OPUS-4.8] Issue-native orchestration: the untrusted-input safeguard.
"""trust-gate — decide whether the automation may act on an issue/PR/comment's CONTENT.

Fails CLOSED. The automation only acts on content authored by a TRUSTED identity (the maintainer or
an automation identity). Third-party content is `untrusted` and must NOT be fed to any model — until
the maintainer explicitly promotes it with a 👍 reaction (see research/issue-native-orchestration.md).

Verdicts:
  trusted    — author is the maintainer / automation identity → act normally.
  promoted   — third-party author, but the maintainer 👍-approved it → act (maintainer opted in).
  untrusted  — third-party, unapproved → take NO model action on its content; quarantine + notify.

Exit code: 0 if actionable (trusted|promoted), 3 if untrusted — so a workflow step can gate on it.

Usage:
  trust-gate.py --author <login> --trusted a,b,c [--maintainer-approved]
  trust-gate.py --self-test
"""
import argparse
import sys


def verdict(author: str, trusted, maintainer_approved: bool) -> str:
    """Pure decision. `trusted` is an iterable of trusted logins; comparison is case-insensitive."""
    a = (author or "").strip().lower()
    tl = {str(t).strip().lower() for t in trusted if str(t).strip()}
    if a and a in tl:
        return "trusted"
    if maintainer_approved:
        return "promoted"
    return "untrusted"


def actionable(v: str) -> bool:
    return v in ("trusted", "promoted")


def _self_test() -> int:
    T = ["jeswr", "sparq-bot[bot]"]
    cases = [
        # (author, trusted, approved) -> expected verdict
        ("jeswr", T, False, "trusted"),
        ("JESWR", T, False, "trusted"),            # case-insensitive
        ("sparq-bot[bot]", T, False, "trusted"),   # automation identity
        ("randoperson", T, False, "untrusted"),    # third-party, unapproved
        ("randoperson", T, True, "promoted"),      # third-party, maintainer 👍
        ("", T, False, "untrusted"),               # missing author fails closed
        ("", T, True, "promoted"),                 # explicit approval still promotes
        ("jeswr", [], False, "untrusted"),         # empty trusted set → nobody trusted
    ]
    ok = True
    for author, trusted, approved, want in cases:
        got = verdict(author, trusted, approved)
        flag = "ok  " if got == want else "FAIL"
        if got != want:
            ok = False
        print(f"  {flag} author={author!r:16} approved={approved!s:5} -> {got} (want {want})")
    # actionable() consistency
    assert actionable("trusted") and actionable("promoted") and not actionable("untrusted")
    print("trust-gate self-test", "PASSED" if ok else "FAILED")
    return 0 if ok else 1


def main() -> int:
    ap = argparse.ArgumentParser(description="Untrusted-input safeguard for issue-native orchestration")
    ap.add_argument("--author", default="")
    ap.add_argument("--trusted", default="", help="comma-separated trusted logins")
    ap.add_argument("--maintainer-approved", action="store_true",
                    help="the maintainer has 👍-approved this third-party item")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()
    if args.self_test:
        return _self_test()
    trusted = [t for t in args.trusted.split(",") if t.strip()]
    v = verdict(args.author, trusted, args.maintainer_approved)
    print(v)
    return 0 if actionable(v) else 3


if __name__ == "__main__":
    sys.exit(main())
