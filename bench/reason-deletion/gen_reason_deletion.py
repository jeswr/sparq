#!/usr/bin/env python3
# [FABLE-5] Deletion-workload corpus generator for bench/reason-deletion (bead sq-31fza,
# parent sq-6tykl.4). Emits a DETERMINISTIC LUBM-class synthetic ABox as N-Triples on stdout.
#
#   gen_reason_deletion.py <units>
#
# "LUBM-class" = the same benchmark genre as LUBM's UBA generator: a deterministic,
# scale-parameterized instance generator over a FIXED schema with class hierarchies and
# typed (domain/range/subPropertyOf/inverseOf) properties, so an RDFS / OWL 2 RL closure
# is materially larger than the ABox and deletion maintenance has real re-derivation work.
#
# WHY the OLYMPICS vocabulary (not LUBM's univ-bench IRIs): the driver for this suite is
# the EXISTING, unmodified example crates/sparq-reason/examples/incremental_olympics_bench.rs
# (the bead forbids touching sparq-reason source), and that example SYNTHESIZES its TBox
# over the olympics vocabulary (foaf:Person / dbo:team / oly:athlete / ...). Feeding it
# actual univ-bench IRIs would make every rule a no-op and the deletion workload vacuous.
# So the generator emits LUBM-class DATA SHAPE bound to the vocabulary the driver's TBox
# actually ranges over. See README.md ("Why not literal LUBM data").
#
# Schema exercised per unit (1 unit = 1 athlete + 1 result = 8 ABox triples):
#   athlete  rdf:type foaf:Person          -> subClassOf chain Person->Agent->Entity
#                                             (+ owl:equivalentClass Human in the OWL runs)
#   athlete  dbo:team team_k               -> domain/range + subPropertyOf affiliatedWith
#                                             (+ owl:inverseOf hasMember in the OWL runs)
#   athlete  foaf:age  "N"^^xsd:integer    -> domain Person
#   athlete  foaf:gender "..."             -> domain Person
#   result   oly:athlete athlete           -> domain Result->Entity, range Person
#   result   oly:games   games_g           -> range dbo:Olympics->Event->Entity
#   result   oly:event   event_e           -> range dbo:SportsEvent->Event->Entity
#   result   oly:medal   medal_m           -> range syn:Medal, subPropertyOf award
#
# Shared entities (teams/games/events/medals) scale sub-linearly with units, like LUBM's
# departments-per-university, so the closure has both per-unit and amortized derivations.
#
# DETERMINISM: pure function of <units> — no RNG state beyond a fixed-seed LCG used only
# to vary ages/genders/assignments; byte-identical output for a given <units> every run.
# The RANDOMIZED insert/delete batches come from the driver example's own fixed-seed
# xorshift sampler, NOT from this generator. Instance IRIs live under their own namespace
# (rd:) so they can never collide with the driver's fresh-insert IRIs (syn:newAthlete_*).
#
# stdlib-only; no network.
import sys

RDF_TYPE = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
FOAF = "http://xmlns.com/foaf/0.1/"
DBO = "http://dbpedia.org/ontology/"
OLY = "http://wallscope.co.uk/ontology/olympics/"
XSD_INT = "http://www.w3.org/2001/XMLSchema#integer"
# Instance namespace DISTINCT from the driver's syn: (http://sparq.dev/bench/olympics#)
# so generated instances never collide with the example's fresh_inserts IRIs.
RD = "http://sparq.dev/bench/reason-deletion#"

GENDERS = ("female", "male", "nonbinary")
MEDALS = ("Gold", "Silver", "Bronze")


def main() -> int:
    if len(sys.argv) != 2 or not sys.argv[1].isdigit() or int(sys.argv[1]) < 1:
        print("usage: gen_reason_deletion.py <units>=positive int", file=sys.stderr)
        return 2
    units = int(sys.argv[1])
    # Shared-entity pools: sub-linear in units (LUBM-style shared structure).
    teams = max(1, units // 50)
    games = max(1, units // 400)
    events = max(1, units // 100)

    # Fixed-seed LCG (MMIX constants) — deterministic variation only.
    state = 0x5DEECE66D
    def lcg() -> int:
        nonlocal state
        state = (state * 6364136223846793005 + 1442695040888963407) % (1 << 64)
        return state >> 33

    out = sys.stdout
    w = out.write
    for i in range(units):
        a = f"<{RD}athlete{i}>"
        r = f"<{RD}result{i}>"
        team = f"<{RD}team{lcg() % teams}>"
        game = f"<{RD}games{lcg() % games}>"
        event = f"<{RD}event{lcg() % events}>"
        medal = f"<{RD}medal{MEDALS[lcg() % 3]}>"
        age = 16 + lcg() % 40
        gender = GENDERS[lcg() % 3]
        w(f"{a} <{RDF_TYPE}> <{FOAF}Person> .\n")
        w(f"{a} <{DBO}team> {team} .\n")
        w(f'{a} <{FOAF}age> "{age}"^^<{XSD_INT}> .\n')
        w(f'{a} <{FOAF}gender> "{gender}" .\n')
        w(f"{r} <{OLY}athlete> {a} .\n")
        w(f"{r} <{OLY}games> {game} .\n")
        w(f"{r} <{OLY}event> {event} .\n")
        w(f"{r} <{OLY}medal> {medal} .\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
