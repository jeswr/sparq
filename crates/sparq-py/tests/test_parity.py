# [OPUS-4.8] sq-f9tu: result-PARITY + panic-SURFACE tests for the `sparq`
# Python bindings (pyo3 + maturin).
#
# Two distinct concerns:
#
# 1. RESULT PARITY. The pre-existing tests (test_basic.py / test_text.py) are
#    smoke tests; they exercise the API but assert little about cross-boundary
#    fidelity. Here we (a) pin every RDF Term KIND round-tripping across the FFI
#    boundary against hand-computed expecteds derived from SPARQL 1.1 semantics
#    (typed / language-tagged / directional literals, blank nodes, IRIs, and
#    RDF-1.2 triple terms), and (b) cross-check the two INDEPENDENT engine result
#    paths against each other: `Graph.query` materialises per-cell `Term`
#    objects, while `Graph.query_json` serialises straight from the dictionary at
#    the id level (json.rs). Agreement between them is a real parity signal — a
#    binding-layer term-materialisation bug would show up as a divergence.
#
# 2. PANIC SURFACE. The wheel is built with the `python-release` profile
#    (`panic = "unwind"`, see the workspace Cargo.toml) precisely so that a Rust
#    panic inside the `sparq` module surfaces as `pyo3_runtime.PanicException`
#    rather than `panic = "abort"` aborting the whole interpreter. There was no
#    test exercising that path. `Graph._panic_for_test()` (a private,
#    intentionally-undocumented hook) drives a deterministic Rust panic so we can
#    assert it reaches Python as a catchable exception.
#
#    EMPIRICAL NOTE: the engine itself proved robust — no organic SPARQL input
#    (division by zero, malformed regex, RDF-star, unsupported graph patterns,
#    out-of-range SUBSTR, arithmetic overflow, ...) reaches a Rust panic; every
#    such case returns a clean `ValueError`. So a dedicated panic hook is the
#    only reliable way to test the unwind boundary; see the source comment on
#    `_panic_for_test`.
#
# Run like test_basic.py:
#     python3 -m venv .venv-py && . .venv-py/bin/activate
#     pip install maturin pytest
#     (cd crates/sparq-py && maturin develop)
#     pytest crates/sparq-py/tests

import json

import pytest

import sparq

XSD = "http://www.w3.org/2001/XMLSchema#"
RDF = "http://www.w3.org/1999/02/22-rdf-syntax-ns#"


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _term_dict(t):
    """A `Term` as a plain dict — what we compare against hand-computed truth."""
    return {
        "kind": t.kind,
        "value": t.value,
        "language": t.language,
        "datatype": t.datatype,
        # [OPUS-4.8] sq-bj7o: RDF 1.2 base direction (its:dir) — None for all but
        # directional language strings.
        "direction": t.direction,
    }


def _rows_via_json(graph, sparql):
    """The SAME SELECT through the id-level JSON writer, normalised to the same
    shape `Graph.query` produces (a list of {var: term-dict}). This is the
    independent oracle path: json.rs serialises from the dictionary directly,
    never building a Python `Term`."""
    doc = json.loads(graph.query_json(sparql))
    out = []
    for b in doc["results"]["bindings"]:
        row = {}
        for var, cell in b.items():
            kind = cell["type"]
            if kind == "literal":
                # SPARQL JSON: language-tagged literals carry "xml:lang" and no
                # explicit datatype (it is implicitly rdf:langString); plain ones
                # carry "datatype" only when it is not xsd:string. RDF 1.2
                # directional language strings additionally carry "its:dir" (the
                # base direction) and are implicitly rdf:dirLangString.
                lang = cell.get("xml:lang")
                dt = cell.get("datatype")
                direction = cell.get("its:dir")  # [OPUS-4.8] sq-bj7o
                if lang is not None:
                    dt = RDF + ("dirLangString" if direction is not None else "langString")
                elif dt is None:
                    dt = XSD + "string"
                row[var] = {
                    "kind": "literal",
                    "value": cell["value"],
                    "language": lang,
                    "datatype": dt,
                    "direction": direction,
                }
            elif kind == "triple":
                # The JSON writer nests the quoted triple; `Term` renders it as an
                # N-Triples string. Compare only kind here (value rendering differs
                # by design) — the dedicated triple-term test covers the value.
                row[var] = {"kind": "triple"}
            else:  # uri / bnode
                row[var] = {"kind": kind, "value": cell["value"], "language": None, "datatype": None, "direction": None}
        out.append(row)
    return out


