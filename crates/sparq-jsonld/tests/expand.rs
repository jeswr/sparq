//! [OPUS-4.8] (sq-oy1f.25) End-to-end tests for the JSON-LD 1.1 **Expansion Algorithm**,
//! exercised through the crate's public [`sparq_jsonld::expand`] entry. Fixtures are drawn
//! from the worked examples of the W3C JSON-LD 1.1 API + syntax specs
//! (<https://www.w3.org/TR/json-ld11-api/#expansion-algorithms>). No network: expansion runs
//! against the deny-by-default `NoopLoader`.
//!
//! Comparison follows the suite's normative rule — deep JSON equality where **array order is
//! significant only inside `@list`**; every other array (property values, `@type`, `@graph`,
//! `@set`) is compared order-insensitively via the [`canon`] canonicaliser.

use sparq_jsonld::{expand, Json, JsonLdErrorCode, JsonLdOptions, NoopLoader};

/// Parses a JSON fixture.
fn json(s: &str) -> Json {
    Json::parse(s).expect("valid JSON fixture")
}

/// Expands `input` with the default options (no base).
fn exp(input: &str) -> Json {
    expand(&json(input), &JsonLdOptions::default(), &NoopLoader).expect("expansion should succeed")
}

/// Expands `input` with an explicit base IRI.
fn exp_base(input: &str, base: &str) -> Json {
    let mut opts = JsonLdOptions::default();
    opts.base = Some(base.to_string());
    expand(&json(input), &opts, &NoopLoader).expect("expansion should succeed")
}

/// Canonicalises a `Json` for the normative comparison: object keys are sorted, and every
/// array is sorted by its canonical serialisation **except** the array under an `@list` key,
/// whose order is preserved.
fn canon(j: &Json) -> Json {
    match j {
        Json::Obj(members) => {
            let mut out: Vec<(String, Json)> = members
                .iter()
                .map(|(k, v)| {
                    let cv = if k == "@list" {
                        canon_ordered(v)
                    } else {
                        canon(v)
                    };
                    (k.clone(), cv)
                })
                .collect();
            out.sort_by(|a, b| a.0.cmp(&b.0));
            Json::Obj(out)
        }
        Json::Arr(items) => {
            let mut c: Vec<Json> = items.iter().map(canon).collect();
            c.sort_by_key(serialize);
            Json::Arr(c)
        }
        other => other.clone(),
    }
}

/// Canonicalises the elements of an ordered array (an `@list` value) **without** reordering.
fn canon_ordered(j: &Json) -> Json {
    match j {
        Json::Arr(items) => Json::Arr(items.iter().map(canon).collect()),
        other => canon(other),
    }
}

/// Serialises a `Json` to its canonical minified string.
fn serialize(j: &Json) -> String {
    let mut s = String::new();
    j.write(&mut s);
    s
}

/// Asserts the expansion of `input` equals `expected` under the normative comparison.
#[track_caller]
fn assert_expands(input: &str, expected: &str) {
    let got = exp(input);
    let want = json(expected);
    assert_eq!(
        canon(&got),
        canon(&want),
        "\n  input:    {}\n  got:      {}\n  expected: {}",
        input,
        serialize(&got),
        serialize(&want)
    );
}

// ---------------------------------------------------------------------------------------
// Core expansion: term IRIs, @id / @type coercion, value objects.
// ---------------------------------------------------------------------------------------

#[test]
fn simple_terms_and_id_coercion() {
    // JSON-LD 1.1 API — the canonical "homepage @type: @id" example.
    assert_expands(
        r#"{
            "@context": {
                "name": "http://schema.org/name",
                "homepage": {"@id": "http://schema.org/url", "@type": "@id"}
            },
            "@id": "http://me.example.com/",
            "name": "Alice",
            "homepage": "http://alice.example.com/"
        }"#,
        r#"[{
            "@id": "http://me.example.com/",
            "http://schema.org/name": [{"@value": "Alice"}],
            "http://schema.org/url": [{"@id": "http://alice.example.com/"}]
        }]"#,
    );
}

