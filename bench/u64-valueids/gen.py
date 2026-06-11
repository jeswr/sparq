#!/usr/bin/env python3
"""T3 measurement fixture: literal-heavy synthetic data for quantifying the
dict-resolve cost in BIND/FILTER/GROUP BY/ORDER BY today (the WIN CEILING of
inlining more value types into the id).

Per subject (N subjects): one xsd:integer in the INLINE range, one above it
(dict + numerics-cache path), one xsd:decimal (dict + numerics-cache path),
one xsd:dateTime (dict path, term-materialised per evaluation — NO cache),
one group key. ~5N triples.

Usage: gen.py <N> <out.nt>
"""
import sys

n = int(sys.argv[1]) if len(sys.argv) > 1 else 1_000_000
out = sys.argv[2] if len(sys.argv) > 2 else "/tmp/t3-literals.nt"

XSD = "http://www.w3.org/2001/XMLSchema#"
with open(out, "w") as f:
    for i in range(n):
        s = f"<http://ex/s{i}>"
        v = i % 100_000
        # inline-range integer (canonical, non-negative, < 2^30)
        f.write(f'{s} <http://ex/int> "{v}"^^<{XSD}integer> .\n')
        # big integer: above the 2^30 inline cap -> dictionary + numerics cache
        f.write(f'{s} <http://ex/big> "{2_000_000_000 + v}"^^<{XSD}integer> .\n')
        # decimal -> dictionary + numerics cache
        f.write(f'{s} <http://ex/dec> "{v}.5"^^<{XSD}decimal> .\n')
        # dateTime -> dictionary, term-materialised per FILTER/ORDER evaluation
        yr = 2000 + (i % 25)
        mo = 1 + (i % 12)
        dy = 1 + (i % 28)
        f.write(f'{s} <http://ex/date> "{yr:04d}-{mo:02d}-{dy:02d}T12:00:00Z"^^<{XSD}dateTime> .\n')
        # group key (40 groups)
        f.write(f"{s} <http://ex/group> <http://ex/g{i % 40}> .\n")
print(f"wrote {5*n} triples to {out}")