def _rows_via_query(graph, sparql):
    res = graph.query(sparql)
    return [{var: _term_dict(t) for var, t in row.items()} for row in res.rows]


# ===========================================================================
# (a) TERM-KIND round-trip parity — hand-computed expecteds
# ===========================================================================

TERMS_DATA = f"""
@prefix ex:  <http://ex/> .
@prefix xsd: <{XSD}> .
ex:s ex:iri      ex:object ;
     ex:plain    "plain text" ;
     ex:typedStr "typed"^^xsd:string ;
     ex:int      42 ;
     ex:dec      3.14 ;
     ex:dbl      1.0e3 ;
     ex:bool     true ;
     ex:date     "2020-01-01"^^xsd:date ;
     ex:custom   "abc"^^ex:myType ;
     ex:lang     "bonjour"@fr ;
     ex:dir      "hello"@en--ltr ;
     ex:bnode    [ ex:nested ex:x ] .
"""


def _object_of(graph, pred):
    res = graph.query(f"SELECT ?o WHERE {{ <http://ex/s> <http://ex/{pred}> ?o }}")
    assert len(res) == 1, f"expected exactly one object for ex:{pred}"
    return res.rows[0]["o"]


@pytest.mark.parametrize(
    "pred,expected",
    [
        ("iri", {"kind": "uri", "value": "http://ex/object", "language": None, "datatype": None, "direction": None}),
        ("plain", {"kind": "literal", "value": "plain text", "language": None, "datatype": XSD + "string", "direction": None}),
        # An explicit ^^xsd:string is indistinguishable from a plain literal (RDF 1.1).
        ("typedStr", {"kind": "literal", "value": "typed", "language": None, "datatype": XSD + "string", "direction": None}),
        ("int", {"kind": "literal", "value": "42", "language": None, "datatype": XSD + "integer", "direction": None}),
        ("dec", {"kind": "literal", "value": "3.14", "language": None, "datatype": XSD + "decimal", "direction": None}),
        ("dbl", {"kind": "literal", "value": "1.0e3", "language": None, "datatype": XSD + "double", "direction": None}),
        ("bool", {"kind": "literal", "value": "true", "language": None, "datatype": XSD + "boolean", "direction": None}),
        ("date", {"kind": "literal", "value": "2020-01-01", "language": None, "datatype": XSD + "date", "direction": None}),
        ("custom", {"kind": "literal", "value": "abc", "language": None, "datatype": "http://ex/myType", "direction": None}),
        # Language tag is preserved and lower-cased per the parser; datatype is rdf:langString.
        ("lang", {"kind": "literal", "value": "bonjour", "language": "fr", "datatype": RDF + "langString", "direction": None}),
        # [OPUS-4.8] sq-bj7o: directional language string keeps its base direction.
        ("dir", {"kind": "literal", "value": "hello", "language": "en", "datatype": RDF + "dirLangString", "direction": "ltr"}),
    ],
)
def test_term_kind_roundtrip(pred, expected):
    """Each RDF term kind round-trips across the FFI boundary with the exact
    kind / value / language / datatype dictated by SPARQL 1.1 + RDF 1.1."""
    g = sparq.Graph.load(TERMS_DATA)
    assert _term_dict(_object_of(g, pred)) == expected


def test_term_kind_blank_node_roundtrip():
    """A blank node surfaces as kind 'bnode' with a non-empty label and no
    language / datatype. (The exact label is engine-assigned, so we only pin its
    shape, then prove the label is internally consistent by joining on it.)"""
    g = sparq.Graph.load(TERMS_DATA)
    o = _object_of(g, "bnode")
    assert o.kind == "bnode"
    assert o.value and o.language is None and o.datatype is None
    # The bnode is the subject of its own nested triple — same engine, same label.
    res = g.query(
        "SELECT ?label WHERE { <http://ex/s> <http://ex/bnode> ?b . "
        "?b <http://ex/nested> ?label }"
    )
    assert len(res) == 1 and str(res.rows[0]["label"]) == "http://ex/x"


