#!/usr/bin/env bash
# sparq-reason (N3 engine) vs EYE / cwm / jen3 — same machine, same workloads, wall
# seconds (sparq min-of-3; competitors min-of-3 on the quick cells, single run on the
# minutes-long ones — see the loop). Every engine parses the document, runs the
# forward closure, and SERIALIZES the closure (sparq: `reason <f> n3 n3 /dev/null`;
# EYE: `--pass > /dev/null`; cwm: `--think --data > /dev/null`; jen3:
# `-conclusion > /dev/null`), so the comparison covers the whole pipeline.
#
#   EYE=$HOME/.local/bin/eye SPARQ_CLI=target/release/sparq-cli \
#     bench/inference/eye-comparison.sh
#
# Columns (sq-hmd7l.11) [FABLE-5]:
#   EYE is the PINNED REFERENCE competitor — required (the run fails early and
#   loudly without it). cwm and jen3 are OPTIONAL columns: an absent tool prints
#   'absent' in its column and the run stays green (graceful skip).
#     CWM=cwm                                   cwm executable (W3C SWAP, Python)
#     JEN3_JAR=$HOME/.local/opt/jen3/jen3.jar   jen3 jar (william-vw/jen3 — a Java/
#                                               Apache-Jena fork with N3 support; NOT an npm lib)
#     JAVA=java                                 JVM used to run the jen3 jar
#     CWM_HEAVY=1     also run cwm on dt10k/anc500/grid30 (cwm is the honest SLOW
#                     column — literature places it at minutes on DT-1000 already,
#                     so the bigger cells are minutes-to-hours: opt-in only)
#     EYE_DT100K=1 / JEN3_DT100K=1 / CWM_DT100K=1   run dt100k (hours+ — never by default)
#
# CORRECTNESS GATE (self-asserting, before timing) [FABLE-5] (sq-hmd7l.11):
# mirroring bench/deep-taxonomy/run.sh, every workload's sparq closure count is
# asserted against its DETERMINISTIC structural expected size, and every present
# competitor's closure is cross-checked against that same expected count BEFORE its
# cell is timed — a wrong closure fails the run loudly (timing a wrong closure is
# worse than no number). Counting is done by sparq's own N3 parser over a RULE-FREE
# document (asserted: with `=>` rules present, sparq's re-derivation could MASK an
# under-deriving competitor, so the counter refuses rule-carrying input).
#
# Workloads (generated here, scalable):
#   socrates   — vendored EYE case (startup/latency floor)
#   dt1k/10k/100k — DeepTaxonomy (gen_deeptaxonomy.py): 1 instance, N-deep
#                subClassOf chain, one transitivity meta-rule
#   anc500     — transitive ancestor chain, 500 links (quadratic closure ~125k)
#   grid30     — 30x30 grid reachability (edge → reach; reach+edge → reach)
# Results: bench/inference/eye-comparison.md
#
# [OPUS-4.8] (sq-k5qq) Strictly-additive machine-readable emit: set SPARQ_INFER_JSON to a
# path and the run ALSO writes the SAME per-workload wall seconds to that file as a stable,
# hand-built JSON document (the shell analogue of the Rust harnesses' dependency-free
# `format!`-JSON). STDOUT (the printf table) is byte-for-byte unchanged whether or not the
# env var is set; the seconds are best-effort, MEASURED on the running host — ADVISORY +
# NON-CANONICAL (stated in the emitted `note`) — nothing is committed.
set -euo pipefail
cd "$(dirname "$0")/../.."
CLI="${SPARQ_CLI:-target/release/sparq-cli}"
EYE="${EYE:-eye}"
CWM="${CWM:-cwm}"
JEN3_JAR="${JEN3_JAR:-$HOME/.local/opt/jen3/jen3.jar}"
JAVA="${JAVA:-java}"
JSON_OUT="${SPARQ_INFER_JSON:-}"
JSON_ROWS=""   # accumulated `{ ... }` row objects, comma-joined at the end
B="$(mktemp -d)"; trap 'rm -rf "$B"' EXIT

