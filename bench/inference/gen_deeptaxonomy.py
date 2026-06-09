#!/usr/bin/env python3
"""Generate the DeepTaxonomy (DT) rule-heavy inference benchmark in Notation3.
One instance fact + a deep subClassOf chain + a transitivity rule — the canonical test
where NAIVE forward chaining blows up (O(N^2)) and semi-naive + indexing is linear.
  python3 gen_deeptaxonomy.py 100000 > dt100k.n3   ;   sparq-cli reason dt100k.n3 n3 n3
"""
import sys
N = int(sys.argv[1]) if len(sys.argv) > 1 else 1000
print('@prefix : <http://ex/> .')
print(':i a :n0 .')
print('{ ?x a ?c . ?c :sc ?d } => { ?x a ?d } .')
for k in range(N):
    print(f':n{k} :sc :n{k+1} .')