def test_term_kind_directional_language_string():
    """RDF 1.2 directional language string: `"hello"@en--ltr` round-trips with
    language 'en', datatype rdf:dirLangString, AND base direction 'ltr' exposed
    on the dedicated `Term.direction` field.

    [OPUS-4.8] sq-bj7o: before the fix the Term path dropped the base direction
    (only kind/value/language/datatype surfaced); now `direction` carries the
    RDF 1.2 base direction (the SPARQL 1.2 `its:dir` value)."""
    g = sparq.Graph.load(TERMS_DATA)
    o = _object_of(g, "dir")
    assert o.kind == "literal"
    assert o.value == "hello"
    assert o.language == "en"
    assert o.datatype == RDF + "dirLangString"
    assert o.direction == "ltr"


# [OPUS-4.8] sq-bj7o REGRESSION TEST. The base DIRECTION of an RDF-1.2 directional
# language string used to be LOST by the Term path but smuggled (into `xml:lang` as
# `en--ltr`) by the JSON path — so `Graph.query` and `Graph.query_json` DIVERGED.
# The fix gives the Term a dedicated `direction` field and makes `query_json` emit
# the spec-correct SPARQL 1.2 `its:dir` (with `xml:lang` now the bare tag). This
# test asserts the two paths now AGREE on the direction for both ltr and rtl.
def test_directional_literal_direction_parity_query_vs_json():
    ltr = sparq.Graph.load('@prefix ex: <http://ex/> . ex:s ex:p "x"@en--ltr .')
    rtl = sparq.Graph.load('@prefix ex: <http://ex/> . ex:s ex:p "x"@en--rtl .')

    def term_dir(g):
        o = g.query("SELECT ?o WHERE { ?s ?p ?o }").rows[0]["o"]
        # The Term keeps the bare tag in `language` and the direction separately.
        assert o.language == "en"
        return o.direction

    def json_dir(g):
        cell = json.loads(g.query_json("SELECT ?o WHERE { ?s ?p ?o }"))["results"]["bindings"][0]["o"]
        # SPARQL 1.2 JSON: `xml:lang` is the bare tag, `its:dir` the direction.
        assert cell.get("xml:lang") == "en"
        return cell.get("its:dir")

    # Both paths distinguish ltr from rtl ...
    assert term_dir(ltr) != term_dir(rtl)
    assert json_dir(ltr) != json_dir(rtl)
    # ... and the two paths AGREE term-by-term (the parity claim the bead is about).
    assert term_dir(ltr) == json_dir(ltr) == "ltr"
    assert term_dir(rtl) == json_dir(rtl) == "rtl"


# ===========================================================================
# (b) PARITY: Graph.query (Term path) vs Graph.query_json (id-level JSON path)
# ===========================================================================

PARITY_DATA = f"""
@prefix ex:  <http://ex/> .
@prefix xsd: <{XSD}> .
ex:alice ex:knows ex:bob ; ex:age 30 ; ex:name "Alice" .
ex:bob   ex:knows ex:carol ; ex:age 25 ; ex:name "Bob"@en .
ex:carol ex:age 35 ; ex:height "1.7"^^xsd:decimal .
"""

PARITY_QUERIES = [
    # All term kinds in the object position, ordered for determinism.
    "PREFIX ex: <http://ex/> SELECT ?s ?o WHERE { ?s ex:age ?o } ORDER BY ?o",
    "PREFIX ex: <http://ex/> SELECT ?s ?n WHERE { ?s ex:name ?n } ORDER BY ?s",
    "PREFIX ex: <http://ex/> SELECT ?s ?p ?o WHERE { ?s ?p ?o } ORDER BY ?s ?p ?o",
    # Projection with an unbound variable (OPTIONAL) — both paths must agree on
    # WHICH cells are absent.
    "PREFIX ex: <http://ex/> SELECT ?s ?k WHERE { ?s ex:age ?a OPTIONAL { ?s ex:knows ?k } } ORDER BY ?s",
    # A computed (BIND) decimal value.
    "PREFIX ex: <http://ex/> SELECT ?h WHERE { ?s ex:height ?h }",
]


@pytest.mark.parametrize("q", PARITY_QUERIES)
def test_query_matches_query_json(q):
    """The Term-materialising path (`query`) and the id-level JSON writer
    (`query_json`) are independent serialisations of the same solution
    sequence. They must produce identical rows (kind / value / language /
    datatype, and the same set of bound variables per row)."""
    g = sparq.Graph.load(PARITY_DATA)
    via_query = _rows_via_query(g, q)
    via_json = _rows_via_json(g, q)
    # `query` rows carry the full term dict; the JSON oracle reconstructs the
    # same shape. Triple terms are compared by kind only (see _rows_via_json), so
    # drop value/lang/datatype on query-side triple cells to match.
    norm_query = []
    for row in via_query:
        norm = {}
        for var, td in row.items():
            norm[var] = {"kind": "triple"} if td["kind"] == "triple" else td
        norm_query.append(norm)
    assert norm_query == via_json