#[test]
fn typed_value_object() {
    assert_expands(
        r#"{
            "@context": {"age": {"@id": "http://ex/age", "@type": "http://www.w3.org/2001/XMLSchema#integer"}},
            "age": "42"
        }"#,
        r#"[{"http://ex/age": [{"@value": "42", "@type": "http://www.w3.org/2001/XMLSchema#integer"}]}]"#,
    );
}

#[test]
fn default_language_applied_to_strings() {
    assert_expands(
        r#"{"@context": {"@language": "ja", "term": "http://ex/term"}, "term": "花"}"#,
        r#"[{"http://ex/term": [{"@value": "花", "@language": "ja"}]}]"#,
    );
}

#[test]
fn number_and_boolean_values_have_no_language() {
    // A default @language must NOT be attached to non-string values.
    assert_expands(
        r#"{"@context": {"@language": "en", "n": "http://ex/n", "b": "http://ex/b"}, "n": 42, "b": true}"#,
        r#"[{"http://ex/n": [{"@value": 42}], "http://ex/b": [{"@value": true}]}]"#,
    );
}

// ---------------------------------------------------------------------------------------
// Keyword aliases.
// ---------------------------------------------------------------------------------------

#[test]
fn keyword_aliases_for_id_and_type() {
    assert_expands(
        r#"{
            "@context": {"id": "@id", "type": "@type", "Person": "http://schema.org/Person"},
            "id": "http://ex/me",
            "type": "Person"
        }"#,
        r#"[{"@id": "http://ex/me", "@type": ["http://schema.org/Person"]}]"#,
    );
}

// ---------------------------------------------------------------------------------------
// @list / @set.
// ---------------------------------------------------------------------------------------

#[test]
fn list_container_preserves_order() {
    assert_expands(
        r#"{"@context": {"foo": {"@id": "http://ex/foo", "@container": "@list"}}, "foo": ["a", "b", "c"]}"#,
        r#"[{"http://ex/foo": [{"@list": [{"@value": "a"}, {"@value": "b"}, {"@value": "c"}]}]}]"#,
    );
}

#[test]
fn set_container_unwraps() {
    assert_expands(
        r#"{"@context": {"foo": {"@id": "http://ex/foo", "@container": "@set"}}, "foo": ["a"]}"#,
        r#"[{"http://ex/foo": [{"@value": "a"}]}]"#,
    );
}

#[test]
fn explicit_list_object() {
    assert_expands(
        r#"{"http://ex/foo": {"@list": ["a", "b"]}}"#,
        r#"[{"http://ex/foo": [{"@list": [{"@value": "a"}, {"@value": "b"}]}]}]"#,
    );
}

// ---------------------------------------------------------------------------------------
// @graph, top-level reduction, scalar drops.
// ---------------------------------------------------------------------------------------

#[test]
fn top_level_graph_reduces() {
    assert_expands(
        r#"{"@context": {"foo": "http://ex/foo"}, "@graph": [{"@id": "http://ex/1", "foo": "bar"}]}"#,
        r#"[{"@id": "http://ex/1", "http://ex/foo": [{"@value": "bar"}]}]"#,
    );
}

#[test]
fn top_level_scalar_expands_to_empty_array() {
    assert_eq!(exp("42"), Json::Arr(vec![]));
    assert_eq!(exp("null"), Json::Arr(vec![]));
    assert_eq!(exp("\"a string\""), Json::Arr(vec![]));
}

#[test]
fn free_floating_id_only_node_is_dropped() {
    // A top-level node consisting only of @id projects to no output.
    assert_eq!(exp(r#"{"@id": "http://ex/1"}"#), Json::Arr(vec![]));
}

// ---------------------------------------------------------------------------------------
// @reverse.
// ---------------------------------------------------------------------------------------

#[test]
fn reverse_property_term() {
    assert_expands(
        r#"{
            "@context": {"parentOf": {"@reverse": "http://ex/childOf"}},
            "@id": "http://ex/parent",
            "parentOf": {"@id": "http://ex/child"}
        }"#,
        r#"[{
            "@id": "http://ex/parent",
            "@reverse": {"http://ex/childOf": [{"@id": "http://ex/child"}]}
        }]"#,
    );
}

