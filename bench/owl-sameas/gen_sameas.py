#!/usr/bin/env python3
# [OPUS-4.8] OWL sameAs equality micro-suite generator (sq-msl6, design A3). Emits, in N-Triples,
# K INDEPENDENT owl:sameAs equivalence classes, each of N individuals declared equal via a STAR of
# (N-1) sameAs assertions to the class anchor, plus M shared "data" triples asserted ONLY on the
# anchor (anchor :p<j> :v<j>). It is a pure function of (K, N, M) — no RNG, byte-identical every
# run — so the OWL-RL closure size it induces is a CLOSED FORM (see below), making this a
# zero-download, deterministic hard gate on the reasoner's owl:sameAs equality handling.
#
#   python3 gen_sameas.py [K=4] [N=8] [M=3] > sameas.nt
#
# CLOSED-FORM CLOSURE SIZE after `sparq-cli reason <f> ntriples owl` (VERIFIED empirically against
# the engine, not assumed — sparq-reason materialises owl:sameAs by union-find entity rewriting and
# then EXPANDS each class back to the full closure; crates/sparq-reason/src/owl.rs):
#   closure_triples = K * N * (N + M)
#     - sameAs relation per class: the FULL N*N ordered pairs INCLUDING the reflexive pairs
#       (the engine emits eq-ref restricted to the terms equality touches) -> K * N^2.
#     - each of the M anchor data triples (anchor :p<j> :v<j>) re-emits for every class member
#       (eq-rep on the subject; :p<j>/:v<j> are singletons) -> K * N * M.
#   Total = K*N^2 + K*N*M = K * N * (N + M).
#
# The class-membership probe `SELECT ?x WHERE { ?x :p0 :v0 }` (query.rq) returns one row per member
# of every class, i.e. K * N rows — every member inherits the anchor's data triple via sameAs, the
# whole point of the suite (returns 0 on the raw data; only correct after reasoning).
#
# WHY a STAR (not a clique) of sameAs edges: N-1 edges suffice to merge a class of N (union-find is
# transitive+symmetric), keeping the INPUT linear in N while the CLOSURE is quadratic in N — the
# under-derivation a single-query gate would miss is exactly the missing N^2 sameAs pairs, which the
# closure_triples gate catches deterministically (design A5: closure_triples is the most valuable
# reasoner gate). Data values :v<j> are shared across classes but are objects (singletons in the
# eq-rep expansion), so they add no cross-class terms — the closed form is exact.
import sys

K = int(sys.argv[1]) if len(sys.argv) > 1 else 4
N = int(sys.argv[2]) if len(sys.argv) > 2 else 8
M = int(sys.argv[3]) if len(sys.argv) > 3 else 3
SAME_AS = "http://www.w3.org/2002/07/owl#sameAs"

for c in range(K):
    anchor = f"http://ex/c{c}_e0"
    # (N-1) star edges: anchor sameAs every other member of the class.
    for e in range(1, N):
        print(f"<{anchor}> <{SAME_AS}> <http://ex/c{c}_e{e}> .")
    # M shared data triples on the anchor only (inherited by all members via sameAs).
    for p in range(M):
        print(f"<{anchor}> <http://ex/p{p}> <http://ex/v{p}> .")
