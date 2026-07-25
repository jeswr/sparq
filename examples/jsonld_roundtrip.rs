//! [GPT-5.6] Executable expand → flatten → compact → frame JSON-LD demonstration.

use sparq_jsonld::compact::compact;
use sparq_jsonld::frame::{frame, FrameOptions};
use sparq_jsonld::{expand, flatten, Json, JsonLdError, JsonLdOptions, NoopLoader};

const DOCUMENT: &str = r#"{
  "@context": {
    "ex": "https://example.com/",
    "name": "ex:name",
    "Person": "ex:Person"
  },
  "@id": "ex:alice",
  "@type": "Person",
  "name": "Alice"
}"#;

const CONTEXT: &str = r#"{
  "ex": "https://example.com/",
  "name": "ex:name",
  "Person": "ex:Person"
}"#;

const FRAME: &str = r#"{
  "@context": {
    "ex": "https://example.com/",
    "name": "ex:name",
    "Person": "ex:Person"
  },
  "@type": "Person"
}"#;

fn pipeline() -> Result<(Json, Json, Json, Json), JsonLdError> {
    let input = Json::parse(DOCUMENT).expect("inline document is valid JSON");
    let context = Json::parse(CONTEXT).expect("inline context is valid JSON");
    let frame_document = Json::parse(FRAME).expect("inline frame is valid JSON");
    let options = JsonLdOptions::default();
    let loader = NoopLoader;

    let expanded = expand(&input, &options, &loader)?;
    let flattened = flatten(&expanded, &options, &loader)?;
    let compacted = compact(&flattened, &context, &options, &loader)?;
    let framed = frame(
        &compacted,
        &frame_document,
        &options,
        &FrameOptions::default(),
        &loader,
    )?;

    Ok((expanded, flattened, compacted, framed))
}

fn print_stage(label: &str, value: &Json) {
    let mut rendered = String::new();
    value.write(&mut rendered);
    println!("{label}:\n{rendered}\n");
}

fn main() -> Result<(), JsonLdError> {
    let (expanded, flattened, compacted, framed) = pipeline()?;
    print_stage("expanded", &expanded);
    print_stage("flattened", &flattened);
    print_stage("compacted", &compacted);
    print_stage("framed", &framed);

    let expected = Json::parse(
        r#"{"@context":{"ex":"https://example.com/","name":"ex:name","Person":"ex:Person"},"@id":"ex:alice","@type":"Person","name":"Alice"}"#,
    )
    .expect("inline expectation is valid JSON");
    assert_eq!(framed, expected, "framed document must round-trip");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_frame_matches_the_round_trip_oracle() {
        let (_, _, _, framed) = pipeline().expect("offline pipeline succeeds");
        let expected = Json::parse(
            r#"{"@context":{"ex":"https://example.com/","name":"ex:name","Person":"ex:Person"},"@id":"ex:alice","@type":"Person","name":"Alice"}"#,
        )
        .expect("valid expected JSON");
        assert_eq!(framed, expected);
    }
}