#[test]
fn reverse_keyword() {
    assert_expands(
        r#"{
            "@id": "http://ex/a",
            "@reverse": {"http://ex/p": {"@id": "http://ex/b"}}
        }"#,
        r#"[{"@id": "http://ex/a", "@reverse": {"http://ex/p": [{"@id": "http://ex/b"}]}}]"#,
    );
}

// ---------------------------------------------------------------------------------------
// Scoped contexts.
// ---------------------------------------------------------------------------------------

#[test]
fn property_scoped_context() {
    assert_expands(
        r#"{
            "@context": {"foo": {"@id": "http://ex/foo", "@context": {"bar": "http://ex/bar"}}},
            "foo": {"bar": "baz"}
        }"#,
        r#"[{"http://ex/foo": [{"http://ex/bar": [{"@value": "baz"}]}]}]"#,
    );
}

#[test]
fn type_scoped_context() {
    assert_expands(
        r#"{
            "@context": {
                "Foo": {"@id": "http://ex/Foo", "@context": {"bar": "http://ex/bar"}},
                "type": "@type"
            },
            "type": "Foo",
            "bar": "baz"
        }"#,
        r#"[{"@type": ["http://ex/Foo"], "http://ex/bar": [{"@value": "baz"}]}]"#,
    );
}

// ---------------------------------------------------------------------------------------
// @nest.
// ---------------------------------------------------------------------------------------

#[test]
fn nest_flattens_into_parent() {
    assert_expands(
        r#"{
            "@context": {"nest": "@nest", "name": "http://ex/name", "age": "http://ex/age"},
            "@id": "http://ex/1",
            "name": "Alice",
            "nest": {"age": "30"}
        }"#,
        r#"[{
            "@id": "http://ex/1",
            "http://ex/name": [{"@value": "Alice"}],
            "http://ex/age": [{"@value": "30"}]
        }]"#,
    );
}

// ---------------------------------------------------------------------------------------
// Container maps: @index / @id / @type / @language.
// ---------------------------------------------------------------------------------------

#[test]
fn index_map() {
    assert_expands(
        r#"{"@context": {"foo": {"@id": "http://ex/foo", "@container": "@index"}}, "foo": {"a": "1", "b": "2"}}"#,
        r#"[{"http://ex/foo": [{"@value": "1", "@index": "a"}, {"@value": "2", "@index": "b"}]}]"#,
    );
}

#[test]
fn id_map() {
    assert_expands(
        r#"{
            "@context": {"foo": {"@id": "http://ex/foo", "@container": "@id"}, "name": "http://ex/name"},
            "foo": {"http://ex/1": {"name": "Alice"}}
        }"#,
        r#"[{"http://ex/foo": [{"@id": "http://ex/1", "http://ex/name": [{"@value": "Alice"}]}]}]"#,
    );
}

#[test]
fn type_map() {
    assert_expands(
        r#"{
            "@context": {"foo": {"@id": "http://ex/foo", "@container": "@type"}, "name": "http://ex/name"},
            "foo": {"http://ex/T": {"name": "Alice"}}
        }"#,
        r#"[{"http://ex/foo": [{"@type": ["http://ex/T"], "http://ex/name": [{"@value": "Alice"}]}]}]"#,
    );
}

#[test]
fn language_map() {
    assert_expands(
        r#"{
            "@context": {"label": {"@id": "http://ex/label", "@container": "@language"}},
            "label": {"en": "Hello", "de": "Hallo"}
        }"#,
        r#"[{"http://ex/label": [{"@value": "Hello", "@language": "en"}, {"@value": "Hallo", "@language": "de"}]}]"#,
    );
}

// ---------------------------------------------------------------------------------------
// Property-valued index containers (`@container: @index` + `"@index": <prop>`) and the
// expanded-index @none guards — mirrors of W3C expand suite cases pi05/pi06/pi07/pi10/m012
// (§5.1.2 steps 13.8.3.7–13.8.3.10).
// ---------------------------------------------------------------------------------------