# EYE is the pinned reference — fail early and clearly rather than mid-table.
if ! command -v "$EYE" >/dev/null 2>&1; then
  echo "ERROR: EYE not found ('$EYE') — EYE is the pinned reference column;" \
       "install eyereasoner/eye (install.sh) or set EYE=/path/to/eye" >&2
  exit 1
fi
# cwm / jen3 are optional: absent => the whole column reports 'absent' (graceful skip).
HAVE_CWM=0
if command -v "$CWM" >/dev/null 2>&1; then HAVE_CWM=1; else
  echo "note: cwm not found ('$CWM') — cwm column reports 'absent' (set CWM=/path/to/cwm)" >&2
fi
HAVE_JEN3=0
if command -v "$JAVA" >/dev/null 2>&1 && [ -f "$JEN3_JAR" ]; then HAVE_JEN3=1; else
  echo "note: jen3 not found (JAVA='$JAVA', JEN3_JAR='$JEN3_JAR') — jen3 column reports 'absent'" >&2
fi

cp crates/sparq-reason/tests/eye/socrates.n3 "$B/socrates.n3"
python3 bench/inference/gen_deeptaxonomy.py 1000   > "$B/dt1k.n3"
python3 bench/inference/gen_deeptaxonomy.py 10000  > "$B/dt10k.n3"
python3 bench/inference/gen_deeptaxonomy.py 100000 > "$B/dt100k.n3"
{ echo '@prefix : <http://ex/> .'
  echo '{ ?x :ancestor ?y . ?y :ancestor ?z } => { ?x :ancestor ?z } .'
  for j in $(seq 1 500); do echo ":p$j :ancestor :p$((j+1)) ."; done; } > "$B/anc500.n3"
python3 - > "$B/grid30.n3" <<'EOF'
N = 30
print('@prefix : <http://ex/> .')
print('{ ?x :edge ?y } => { ?x :reach ?y } .')
print('{ ?x :reach ?y . ?y :edge ?z } => { ?x :reach ?z } .')
for i in range(N):
    for j in range(N):
        if i + 1 < N: print(f':n{i}_{j} :edge :n{i+1}_{j} .')
        if j + 1 < N: print(f':n{i}_{j} :edge :n{i}_{j+1} .')
EOF

# ---- deterministic structural corpus constants (bench/deep-taxonomy expected.tsv
# analogue — these are corpus SHAPES, not performance numbers) [FABLE-5] ------------
# ground facts:  socrates 2 (vendored);  dtN = N+1 (1 instance + N `:sc` links);
#                anc500 = 500 chain links;  grid30 = 2·30·29 = 1,740 edges.
# closure:       socrates 3 (+1 derived);  dtN = 2N+1 (instance typed up the chain);
#                anc500 = 501·500/2 = 125,250 (all ordered chain pairs);
#                grid30 = 1,740 edges + 215,325 reach = 217,065.
ground_facts() {
  case "$1" in
    socrates) echo 2;; dt1k) echo 1001;; dt10k) echo 10001;; dt100k) echo 100001;;
    anc500) echo 500;; grid30) echo 1740;;
  esac
}
expected_closure() {
  case "$1" in
    socrates) echo 3;; dt1k) echo 2001;; dt10k) echo 20001;; dt100k) echo 200001;;
    anc500) echo 125250;; grid30) echo 217065;;
  esac
}

mon() { # min-of-N wall seconds: mon <runs> <cmd…>
  local runs="$1"; shift
  local best=""
  for _ in $(seq 1 "$runs"); do
    local t0 t1 t
    t0=$(python3 -c 'import time; print(time.time())')
    "$@" > /dev/null 2>&1
    t1=$(python3 -c 'import time; print(time.time())')
    t=$(python3 -c "print(f'{$t1-$t0:.3f}')")
    if [ -z "$best" ] || python3 -c "exit(0 if $t < $best else 1)"; then best="$t"; fi
  done
  echo "$best"
}

