#!/usr/bin/env python3
"""Generate a TWO-SIDED selective join benchmark: a dense `follows` graph with TWO
independent rare predicates, `premA` and `premC` (each 0.1% of nodes).

The query
  ?a premA ?x . ?a follows ?b . ?b follows ?c . ?c premC ?y
has selectivity at BOTH ends but a DENSE middle. A bind join seeded from one end
(say premA) must still expand a -> b -> c through the high-fanout `follows` BEFORE the
premC constraint at the far end can prune — so the intermediate at `c` blows up to
~|premA| * fanout^2 even though the final result is tiny. This is the case a bitmap
semi-join / predicate-transfer pre-filter targets: intersect the reachable `c` set with
premC's domain bitmap up front, pruning the middle before materializing.

  python3 gen2.py [N] > selective2.nt
"""
import random, sys
random.seed(11)
N = int(sys.argv[1]) if len(sys.argv) > 1 else 500_000
FAN = int(sys.argv[2]) if len(sys.argv) > 2 else 4
premA = set(random.sample(range(N), max(1, N // 1000)))
premC = set(random.sample(range(N), max(1, N // 1000)))
out = sys.stdout.write
for i in range(N):
    for _ in range(FAN):
        out(f"<http://ex/n{i}> <http://ex/follows> <http://ex/n{random.randrange(N)}> .\n")
    if i in premA:
        out(f'<http://ex/n{i}> <http://ex/premA> "1"^^<http://www.w3.org/2001/XMLSchema#integer> .\n')
    if i in premC:
        out(f'<http://ex/n{i}> <http://ex/premC> "1"^^<http://www.w3.org/2001/XMLSchema#integer> .\n')
