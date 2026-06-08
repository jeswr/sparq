#!/usr/bin/env python3
"""Generate a SELECTIVE join benchmark: a dense `follows` graph with a RARE `premium`
predicate (0.1% of nodes). The 3-pattern chain
  ?a follows ?b . ?b follows ?c . ?c premium ?p
returns only a few thousand rows, but a naive plan scans the ~millions-row `follows`
relation for the merge — the case the index-nested-loop (bind) join fixes.
  python3 gen.py [N] > selective.nt
"""
import random, sys
random.seed(7)
N = int(sys.argv[1]) if len(sys.argv) > 1 else 500_000
prem = set(random.sample(range(N), max(1, N // 1000)))
for i in range(N):
    for _ in range(4):
        print(f"<http://ex/n{i}> <http://ex/follows> <http://ex/n{random.randrange(N)}> .")
    if i in prem:
        print(f'<http://ex/n{i}> <http://ex/premium> "1"^^<http://www.w3.org/2001/XMLSchema#integer> .')
