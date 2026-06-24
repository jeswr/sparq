"""Tests for the opt-in Arrow export: ``Graph.query_arrow(sparql) -> pyarrow.Table``.

[OPUS-4.8] sq-lt1ml (gh-910). The Python half of the Arrow columnar export. These tests
only run when BOTH:

* the wheel was built with ``--features arrow`` (so ``Graph.query_arrow`` exists), and
* ``pyarrow`` is importable (the consumer side of the Arrow C Data Interface).

Either missing -> the whole module is skipped, so the lean (no-arrow) wheel's pytest run
stays green without these. The build that exercises the arrow path installs pyarrow and
builds with the feature on (see ``.github/workflows/python.yml`` arrow job).

What is asserted is the load-bearing invariant of the binding: a real ``pyarrow.Table``
with one ``struct<kind,value,datatype,language,direction>`` column per SELECT variable,
the right rows, and an UNBOUND cell read back as a null struct slot (distinct from a bound
empty-string literal) — i.e. exactly the ``sparq-arrow`` term mapping, faithfully bridged.
"""

import pytest

import sparq

pa = pytest.importorskip("pyarrow", reason="Arrow export needs pyarrow on the consumer side")

if not hasattr(sparq.Graph, "query_arrow"):
    pytest.skip(
        "wheel built without the `arrow` feature (no Graph.query_arrow)",
        allow_module_level=True,
    )

TURTLE = """
@prefix ex: <http://ex/> .
ex:alice ex:age 30 ; ex:name "Alice" .
ex:bob   ex:age 25 .
ex:carol ex:age 41 ; ex:name "Carol"@en .
"""

TERM_FIELDS = ["kind", "value", "datatype", "language", "direction"]
XSD_INTEGER = "http://www.w3.org/2001/XMLSchema#integer"
XSD_STRING = "http://www.w3.org/2001/XMLSchema#string"


def _graph():
    return sparq.Graph.load(TURTLE)


def test_returns_pyarrow_table_with_struct_columns():
    """A known SELECT -> a pyarrow.Table whose columns are the term struct."""
    table = _graph().query_arrow(
        "PREFIX ex: <http://ex/> SELECT ?s ?age WHERE { ?s ex:age ?age } ORDER BY ?age"
    )
    assert isinstance(table, pa.Table)
    assert table.column_names == ["s", "age"]
    assert table.num_rows == 3
    for name in table.column_names:
        field = table.schema.field(name)
        assert pa.types.is_struct(field.type), f"{name} should be a struct column"
        assert [f.name for f in field.type] == TERM_FIELDS
        for child in field.type:
            assert pa.types.is_string(child.type)


def test_column_order_matches_projection():
    """Columns follow the SELECT projection order, not graph/lexical order."""
    table = _graph().query_arrow(
        "PREFIX ex: <http://ex/> SELECT ?age ?s WHERE { ?s ex:age ?age } LIMIT 1"
    )
    assert table.column_names == ["age", "s"]


def test_term_decomposition_uri_and_typed_literal():
    """A bound IRI / typed literal decomposes into the right struct fields."""
    table = _graph().query_arrow(
        "PREFIX ex: <http://ex/> SELECT ?s ?age WHERE { ?s ex:age ?age } ORDER BY ?age"
    )
    rows = table.to_pylist()
    youngest = rows[0]
    assert youngest["s"] == {
        "kind": "uri",
        "value": "http://ex/bob",
        "datatype": None,
        "language": None,
        "direction": None,
    }
    assert youngest["age"] == {
        "kind": "literal",
        "value": "25",
        "datatype": XSD_INTEGER,
        "language": None,
        "direction": None,
    }


def test_unbound_is_null_struct_slot():
    """An OPTIONAL that does not match is a NULL struct slot, not an empty literal."""
    table = _graph().query_arrow(
        "PREFIX ex: <http://ex/> "
        "SELECT ?s ?name WHERE { ?s ex:age ?age OPTIONAL { ?s ex:name ?name } } ORDER BY ?age"
    )
    by_subject = {row["s"]["value"]: row["name"] for row in table.to_pylist()}
    # bob has no ex:name -> unbound -> the whole struct slot is null (None in pyarrow).
    assert by_subject["http://ex/bob"] is None
    # alice has a plain string name -> a bound literal, distinct from the null above.
    assert by_subject["http://ex/alice"] == {
        "kind": "literal",
        "value": "Alice",
        "datatype": XSD_STRING,
        "language": None,
        "direction": None,
    }


def test_language_tagged_literal_preserved():
    """A language-tagged literal keeps its `language` field (no datatype)."""
    table = _graph().query_arrow(
        "PREFIX ex: <http://ex/> SELECT ?name WHERE { ex:carol ex:name ?name }"
    )
    row = table.to_pylist()[0]
    assert row["name"]["kind"] == "literal"
    assert row["name"]["value"] == "Carol"
    assert row["name"]["language"] == "en"
    assert row["name"]["datatype"] is None


def test_empty_result_keeps_schema():
    """A SELECT with no solutions still yields the typed, zero-row table."""
    table = _graph().query_arrow(
        "PREFIX ex: <http://ex/> SELECT ?s WHERE { ?s ex:nonexistent ?o }"
    )
    assert table.num_rows == 0
    assert table.column_names == ["s"]
    assert pa.types.is_struct(table.schema.field("s").type)


def test_matches_query_json_rows():
    """The Arrow rows agree with `query_json` (the native projection) value-for-value."""
    import json

    sparql = "PREFIX ex: <http://ex/> SELECT ?s ?age WHERE { ?s ex:age ?age } ORDER BY ?age"
    g = _graph()
    arrow_rows = g.query_arrow(sparql).to_pylist()
    json_doc = json.loads(g.query_json(sparql))
    json_rows = json_doc["results"]["bindings"]
    assert len(arrow_rows) == len(json_rows)
    for arrow_row, json_row in zip(arrow_rows, json_rows):
        # `?s` is a uri, `?age` a typed literal: compare value + kind across the two paths.
        assert arrow_row["s"]["value"] == json_row["s"]["value"]
        assert arrow_row["s"]["kind"] == json_row["s"]["type"]
        assert arrow_row["age"]["value"] == json_row["age"]["value"]
        assert arrow_row["age"]["datatype"] == json_row["age"]["datatype"]