#[test]
fn property_valued_index_becomes_property_value() {
    // W3C expand/pi06: the index of a property-valued index map re-expands (Value
    // Expansion, with the index key as active property) as a value of that property,
    // instead of `@index`.
    assert_expands(
        r#"{
            "@context": {
                "@base": "http://example.com/",
                "@vocab": "http://example.com/",
                "author": {"@type": "@id", "@container": "@index", "@index": "prop"}
            },
            "@id": "article",
            "author": {"regular": "person/1", "guest": ["person/2", "person/3"]}
        }"#,
        r#"[{
            "@id": "http://example.com/article",
            "http://example.com/author": [
                {"@id": "http://example.com/person/1", "http://example.com/prop": [{"@value": "regular"}]},
                {"@id": "http://example.com/person/2", "http://example.com/prop": [{"@value": "guest"}]},
                {"@id": "http://example.com/person/3", "http://example.com/prop": [{"@value": "guest"}]}
            ]
        }]"#,
    );
}

#[test]
fn property_valued_index_precedes_existing_values() {
    // §5.1.2 step 13.8.3.7.3: index property values are "an array consisting of re-expanded
    // index FOLLOWED BY the existing values" — the suite's pi07 expected output shows the
    // same ordering. The shared `canon` comparison is order-insensitive, so this asserts on
    // the serialised property array directly.
    let got = exp(r#"{
        "@context": {
            "@base": "http://example.com/",
            "@vocab": "http://example.com/",
            "author": {"@type": "@id", "@container": "@index", "@index": "prop"}
        },
        "@id": "article",
        "author": {"regular": {"@id": "person/1", "prop": "foo"}}
    }"#);
    let s = serialize(&got);
    assert!(
        s.contains(r#""http://example.com/prop":[{"@value":"regular"},{"@value":"foo"}]"#),
        "re-expanded index must precede the item's existing values, got: {s}",
    );
}

#[test]
fn property_valued_index_none_key_adds_no_property() {
    // W3C expand/pi10: an `@none` index adds NO property (step 13.8.3.7 requires "expanded
    // index is not @none"). The `guest` entry also pins the `@type: @vocab` re-expansion of
    // the index into a vocab-IRI node reference.
    assert_expands(
        r#"{
            "@context": {
                "@base": "http://example.com/",
                "@vocab": "http://example.com/",
                "author": {"@type": "@id", "@container": "@index", "@index": "prop"},
                "prop": {"@type": "@vocab"}
            },
            "@id": "http://example.com/article",
            "author": {"@none": {"@id": "person/1"}, "guest": [{"@id": "person/2"}]}
        }"#,
        r#"[{
            "@id": "http://example.com/article",
            "http://example.com/author": [
                {"@id": "http://example.com/person/1"},
                {"@id": "http://example.com/person/2",
                 "http://example.com/prop": [{"@id": "http://example.com/guest"}]}
            ]
        }]"#,
    );
}

#[test]
fn property_valued_index_on_value_object_errors() {
    // W3C expand/pi05: adding the index property to a value object is an
    // "invalid value object" error (§5.1.2 step 13.8.3.7.5).
    assert_error(
        r#"{
            "@context": {
                "@vocab": "http://example.com/",
                "container": {"@id": "http://example.com/container", "@container": "@index", "@index": "prop"}
            },
            "@id": "http://example.com/annotationsTest",
            "container": {"en": "The Queen"}
        }"#,
        JsonLdErrorCode::InvalidValueObject,
    );
}

#[test]
fn type_map_none_alias_adds_no_type() {
    // W3C expand/m012: the @none guards compare the EXPANDED index, so a term aliased to
    // `@none` used as a type-map key adds no `@type` (step 13.8.3.10).
    assert_expands(
        r#"{
            "@context": {"@vocab": "http://example/", "typemap": {"@container": "@type"}, "none": "@none"},
            "typemap": {"@none": {"label": "a"}, "none": {"label": "b"}}
        }"#,
        r#"[{
            "http://example/typemap": [
                {"http://example/label": [{"@value": "a"}]},
                {"http://example/label": [{"@value": "b"}]}
            ]
        }]"#,
    );
}

