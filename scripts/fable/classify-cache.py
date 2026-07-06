#!/usr/bin/env python3
# classify-cache.py — [OPUS-4.8] Fable context-broker: hash-keyed classification cache.
#
# Persists, per file, the signal used to decide whether Claude Fable can read it
# directly or must go through a benign prose digest (see
# research/fable-context-broker.md). The cache is keyed by path and invalidated
# on content change (sha256), so a file is only (re)classified after it is
# modified.
#
# Cache entry schema (JSON), keyed by path:
#     {
#       "content_sha256": "<hex>",
#       "observed_tier":  "fable" | "downgraded" | null,
#       "classification": "benign_prose" | "code" | "security" | "unknown",
#       "source":         "observed" | "predicted",
#       "updated":        "<ISO-8601 UTC>"
#     }
#
# AUTHORITATIVE signal = the OBSERVED downgrade from a real run (source=observed);
# an a-priori classification is only an advisory prior (source=predicted) — a
# security-term-free code file (numeric.rs) still trips the downgrade.
#
# Pure standard library. Advisory tooling only — NOT wired into the CI gate.
#
# Usage:
#   classify-cache.py get         <path>
#   classify-cache.py set         <path> [--observed-tier T] [--classification C] [--source S]
#   classify-cache.py stale-check <path>
#   classify-cache.py list
#
# Cache location: $FABLE_CACHE_PATH, else --cache-path, else
#   .claude/fable/file-classifications.json (relative to cwd).

import argparse
import datetime
import hashlib
import json
import os
import sys

DEFAULT_CACHE_PATH = os.path.join(".claude", "fable", "file-classifications.json")

OBSERVED_TIERS = ("fable", "downgraded")            # or null
CLASSIFICATIONS = ("benign_prose", "code", "security", "unknown")
SOURCES = ("observed", "predicted")


def cache_path(args):
    """Resolve the cache file path: --cache-path > $FABLE_CACHE_PATH > default."""
    if getattr(args, "cache_path", None):
        return args.cache_path
    return os.environ.get("FABLE_CACHE_PATH", DEFAULT_CACHE_PATH)


def load_cache(path):
    """Return the cache dict, or {} if the file is absent/empty."""
    try:
        with open(path, "r", encoding="utf-8") as fh:
            text = fh.read()
    except FileNotFoundError:
        return {}
    if not text.strip():           # absent handled above; empty/whitespace-only file
        return {}
    data = json.loads(text)
    if not isinstance(data, dict):
        raise ValueError("cache root is not a JSON object: %s" % path)
    return data


def save_cache(path, cache):
    """Write the cache dict atomically-ish, creating parent dirs as needed."""
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)
    tmp = path + ".tmp"
    with open(tmp, "w", encoding="utf-8") as fh:
        json.dump(cache, fh, indent=2, sort_keys=True)
        fh.write("\n")
    os.replace(tmp, path)


def sha256_of_file(path):
    """Hex sha256 of a file's bytes."""
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(65536), b""):
            h.update(chunk)
    return h.hexdigest()


def now_iso():
    return datetime.datetime.now(datetime.timezone.utc).replace(microsecond=0).isoformat()


def cmd_get(args):
    cache = load_cache(cache_path(args))
    entry = cache.get(args.path)
    if entry is None:
        print("null")
        return 1
    print(json.dumps(entry, indent=2, sort_keys=True))
    return 0


def cmd_set(args):
    if not os.path.isfile(args.path):
        sys.stderr.write("error: no such file to hash: %s\n" % args.path)
        return 2

    observed_tier = args.observed_tier
    if observed_tier is not None and observed_tier not in OBSERVED_TIERS:
        sys.stderr.write("error: --observed-tier must be one of %s (or omit for null)\n"
                         % ", ".join(OBSERVED_TIERS))
        return 2
    if args.classification not in CLASSIFICATIONS:
        sys.stderr.write("error: --classification must be one of %s\n"
                         % ", ".join(CLASSIFICATIONS))
        return 2

    # Convenience: if the caller supplies an observed tier but no explicit
    # source, the signal is OBSERVED; otherwise it is a PREDICTED prior.
    source = args.source
    if source is None:
        source = "observed" if observed_tier is not None else "predicted"
    if source not in SOURCES:
        sys.stderr.write("error: --source must be one of %s\n" % ", ".join(SOURCES))
        return 2

    path = cache_path(args)
    cache = load_cache(path)
    cache[args.path] = {
        "content_sha256": sha256_of_file(args.path),
        "observed_tier": observed_tier,
        "classification": args.classification,
        "source": source,
        "updated": now_iso(),
    }
    save_cache(path, cache)
    print(json.dumps(cache[args.path], indent=2, sort_keys=True))
    return 0


def cmd_stale_check(args):
    """Print 'true' if the file's current sha differs from the cached sha.

    A missing cache entry counts as stale (true) — the file has never been
    classified, so classification must run.
    """
    if not os.path.isfile(args.path):
        sys.stderr.write("error: no such file to hash: %s\n" % args.path)
        return 2
    cache = load_cache(cache_path(args))
    entry = cache.get(args.path)
    if entry is None or entry.get("content_sha256") != sha256_of_file(args.path):
        print("true")
    else:
        print("false")
    return 0


def cmd_list(args):
    cache = load_cache(cache_path(args))
    print(json.dumps(cache, indent=2, sort_keys=True))
    return 0


def build_parser():
    p = argparse.ArgumentParser(
        description="Fable context-broker hash-keyed file-classification cache "
                    "(advisory; not wired into CI).")
    p.add_argument("--cache-path", default=None,
                   help="cache JSON path (default: $FABLE_CACHE_PATH or %s)"
                        % DEFAULT_CACHE_PATH)
    sub = p.add_subparsers(dest="cmd", required=True)

    g = sub.add_parser("get", help="print the cache entry for <path> (null if absent)")
    g.add_argument("path")
    g.set_defaults(func=cmd_get)

    s = sub.add_parser("set", help="hash <path> and upsert its classification entry")
    s.add_argument("path")
    s.add_argument("--observed-tier", choices=OBSERVED_TIERS, default=None,
                   help="tier observed to serve a run that read this file (omit for null)")
    s.add_argument("--classification", choices=CLASSIFICATIONS, default="unknown")
    s.add_argument("--source", choices=SOURCES, default=None,
                   help="default: 'observed' if --observed-tier given, else 'predicted'")
    s.set_defaults(func=cmd_set)

    sc = sub.add_parser("stale-check",
                        help="print true if <path>'s content sha != cached sha")
    sc.add_argument("path")
    sc.set_defaults(func=cmd_stale_check)

    ls = sub.add_parser("list", help="print the whole cache")
    ls.set_defaults(func=cmd_list)
    return p


def main(argv=None):
    args = build_parser().parse_args(argv)
    # Canonicalize the path once so the same file keys to one cache entry
    # regardless of how the caller spelled it (relative vs absolute, ./ , .. ).
    if getattr(args, "path", None) is not None:
        args.path = os.path.abspath(args.path)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
