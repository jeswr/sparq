// [GPT-5.6] sq-7o2fb — standalone JSON-LD context-stage microbenchmark harness.

use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use sparq_jsonld::context::ActiveContext;
use sparq_jsonld::{
    DocumentLoader, Json, JsonLdError, JsonLdErrorCode, JsonLdOptions, RemoteDocument,
};

const BASE: &str = "https://bench.example/";

#[derive(Default)]
struct MapLoader(BTreeMap<String, String>);

impl DocumentLoader for MapLoader {
    fn load_document(&self, url: &str) -> Result<RemoteDocument, JsonLdError> {
        self.0
            .get(url)
            .map(|document| RemoteDocument::new(document.clone(), url))
            .ok_or_else(|| JsonLdError::new(JsonLdErrorCode::LoadingDocumentFailed))
    }
}

struct Fixture {
    name: &'static str,
    context: Json,
    loader: MapLoader,
    expected_terms: usize,
}

impl Fixture {
    fn process(&self) -> ActiveContext {
        ActiveContext::new(Some(BASE))
            .process(
                &self.context,
                Some(BASE),
                &self.loader,
                &JsonLdOptions::default(),
            )
            .unwrap_or_else(|error| panic!("{} context failed: {error}", self.name))
    }

    fn assert_oracle(&self) {
        let actual = self.process().term_count();
        assert_eq!(
            actual, self.expected_terms,
            "{} term-count oracle",
            self.name
        );
        println!(
            "oracle shape={} expected_terms={} actual_terms={} equal=true",
            self.name, self.expected_terms, actual
        );
    }
}

fn flat_fixture() -> Fixture {
    Fixture {
        name: "flat",
        context: Json::parse(&term_object("https://schema.example/", 24))
            .expect("generated flat context is valid JSON"),
        loader: MapLoader::default(),
        expected_terms: 24,
    }
}

fn imported_fixture() -> Fixture {
    let mut documents = BTreeMap::new();
    documents.insert(
        format!("{BASE}level-1.jsonld"),
        "{\"@context\":{\"level1\":\"https://chain.example/level1\"}}".to_owned(),
    );
    documents.insert(
        format!("{BASE}level-2.jsonld"),
        "{\"@context\":{\"level2\":\"https://chain.example/level2\"}}".to_owned(),
    );
    documents.insert(
        format!("{BASE}level-3.jsonld"),
        "{\"@context\":{\"level3\":\"https://chain.example/level3\"}}".to_owned(),
    );
    Fixture {
        name: "deep-import",
        context: Json::parse(&format!(
            "[{{\"@import\":\"{BASE}level-1.jsonld\"}},{{\"@import\":\"{BASE}level-2.jsonld\"}},{{\"@import\":\"{BASE}level-3.jsonld\"}},{{\"local\":\"https://chain.example/local\"}}]"
        ))
        .expect("generated import context is valid JSON"),
        loader: MapLoader(documents),
        expected_terms: 4,
    }
}

fn vocab_fixture() -> Fixture {
    let mut fields = vec!["\"@vocab\":\"https://vocab.example/\"".to_owned()];
    fields.extend((0..96).map(|index| format!("\"term{index}\":\"item{index}\"")));
    Fixture {
        name: "many-term-vocab",
        context: Json::parse(&format!("{{{}}}", fields.join(",")))
            .expect("generated vocab context is valid JSON"),
        loader: MapLoader::default(),
        expected_terms: 96,
    }
}

fn term_object(prefix: &str, count: usize) -> String {
    let terms = (0..count)
        .map(|index| format!("\"term{index}\":\"{prefix}term{index}\""))
        .collect::<Vec<_>>();
    format!("{{{}}}", terms.join(","))
}

fn fixtures() -> Vec<Fixture> {
    vec![flat_fixture(), imported_fixture(), vocab_fixture()]
}

fn main() {
    let smoke = std::env::args().skip(1).try_fold(false, |_seen, argument| {
        if argument == "--smoke" {
            Ok(true)
        } else {
            Err(argument)
        }
    });
    let smoke = match smoke {
        Ok(smoke) => smoke,
        Err(argument) => {
            eprintln!("unknown argument: {argument}\nusage: jsonld-context-bench [--smoke]");
            std::process::exit(2);
        }
    };
    let iterations = if smoke { 8 } else { 2_000 };
    let fixtures = fixtures();

    println!("NON-CANONICAL work-box timings; correctness oracles run before timing");
    for fixture in &fixtures {
        fixture.assert_oracle();
    }

    for fixture in &fixtures {
        let started = Instant::now();
        for _ in 0..iterations {
            let active = fixture.process();
            black_box(active.inverse_context());
        }
        let elapsed = started.elapsed();
        println!(
            "timing shape={} iterations={} elapsed_ns={} contexts_per_second={:.2}",
            fixture.name,
            iterations,
            elapsed.as_nanos(),
            iterations as f64 / elapsed.as_secs_f64()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shape_satisfies_its_independent_term_count_oracle() {
        for fixture in fixtures() {
            assert_eq!(
                fixture.process().term_count(),
                fixture.expected_terms,
                "{}",
                fixture.name
            );
        }
    }

    #[test]
    fn imported_shape_reaches_every_chain_level() {
        let active = imported_fixture().process();
        for term in ["local", "level1", "level2", "level3"] {
            assert!(active.has_term(term), "missing imported term {term}");
        }
    }

    #[test]
    fn vocab_shape_resolves_relative_term_mappings() {
        let active = vocab_fixture().process();
        assert_eq!(
            active.term_definition("term95").and_then(|term| term.iri()),
            Some("https://vocab.example/item95")
        );
    }

    #[test]
    fn no_loader_is_needed_for_flat_contexts() {
        let active = ActiveContext::new(Some(BASE))
            .process(
                &flat_fixture().context,
                Some(BASE),
                &sparq_jsonld::NoopLoader,
                &JsonLdOptions::default(),
            )
            .expect("flat context must not perform remote loads");
        assert_eq!(active.term_count(), 24);
    }
}