#[test]
fn type_map_key_precedes_existing_types() {
    // §5.1.2 step 13.8.3.10: "types … consisting of expanded index FOLLOWED BY any existing
    // values of @type" — the suite's m004 expected output shows the same ordering.
    let got = exp(r#"{
        "@context": {"@vocab": "http://example/", "typemap": {"@container": "@type"}},
        "typemap": {"_:bar": {"@type": "_:foo", "label": "x"}}
    }"#);
    let s = serialize(&got);
    assert!(
        s.contains(r#""@type":["_:bar","_:foo"]"#),
        "the type-map key must precede the item's own @type values, got: {s}",
    );
}

#[test]
fn id_coercion_keyword_shaped_value_yields_null_id() {
    // Value Expansion (§5.3.2 step 1) is literal: a keyword-shaped token under `@type: @id`
    // IRI-expands to null, so the value expands to `{"@id": null}` — retained with a JSON
    // null exactly as the W3C expand/0122 expected output retains `"@id": null` on the
    // `@id`-keyword path (its manifest notes the result "will not be valid JSON-LD").
    assert_expands(
        r#"{"@context": {"p": {"@id": "http://ex/p", "@type": "@id"}}, "p": "@kw"}"#,
        r#"[{"http://ex/p": [{"@id": null}]}]"#,
    );
}

// ---------------------------------------------------------------------------------------
// @json literals.
// ---------------------------------------------------------------------------------------

#[test]
fn json_literal_keeps_value_verbatim() {
    assert_expands(
        r#"{"@context": {"e": {"@id": "http://ex/e", "@type": "@json"}}, "e": {"foo": ["bar", 1, true]}}"#,
        r#"[{"http://ex/e": [{"@value": {"foo": ["bar", 1, true]}, "@type": "@json"}]}]"#,
    );
}

// ---------------------------------------------------------------------------------------
// Null drops + array normalisation + relative IRIs.
// ---------------------------------------------------------------------------------------

#[test]
fn null_values_are_dropped() {
    assert_expands(
        r#"{"@context": {"foo": "http://ex/foo"}, "foo": null, "http://ex/bar": ["a", null, "b"]}"#,
        r#"[{"http://ex/bar": [{"@value": "a"}, {"@value": "b"}]}]"#,
    );
}

#[test]
fn relative_iris_resolve_against_base() {
    assert_eq!(
        canon(&exp_base(
            r#"{"@id": "foo", "http://ex/p": "v"}"#,
            "http://example.org/dir/"
        )),
        canon(&json(
            r#"[{"@id": "http://example.org/dir/foo", "http://ex/p": [{"@value": "v"}]}]"#
        ))
    );
}

// ---------------------------------------------------------------------------------------
// Negative tests: exact spec error codes.
// ---------------------------------------------------------------------------------------

/// Expands `input`, expecting a specific error code.
#[track_caller]
fn assert_error(input: &str, code: JsonLdErrorCode) {
    let err = expand(&json(input), &JsonLdOptions::default(), &NoopLoader)
        .expect_err("expansion should fail");
    assert_eq!(err.code(), code, "input: {}", input);
}

#[test]
fn invalid_id_value() {
    assert_error(r#"{"@id": 42, "http://ex/p": "v"}"#, JsonLdErrorCode::InvalidIdValue);
}

#[test]
fn colliding_keywords() {
    assert_error(
        r#"{"@context": {"id": "@id"}, "@id": "http://ex/1", "id": "http://ex/2"}"#,
        JsonLdErrorCode::CollidingKeywords,
    );
}

#[test]
fn invalid_value_object_disallowed_key() {
    assert_error(
        r#"{"http://ex/p": {"@value": "x", "@id": "http://ex/y"}}"#,
        JsonLdErrorCode::InvalidValueObject,
    );
}

#[test]
fn invalid_value_object_value() {
    // A @value with a non-scalar (and no @json type) is invalid.
    assert_error(
        r#"{"http://ex/p": {"@value": {"nested": "object"}}}"#,
        JsonLdErrorCode::InvalidValueObjectValue,
    );
}

#[test]
fn invalid_language_tagged_string() {
    assert_error(
        r#"{"http://ex/p": {"@value": "x", "@language": 42}}"#,
        JsonLdErrorCode::InvalidLanguageTaggedString,
    );
}

#[test]
fn invalid_type_value() {
    assert_error(
        r#"{"@type": {"not": "a string"}, "http://ex/p": "v"}"#,
        JsonLdErrorCode::InvalidTypeValue,
    );
}

#[test]
fn invalid_reverse_value() {
    assert_error(
        r#"{"@reverse": "not a map"}"#,
        JsonLdErrorCode::InvalidReverseValue,
    );
}

#[test]
fn invalid_nest_value() {
    assert_error(
        r#"{"@context": {"nest": "@nest"}, "nest": "not a map"}"#,
        JsonLdErrorCode::InvalidNestValue,
    );
}

#[test]
fn colliding_value_and_language_type() {
    // A value object may not carry both @type and @language.
    assert_error(
        r#"{"http://ex/p": {"@value": "x", "@type": "http://ex/T", "@language": "en"}}"#,
        JsonLdErrorCode::InvalidValueObject,
    );
}

// ---------------------------------------------------------------------------------------
// [FABLE-5] sq-oy1f.37 — expand-lane correctness regressions (W3C expand cases the native
// document-level oracle exposed). Each is NON-VACUOUS: it fails on the pre-fix expander.
// ---------------------------------------------------------------------------------------

#[test]
fn value_object_type_collapses_to_scalar_via_context_term() {
    // A value object's `@type` is a SINGLE value in the JSON-LD data model.
    // The general keyword path arrayifies `@type`; expansion must collapse the
    // single-element array back to a scalar (W3C expand/0002). Pre-fix the
    // expander raised `invalid typed value` on the arrayified `@type`.
    assert_expands(
        r#"{"@context": {"t2": "http://example.com/t2", "term2": "http://example.com/term2"},
            "term2": {"@value": "v2", "@type": "t2"}}"#,
        r#"[{"http://example.com/term2": [{"@value": "v2", "@type": "http://example.com/t2"}]}]"#,
    );
}

#[test]
fn value_object_type_scalar_for_already_absolute_iri() {
    // The same collapse when `@type` is an already-absolute IRI (W3C expand/0013).
    assert_expands(
        r#"[{"@id": "http://example.com/id1",
             "http://example.com/term2": [{"@value": "v2", "@type": "http://example.com/t2"}]}]"#,
        r#"[{"@id": "http://example.com/id1",
             "http://example.com/term2": [{"@value": "v2", "@type": "http://example.com/t2"}]}]"#,
    );
}

