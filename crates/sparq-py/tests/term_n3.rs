// The sparq-py cdylib disables its lib test harness because linking a PyO3
// extension-module harness fails on macOS. Compile the pure renderer directly so
// these tests remain ordinary Rust integration tests with no Python dependency.
#[path = "../src/term_render.rs"]
mod term_render;

use term_render::{render_n3, XSD_STRING};

#[test]
fn renders_resource_term_kinds() {
    assert_eq!(
        render_n3("uri", "https://example.com/resource", None, None, None),
        "<https://example.com/resource>"
    );
    assert_eq!(render_n3("bnode", "node-1", None, None, None), "_:node-1");

    let triple = "<< <https://example.com/s> <https://example.com/p> \"object\" >>";
    assert_eq!(render_n3("triple", triple, None, None, None), triple);
}

#[test]
fn escapes_literal_backslashes_quotes_and_newlines_in_order() {
    assert_eq!(
        render_n3(
            "literal",
            "first\\\"line\nsecond",
            None,
            Some(XSD_STRING),
            None,
        ),
        "\"first\\\\\\\"line\\nsecond\""
    );
}

#[test]
fn renders_plain_and_typed_literals() {
    assert_eq!(render_n3("literal", "plain", None, None, None), "\"plain\"");
    assert_eq!(
        render_n3("literal", "plain", None, Some(XSD_STRING), None),
        "\"plain\""
    );
    assert_eq!(
        render_n3(
            "literal",
            "42",
            None,
            Some("http://www.w3.org/2001/XMLSchema#integer"),
            None,
        ),
        "\"42\"^^<http://www.w3.org/2001/XMLSchema#integer>"
    );
}

#[test]
fn renders_language_and_directional_literals() {
    assert_eq!(
        render_n3("literal", "bonjour", Some("fr"), None, None),
        "\"bonjour\"@fr"
    );
    assert_eq!(
        render_n3("literal", "hello", Some("en"), None, Some("ltr"),),
        "\"hello\"@en--ltr"
    );
}
