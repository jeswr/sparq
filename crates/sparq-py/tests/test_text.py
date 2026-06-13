"""Full-text search tests for the `sparq` Python package (the TextIndex lifecycle
tracked as a follow-up in beads — `bd list -l area:sparq-py`): `Graph.text_search`,
`Graph.query_text` (the `text:` magic predicates), the lazy build + cache, and
its invalidation on every mutating swap (update / reason_n3_with).

Run like test_basic.py:

    python3 -m venv .venv-py && . .venv-py/bin/activate
    pip install maturin pytest
    (cd crates/sparq-py && maturin develop)
    pytest crates/sparq-py/tests
"""

import pytest

import sparq

TEXT_DATA = """
@prefix ex: <http://ex/> .
ex:a ex:comment "The quick brown fox jumps over the lazy dog" .
ex:b ex:comment "A quick red fox" .
ex:c ex:comment "Sleepy lazy dog"@en .
ex:d ex:comment "Nothing relevant here" .
ex:e ex:count 42 .
"""

PREFIXES = "PREFIX text: <http://sparq.dev/text#> PREFIX ex: <http://ex/> "


def tg():
    return sparq.Graph.load(TEXT_DATA)


# --- text_search (direct ranked hits) ----------------------------------------


def test_text_search_and_semantics_ranked():
    hits = tg().text_search("quick fox")
    assert len(hits) == 2  # AND: only a and b contain both tokens
    for term, score in hits:
        assert term.kind == "literal" and score > 0.0
    scores = [s for _, s in hits]
    assert scores == sorted(scores, reverse=True)  # best-first
    # BM25 length normalisation: the shorter matching literal ranks first
    assert hits[0][0].value == "A quick red fox"
    # AND with a token present nowhere matches nothing
    assert tg().text_search("quick zebra") == []


def test_text_search_any_prefix_and_limit():
    graph = tg()
    assert len(graph.text_search("quick dog")) == 1  # AND: only ex:a has both
    assert len(graph.text_search("quick dog", any=True)) == 3  # OR: a, b, c
    assert len(graph.text_search("qui*")) == 2  # prefix token: quick -> a, b
    assert len(graph.text_search("quick dog", any=True, limit=1)) == 1
    assert graph.text_search("") == []


def test_text_search_language_tagged_and_typed_literals():
    graph = tg()
    hits = graph.text_search("sleepy")
    assert len(hits) == 1
    assert hits[0][0].language == "en"  # language-tagged literals are indexed
    assert graph.text_search("42") == []  # typed literals are not text


def test_text_search_empty_graph():
    assert sparq.Graph.load("").text_search("anything") == []


# --- query_text (text: magic predicates) -------------------------------------


def test_query_text_matches_joins_to_subjects():
    res = tg().query_text(
        PREFIXES
        + 'SELECT ?s WHERE { ?s ex:comment ?lit . ?lit text:matches "quick fox" } ORDER BY ?s'
    )
    assert [str(r["s"]) for r in res] == ["http://ex/a", "http://ex/b"]


def test_query_text_matches_any():
    res = tg().query_text(
        PREFIXES
        + 'SELECT ?s WHERE { ?s ex:comment ?lit . ?lit text:matchesAny "quick dog" } ORDER BY ?s'
    )
    assert [str(r["s"]) for r in res] == ["http://ex/a", "http://ex/b", "http://ex/c"]


def test_query_text_score_ordering():
    res = tg().query_text(
        PREFIXES
        + "SELECT ?s ?sc WHERE { ?s ex:comment ?lit . ?lit text:matches \"fox\" . "
        + "?lit text:score ?sc } ORDER BY DESC(?sc)"
    )
    assert len(res) == 2
    scores = [float(str(r["sc"])) for r in res]
    assert scores == sorted(scores, reverse=True) and scores[0] > scores[1]
    assert res[0]["sc"].datatype == "http://www.w3.org/2001/XMLSchema#double"
    assert str(res[0]["s"]) == "http://ex/b"  # the shorter literal scores higher


def test_query_text_without_magic_passes_through():
    res = tg().query_text(PREFIXES + "SELECT ?s WHERE { ?s ex:count 42 }")
    assert [str(r["s"]) for r in res] == ["http://ex/e"]


def test_query_text_errors():
    graph = tg()
    with pytest.raises(ValueError, match="unknown magic predicate"):
        graph.query_text(PREFIXES + 'SELECT ?l WHERE { ?l text:bogus "x" }')
    with pytest.raises(ValueError, match="constant query-string literal"):
        graph.query_text(PREFIXES + "SELECT ?l WHERE { ?l text:matches ?q }")
    with pytest.raises(ValueError):  # malformed SPARQL still a ValueError
        graph.query_text("SELECT WHERE {")


# --- lifecycle: lazy build, invalidation on mutating swaps, explicit drop ----


def test_text_index_invalidated_on_update():
    graph = tg()
    assert graph.text_search("astronaut") == []  # index built + cached here
    graph.update('INSERT DATA { <http://ex/n> <http://ex/comment> "an astronaut appears" }')
    hits = graph.text_search("astronaut")  # cache invalidated, rebuilt lazily
    assert len(hits) == 1 and hits[0][0].value == "an astronaut appears"
    res = graph.query_text(
        PREFIXES + 'SELECT ?s WHERE { ?s ex:comment ?lit . ?lit text:matches "astronaut" }'
    )
    assert [str(r["s"]) for r in res] == ["http://ex/n"]
    # deletion: the rebuilt index drops the literal once no triple carries it
    graph.update('DELETE DATA { <http://ex/n> <http://ex/comment> "an astronaut appears" }')
    assert not graph.query_text(
        PREFIXES + 'SELECT ?s WHERE { ?s ex:comment ?lit . ?lit text:matches "astronaut" }'
    ).rows


def test_text_index_invalidated_on_reason_n3_with():
    # reason_n3_with rebuilds the graph around a FRESH dictionary (term ids all
    # change) — the strongest staleness test for the cached index.
    graph = tg()
    assert graph.text_search("brave") == []  # index built + cached here
    graph.reason_n3_with('@prefix ex: <http://ex/> . ex:n ex:comment "Dave the brave" .')
    hits = graph.text_search("brave")
    assert len(hits) == 1 and hits[0][0].value == "Dave the brave"
    # pre-existing literals still resolve correctly under the re-encoded ids
    assert len(graph.text_search("quick fox")) == 2


def test_build_and_drop_text_index():
    graph = tg()
    assert graph.build_text_index() == 4  # the four string literals; 42 is not text
    hits = graph.text_search("fox")
    assert len(hits) == 2
    graph.drop_text_index()  # frees the cache ...
    assert graph.text_search("fox") == hits  # ... and the lazy rebuild is identical