# Ground-triple count of a RULE-FREE N3 document, counted by sparq's own N3 parser:
# with no `=>` rules present the N3 closure IS the document, so the self-reported
# closure count is a pure parse+dedup count. Rule-freedom is ASSERTED first — over a
# rule-carrying document sparq would RE-DERIVE the closure and mask an under-deriving
# competitor, so the counter fails closed instead. [FABLE-5] (sq-hmd7l.11)
n3_ground_count() {
  if grep -q '=>' "$1"; then
    echo "ERROR: $1 contains N3 rules ('=>') — refusing to count (re-derivation would mask a bad closure)" >&2
    return 1
  fi
  "$CLI" reason "$1" n3 n3 2>/dev/null | grep -oE '^[0-9]+' | head -1
}

gate() { # gate <engine> <workload> <got> <want> — the self-asserting closure gate
  if [ "$3" != "$4" ]; then
    echo "ERROR: $1 closure cross-check FAILED on $2: got ${3:-nothing}, expected $4 — refusing to time a wrong closure" >&2
    exit 1
  fi
}

runs_for() { # min-of-3 on the quick cells, single run on the minutes-long ones
  case "$1" in dt10k|dt100k|anc500|grid30) echo 1;; *) echo 3;; esac
}

# Default competitor cell sets — conservative, heavy cells opt-in:
#  * EYE  runs everything but dt100k (≈9 h at its observed ≈N^1.95 DT scaling; EYE_DT100K=1).
#  * jen3 runs everything but dt100k (unmeasured at that depth; JEN3_DT100K=1) —
#    its smaller cells were verified tractable on a dev box before defaulting them in.
#  * cwm  runs only socrates + dt1k (the honest SLOW column — literature places cwm
#    at minutes on DT-1000 already); CWM_HEAVY=1 adds dt10k/anc500/grid30 and
#    CWM_DT100K=1 the hours+ dt100k cell.
eye_cell_on()  { [ "$1" != dt100k ] || [ -n "${EYE_DT100K:-}" ]; }
jen3_cell_on() { [ "$1" != dt100k ] || [ -n "${JEN3_DT100K:-}" ]; }
cwm_cell_on() {
  case "$1" in
    socrates|dt1k) return 0;;
    dt100k) [ -n "${CWM_DT100K:-}" ];;
    *) [ -n "${CWM_HEAVY:-}" ];;
  esac
}

# [OPUS-4.8] (sq-k5qq) / [FABLE-5] (sq-hmd7l.11) One JSON fragment per engine column:
# `json_col <name> <value>` where value is numeric seconds, 'skipped' (cell gated off)
# or 'absent' (tool not installed). Workload/engine names are static idents, so a
# JSON-safe quote is enough; numeric seconds are emitted as bare JSON numbers. The
# eye field names are unchanged from the original sq-k5qq shape; cwm/jen3 fields are
# strictly additive.
json_col() {
  case "$2" in
    absent)  echo "\"$1_seconds\": null, \"$1_skipped\": true, \"$1_absent\": true";;
    skipped) echo "\"$1_seconds\": null, \"$1_skipped\": true";;
    *)       echo "\"$1_seconds\": $2, \"$1_skipped\": false";;
  esac
}
add_json_row() { # add_json_row <workload> <sparq_s> <eye> <cwm> <jen3>
  [ -n "$JSON_OUT" ] || return 0
  local row
  row="{ \"workload\": \"$1\", \"sparq_seconds\": $2, $(json_col eye "$3"), $(json_col cwm "$4"), $(json_col jen3 "$5") }"
  if [ -z "$JSON_ROWS" ]; then JSON_ROWS="$row"; else JSON_ROWS="$JSON_ROWS,$row"; fi
}

cell() { # print one right-aligned table cell; numeric seconds get an 's' suffix
  case "$1" in absent|skipped) printf ' %12s' "$1";; *) printf ' %11ss' "$1";; esac
}