def test_query_json_head_vars_match_query_vars():
    """`query_json`'s head.vars equals `query`'s .vars (projection order parity)."""
    g = sparq.Graph.load(PARITY_DATA)
    q = "PREFIX ex: <http://ex/> SELECT ?s ?age WHERE { ?s ex:age ?age } ORDER BY ?age"
    res = g.query(q)
    doc = json.loads(g.query_json(q))
    assert doc["head"]["vars"] == res.vars == ["s", "age"]


# ===========================================================================
# (b') PARITY for the OTHER query forms: ASK / CONSTRUCT / DESCRIBE / UPDATE
# ===========================================================================


def test_ask_parity_with_select_count():
    """ASK must agree with 'does the equivalent SELECT return any row'."""
    g = sparq.Graph.load(PARITY_DATA)
    cases = [
        ("PREFIX ex: <http://ex/> ASK { ex:alice ex:knows ex:bob }", "PREFIX ex: <http://ex/> SELECT * WHERE { ex:alice ex:knows ex:bob }"),
        ("PREFIX ex: <http://ex/> ASK { ex:bob ex:knows ex:alice }", "PREFIX ex: <http://ex/> SELECT * WHERE { ex:bob ex:knows ex:alice }"),
        ("PREFIX ex: <http://ex/> ASK { ?s ex:age 99 }", "PREFIX ex: <http://ex/> SELECT * WHERE { ?s ex:age 99 }"),
    ]
    for ask_q, select_q in cases:
        assert g.ask(ask_q) == (len(g.query(select_q)) > 0)


def test_construct_parity_hand_computed():
    """CONSTRUCT result equals the exact hand-computed triple set (deduplicated,
    SPARQL 1.1 §16.2): one `a ex:Person` per subject that has an age."""
    g = sparq.Graph.load(PARITY_DATA)
    triples = g.construct(
        "PREFIX ex: <http://ex/> CONSTRUCT { ?s a ex:Person } WHERE { ?s ex:age ?a }"
    )
    got = {(s.value, p.value, o.value) for s, p, o in triples}
    expected = {
        (subj, RDF + "type", "http://ex/Person")
        for subj in ("http://ex/alice", "http://ex/bob", "http://ex/carol")
    }
    assert got == expected
    # Every constructed cell is a URI here — pin the kinds too.
    assert all(s.kind == "uri" and p.kind == "uri" and o.kind == "uri" for s, p, o in triples)


def test_construct_literal_object_parity():
    """A CONSTRUCT that carries a language-tagged literal through preserves the
    full literal term (value + language + datatype)."""
    g = sparq.Graph.load(PARITY_DATA)
    ((_, _, o),) = g.construct(
        "PREFIX ex: <http://ex/> CONSTRUCT { ex:bob ex:label ?n } WHERE { ex:bob ex:name ?n }"
    )
    assert _term_dict(o) == {
        "kind": "literal",
        "value": "Bob",
        "language": "en",
        "datatype": RDF + "langString",
        "direction": None,
    }


def test_describe_parity_hand_computed():
    """DESCRIBE <alice> equals alice's exact asserted (subject) triple set."""
    g = sparq.Graph.load(PARITY_DATA)
    triples = g.describe("DESCRIBE <http://ex/alice>")
    got = {(s.value, p.value, o.value) for s, p, o in triples}
    expected = {
        ("http://ex/alice", "http://ex/knows", "http://ex/bob"),
        ("http://ex/alice", "http://ex/age", "30"),
        ("http://ex/alice", "http://ex/name", "Alice"),
    }
    assert got == expected


def test_update_parity_round_trip_count_and_content():
    """An INSERT then a matching DELETE returns the graph to its exact prior
    state — both the triple COUNT and the queryable content."""
    g = sparq.Graph.load(PARITY_DATA)
    before = len(g)
    before_json = g.query_json("SELECT ?s ?p ?o WHERE { ?s ?p ?o } ORDER BY ?s ?p ?o")
    g.update("INSERT DATA { <http://ex/dave> <http://ex/age> 40 }")
    assert len(g) == before + 1
    assert g.ask("PREFIX ex: <http://ex/> ASK { ex:dave ex:age 40 }")
    g.update("DELETE DATA { <http://ex/dave> <http://ex/age> 40 }")
    assert len(g) == before
    after_json = g.query_json("SELECT ?s ?p ?o WHERE { ?s ?p ?o } ORDER BY ?s ?p ?o")
    assert after_json == before_json  # exact content parity, not just count


