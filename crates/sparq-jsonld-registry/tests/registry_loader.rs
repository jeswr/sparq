use sparq_jsonld::{expand, DocumentLoader, Json, JsonLdErrorCode, JsonLdOptions, NoopLoader};
use sparq_jsonld_registry::RegistryLoader;

const SCHEMA_CONTEXT: &str = include_str!("../contexts/schema-org.jsonld");
const SCHEMA_IRI: &str = "https://schema.org/docs/jsonldcontext.jsonld";

#[test]
fn registered_context_round_trips_vendored_bytes() {
    let document = RegistryLoader::new().load_document(SCHEMA_IRI).unwrap();
    assert_eq!(document.document.as_bytes(), SCHEMA_CONTEXT.as_bytes());
    assert_eq!(document.document_url, SCHEMA_IRI);
    assert_eq!(
        document.content_type.as_deref(),
        Some("application/ld+json")
    );
}

#[test]
fn unregistered_context_has_exact_noop_denial() {
    let iri = "https://example.invalid/unregistered-context";
    let registry_error = RegistryLoader::new().load_document(iri).unwrap_err();
    let noop_error = NoopLoader.load_document(iri).unwrap_err();
    assert_eq!(
        registry_error.code(),
        JsonLdErrorCode::LoadingDocumentFailed
    );
    assert_eq!(registry_error.code(), noop_error.code());
    assert_eq!(registry_error.detail(), noop_error.detail());
}

#[test]
fn schema_context_expands_fully_offline() {
    let input = Json::parse(
        r#"{"@context":"https://schema.org/docs/jsonldcontext.jsonld","@type":"Person","name":"Ada"}"#,
    )
    .unwrap();
    let noop_error = expand(&input, &JsonLdOptions::default(), &NoopLoader).unwrap_err();
    assert_eq!(
        noop_error.code(),
        JsonLdErrorCode::LoadingRemoteContextFailed
    );

    let expanded = expand(&input, &JsonLdOptions::default(), &RegistryLoader::new()).unwrap();
    let expected = Json::parse(
        r#"[{"@type":["https://schema.org/Person"],"https://schema.org/name":[{"@value":"Ada"}]}]"#,
    )
    .unwrap();
    assert_eq!(expanded, expected);
}