printf '%-10s %12s %12s %12s %12s\n' workload sparq eye cwm jen3
for w in socrates dt1k dt10k dt100k anc500 grid30; do
  exp=$(expected_closure "$w")

  # ---- sparq: self-assert the closure size (deep-taxonomy analogue), then time ----
  ref=$("$CLI" reason "$B/$w.n3" n3 n3 2>/dev/null | grep -oE '^[0-9]+' | head -1)
  gate sparq "$w" "$ref" "$exp"
  s=$(mon 3 "$CLI" reason "$B/$w.n3" n3 n3 /dev/null)
  printf '%-10s' "$w"; cell "$s"

  # ---- EYE (pinned reference; timing-only — its closure agreement is the pinned
  # cross-check recorded in eye-comparison.md, and its cells are unchanged here) ----
  if eye_cell_on "$w"; then
    e=$(mon "$(runs_for "$w")" "$EYE" --quiet --nope "$B/$w.n3" --pass)
  else
    e=skipped
  fi
  cell "$e"

  # ---- cwm: closure gate BEFORE timing (--data strips rules => countable) ---------
  if [ "$HAVE_CWM" = 0 ]; then
    c=absent
  elif ! cwm_cell_on "$w"; then
    c=skipped
  else
    if ! "$CWM" "$B/$w.n3" --think --data > "$B/$w.cwm.n3" 2>"$B/$w.cwm.err"; then
      echo "ERROR: cwm failed on $w:" >&2; sed -n '1,5p' "$B/$w.cwm.err" >&2; exit 1
    fi
    got=$(n3_ground_count "$B/$w.cwm.n3")
    gate cwm "$w" "$got" "$exp"
    c=$(mon "$(runs_for "$w")" "$CWM" "$B/$w.n3" --think --data)
  fi
  cell "$c"

  # ---- jen3: closure gate BEFORE timing. `-inferences` emits ONLY the derived
  # (rule-free) triples, so the gate is ground(input) + derived == expected closure —
  # sound because every workload's derived set is disjoint from its input facts.
  # Timing then serializes the FULL closure (`-conclusion`), the same pipeline shape
  # as EYE --pass / cwm --think --data. ---------------------------------------------
  if [ "$HAVE_JEN3" = 0 ]; then
    j=absent
  elif ! jen3_cell_on "$w"; then
    j=skipped
  else
    if ! "$JAVA" -jar "$JEN3_JAR" -n3 "$B/$w.n3" -inferences > "$B/$w.jen3.n3" 2>"$B/$w.jen3.err"; then
      echo "ERROR: jen3 failed on $w:" >&2; sed -n '1,5p' "$B/$w.jen3.err" >&2; exit 1
    fi
    derived=$(n3_ground_count "$B/$w.jen3.n3")
    if [ -z "$derived" ]; then
      echo "ERROR: could not count jen3's derived triples on $w (unparseable output?)" >&2
      exit 1
    fi
    gate jen3 "$w" "$(( $(ground_facts "$w") + derived ))" "$exp"
    j=$(mon "$(runs_for "$w")" "$JAVA" -jar "$JEN3_JAR" -n3 "$B/$w.n3" -conclusion)
  fi
  cell "$j"

  printf '\n'
  add_json_row "$w" "$s" "$e" "$c" "$j"
done

# [OPUS-4.8] (sq-k5qq) Strictly-additive JSON emit: write the accumulated rows only when
# SPARQ_INFER_JSON was set. The STDOUT table above is unchanged either way.
if [ -n "$JSON_OUT" ]; then
  {
    printf '{\n'
    printf '  "harness": "sparq-reason vs EYE/cwm/jen3 (eye-comparison)",\n'
    printf '  "note": "every wall-second figure is best-effort, MEASURED on the running host — ADVISORY, NON-CANONICAL (this dev box) — do not bake into committed files",\n'
    printf '  "rows": [ %s ]\n' "$JSON_ROWS"
    printf '}\n'
  } > "$JSON_OUT"
  echo "wrote results JSON to $JSON_OUT" >&2
fi