#[test]
fn set_container_empty_array_property_is_retained() {
    // A `@set`-container term whose value expands to an empty array must RETAIN
    // the property as `[]` — the `addValue` `asArray=true` rule (W3C expand/0004,
    // 0015). Pre-fix the property was dropped by the plain empty-array skip.
    assert_expands(
        r#"{"@context": {"myset": {"@id": "http://example.com/myset", "@container": "@set"}},
            "@id": "http://example.org/id",
            "myset": {"@set": []}}"#,
        r#"[{"@id": "http://example.org/id", "http://example.com/myset": []}]"#,
    );
}

#[test]
fn plain_empty_array_property_is_retained() {
    // A plain empty-array property value (no container mapping) is likewise
    // retained as `[]` on the forward-property path (W3C expand/0004 set3).
    assert_expands(
        r#"{"@id": "http://example.org/id", "http://example.org/set3": []}"#,
        r#"[{"@id": "http://example.org/id", "http://example.org/set3": []}]"#,
    );
}

#[test]
fn free_floating_value_object_is_dropped() {
    // A top-level free-floating value object (active property null) is dropped —
    // the whole document reduces to `[]` (W3C expand/0045).
    assert_expands(r#"{"@value": "free-floating value"}"#, r#"[]"#);
}

#[test]
fn free_floating_value_objects_under_graph_are_dropped() {
    // Free-floating value objects (even language-tagged / typed) directly under
    // `@graph` are dropped; a `@graph` of only such values reduces to `[]`
    // (W3C expand/0046).
    assert_expands(
        r#"{"@graph": [
            {"@value": "plain"},
            {"@value": "tagged", "@language": "en"},
            {"@value": "typed", "@type": "http://example.com/type"}
        ]}"#,
        r#"[]"#,
    );
}