# ===========================================================================
# (a'') RDF-1.2 triple term (quoted triple) round-trip
# ===========================================================================

# Turtle reification: << ex:a ex:b ex:c >> ex:certainty "0.9" . The engine models
# this with the RDF-1.2 reification vocabulary (a fresh bnode rdf:reifies the
# quoted triple), so the quoted triple surfaces as a Term of kind "triple".
STAR_DATA = '@prefix ex: <http://ex/> . << ex:a ex:b ex:c >> ex:certainty "0.9" .'


def test_triple_term_roundtrips_as_kind_triple():
    g = sparq.Graph.load(STAR_DATA)
    res = g.query(
        "SELECT ?o WHERE { ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?o }"
    )
    assert len(res) == 1
    o = res.rows[0]["o"]
    assert o.kind == "triple"
    assert o.language is None and o.datatype is None
    # Value is the N-Triples rendering of the quoted triple.
    assert o.value == "<http://ex/a> <http://ex/b> <http://ex/c>"
    # repr passes the quoted-triple rendering straight through (no extra quoting).
    assert repr(o) == "Term(<http://ex/a> <http://ex/b> <http://ex/c>)"


def test_triple_term_query_json_nests_the_triple():
    """The JSON path nests the quoted triple as a structured object (subject /
    predicate / object), confirming the id-level writer handles triple terms."""
    g = sparq.Graph.load(STAR_DATA)
    doc = json.loads(
        g.query_json(
            "SELECT ?o WHERE { ?s <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?o }"
        )
    )
    (binding,) = doc["results"]["bindings"]
    cell = binding["o"]
    assert cell["type"] == "triple"
    assert cell["value"]["subject"] == {"type": "uri", "value": "http://ex/a"}
    assert cell["value"]["predicate"] == {"type": "uri", "value": "http://ex/b"}
    assert cell["value"]["object"] == {"type": "uri", "value": "http://ex/c"}


# ===========================================================================
# (2) PANIC SURFACE: a Rust panic must reach Python, not abort the interpreter
# ===========================================================================


def _is_panic_exception(exc):
    """pyo3 raises panics as `pyo3_runtime.PanicException`, a direct subclass of
    BaseException (NOT Exception), and the `pyo3_runtime` module is not
    importable — so we identify it by its fully-qualified type name."""
    t = type(exc)
    return f"{t.__module__}.{t.__qualname__}" == "pyo3_runtime.PanicException"


def test_rust_panic_surfaces_as_panic_exception():
    """A deliberate Rust panic (via the private `_panic_for_test` hook) must be
    caught by pyo3's unwind boundary and re-raised as `pyo3_runtime.
    PanicException` — proving the `python-release` profile's `panic = "unwind"`
    is what actually governs binding builds. If this instead aborted the
    interpreter, the whole pytest process would die (and CI would show no result
    for this test), so reaching the assertion at all is itself part of the
    guarantee."""
    g = sparq.Graph.load("@prefix ex: <http://ex/> . ex:a ex:b ex:c .")
    # NOTE: catch BaseException — PanicException does NOT derive from Exception,
    # so `pytest.raises(Exception)` would let it through.
    with pytest.raises(BaseException) as ei:
        g._panic_for_test()
    assert _is_panic_exception(ei.value), (
        f"expected pyo3_runtime.PanicException, got {type(ei.value).__module__}."
        f"{type(ei.value).__qualname__}"
    )
    # The panic message is carried through to Python.
    assert "panic-surface test" in str(ei.value)


def test_interpreter_survives_a_caught_panic():
    """After catching a panic, the interpreter (and the same Graph) keeps working
    — a panic is a recoverable Python exception here, not a process abort."""
    g = sparq.Graph.load("@prefix ex: <http://ex/> . ex:a ex:b ex:c .")
    try:
        g._panic_for_test()
    except BaseException:
        pass
    # The Graph is still usable, and the GIL/state is intact.
    assert len(g) == 1
    assert g.ask("PREFIX ex: <http://ex/> ASK { ex:a ex:b ex:c }")
