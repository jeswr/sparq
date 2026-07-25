// [GPT-5.6] sq-1cnox — feature-gated native workspace FormDescription derivation.
//
// The webview sends serialized snapshots of the active workspace's data and shapes graphs,
// plus an RDF focus node, mode, and optional explicit shape. Feature-on builds parse those
// inputs, call sparq_forms::derive_form, and return its serde JSON verbatim. Feature-off
// builds keep the same registered Tauri command but fail loudly with a rebuild hint; they
// never substitute a demo or fabricated description.

/// Derive one complete `sparq_forms::FormDescription` from the active workspace snapshots.
///
/// `dataset` and `shapes` use `format` (the operational GUI sends N-Quads). `focus` and
/// `shape` are structured RDF-node strings: an absolute IRI or a `_:`-prefixed blank-node
/// label. `mode` is `"edit"` or `"view"`. The returned string is the direct serde JSON for
/// the native description; the frontend must not reconstruct keys, groups, or widgets.
#[tauri::command]
pub fn derive_form(
    dataset: String,
    shapes: String,
    format: String,
    focus: String,
    mode: String,
    shape: Option<String>,
) -> Result<String, String> {
    run_derive_form(&dataset, &shapes, &format, &focus, &mode, shape.as_deref())
}

#[cfg(feature = "forms")]
fn run_derive_form(
    dataset: &str,
    shapes: &str,
    format: &str,
    focus: &str,
    mode: &str,
    shape: Option<&str>,
) -> Result<String, String> {
    use sparq_forms::{FormOptions, Mode};

    let data = sparq_core::Graph::load_dataset(dataset, format).map_err(|error| {
        format!("Could not parse the workspace data graph as {format}: {error}")
    })?;
    let shapes = sparq_core::Graph::load_dataset(shapes, format).map_err(|error| {
        format!("Could not parse the workspace shapes graph as {format}: {error}")
    })?;
    let focus = parse_node(focus, "focus")?;
    let mode = match mode {
        "edit" => Mode::Edit,
        "view" => Mode::View,
        other => {
            return Err(format!(
                "Unsupported form mode {other:?}; expected \"edit\" or \"view\"."
            ));
        }
    };
    let shape = shape.map(|value| parse_node(value, "shape")).transpose()?;
    let options = FormOptions {
        mode,
        shape,
        ..FormOptions::default()
    };
    let description = sparq_forms::derive_form(&data, &shapes, &focus, &options);
    serde_json::to_string(&description)
        .map_err(|error| format!("Could not serialize the derived form description: {error}"))
}

#[cfg(feature = "forms")]
fn parse_node(value: &str, field: &str) -> Result<oxrdf::Term, String> {
    if let Some(label) = value.strip_prefix("_:") {
        return oxrdf::BlankNode::new(label)
            .map(oxrdf::Term::from)
            .map_err(|error| format!("Invalid {field} blank node {value:?}: {error}"));
    }
    oxrdf::NamedNode::new(value)
        .map(oxrdf::Term::from)
        .map_err(|error| format!("Invalid {field} IRI {value:?}: {error}"))
}

/// Lean-build stub: form derivation is absent, so report an actionable unavailable state
/// instead of returning a no-op or demo FormDescription.
#[cfg(not(feature = "forms"))]
fn run_derive_form(
    _dataset: &str,
    _shapes: &str,
    _format: &str,
    _focus: &str,
    _mode: &str,
    _shape: Option<&str>,
) -> Result<String, String> {
    Err(
        "Form derivation is unavailable in this build — rebuild the desktop app with \
         `cargo build --features forms` to enable the native forms bridge."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "forms")]
    const DATA: &str = r#"
        @prefix ex: <http://example.org/> .
        ex:alice a ex:Person ; ex:name "Alice" ; ex:audit "reviewed" .
    "#;

    #[cfg(feature = "forms")]
    const SHAPES: &str = r#"
        @prefix ex: <http://example.org/> .
        @prefix sh: <http://www.w3.org/ns/shacl#> .

        ex:PersonShape a sh:NodeShape ;
            sh:targetClass ex:Person ;
            sh:name "Person" ;
            sh:property [ sh:path ex:name ; sh:name "Name" ; sh:minCount 1 ] .

        ex:AuditShape a sh:NodeShape ;
            sh:name "Audit" ;
            sh:property [ sh:path ex:audit ; sh:name "Audit status" ] .
    "#;

    #[cfg(feature = "forms")]
    fn direct_description(mode: sparq_forms::Mode, shape: Option<&str>) -> String {
        let data = sparq_core::Graph::load_dataset(DATA, "turtle").expect("data fixture parses");
        let shapes =
            sparq_core::Graph::load_dataset(SHAPES, "turtle").expect("shapes fixture parses");
        let focus = oxrdf::Term::from(oxrdf::NamedNode::new_unchecked("http://example.org/alice"));
        let options = sparq_forms::FormOptions {
            mode,
            shape: shape.map(|iri| oxrdf::Term::from(oxrdf::NamedNode::new_unchecked(iri))),
            ..sparq_forms::FormOptions::default()
        };
        serde_json::to_string(&sparq_forms::derive_form(&data, &shapes, &focus, &options))
            .expect("direct FormDescription serializes")
    }

    #[test]
    #[cfg(feature = "forms")]
    fn applicable_shape_edit_and_view_match_direct_serialization() {
        for (mode_text, mode) in [
            ("edit", sparq_forms::Mode::Edit),
            ("view", sparq_forms::Mode::View),
        ] {
            let actual = run_derive_form(
                DATA,
                SHAPES,
                "turtle",
                "http://example.org/alice",
                mode_text,
                None,
            )
            .expect("applicable shape derives");
            assert_eq!(actual, direct_description(mode, None));

            let json: serde_json::Value =
                serde_json::from_str(&actual).expect("bridge returns JSON");
            assert_eq!(json["mode"], mode_text);
            assert_eq!(json["shape"]["value"], "http://example.org/PersonShape");
            assert_eq!(json["groups"][0]["fields"][0]["label"], "Name");
            assert_eq!(
                json["groups"][0]["fields"][0]["editable"],
                mode_text == "edit"
            );
        }
    }

    #[test]
    #[cfg(feature = "forms")]
    fn explicit_shape_request_matches_direct_serialization() {
        const AUDIT_SHAPE: &str = "http://example.org/AuditShape";
        let actual = run_derive_form(
            DATA,
            SHAPES,
            "turtle",
            "http://example.org/alice",
            "edit",
            Some(AUDIT_SHAPE),
        )
        .expect("explicit shape derives");
        assert_eq!(
            actual,
            direct_description(sparq_forms::Mode::Edit, Some(AUDIT_SHAPE))
        );

        let json: serde_json::Value = serde_json::from_str(&actual).expect("bridge returns JSON");
        assert_eq!(json["shape"]["value"], AUDIT_SHAPE);
        assert_eq!(json["shapes"][0]["via"], "explicit");
        assert_eq!(json["groups"][0]["fields"][0]["label"], "Audit status");
    }

    #[test]
    #[cfg(not(feature = "forms"))]
    fn lean_build_returns_actionable_unavailable_error() {
        let result = run_derive_form(
            "not parsed by the stub",
            "nor are shapes",
            "nquads",
            "http://example.org/alice",
            "edit",
            None,
        );
        let error = result.expect_err("lean build must not fabricate a FormDescription");
        assert!(
            error.contains("unavailable"),
            "error must name the state: {error}"
        );
        assert!(
            error.contains("--features forms"),
            "error must explain how to enable the bridge: {error}"
        );
    }
}
