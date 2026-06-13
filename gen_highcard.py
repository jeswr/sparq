# High-distinct-cardinality N-Triples generator (the dict-heavy regime).
# Every line introduces UNIQUE subject + UNIQUE literal object so distinct-term
# count grows ~linearly with triple count (worst case for the build-time dictionary).
# Usage: python3 gen_highcard.py <n_triples> > out.nt
import sys

n = int(sys.argv[1])
w = sys.stdout.write
P = "<http://ex/p>"
# A handful of predicates reused; subjects + objects unique -> ~2 distinct terms/triple.
buf = []
for i in range(n):
    # unique IRI subject, unique long literal object (uuid-ish unique string)
    buf.append(f"<http://ex/resource/item{i}> {P} \"unique-literal-value-{i}-payload-aaaaaaaa\" .\n")
    if len(buf) >= 100000:
        w("".join(buf)); buf = []
w("".join(buf))
