// [GPT-5.6] sq-6xasp.3: execute the real LWS router through the exported wasm entry.
#![cfg(target_arch = "wasm32")]

use sparq_lws_wasm::SolidServer;
use wasm_bindgen_test::wasm_bindgen_test;

const BASE_URL: &str = "https://pod.example";
const OWNER: &str = "https://id.example/alice#me";
const INTRUDER: &str = "https://id.example/mallory#me";
const TURTLE: &[u8] = b"<https://pod.example/card> <http://xmlns.com/foaf/0.1/name> \"Ada\" .\n";

#[wasm_bindgen_test]
async fn put_get_round_trip_and_wac_denial_drive_the_real_router() {
    let server = SolidServer::new(BASE_URL.to_owned(), OWNER.to_owned())
        .expect("the in-memory pod must provision its owner ACL");

    let put = server
        .handle_request(
            "PUT".to_owned(),
            "/card".to_owned(),
            vec!["content-type".to_owned(), "text/turtle".to_owned()],
            TURTLE.to_vec(),
            Some(OWNER.to_owned()),
        )
        .await
        .expect("owner PUT must complete");
    assert_eq!(put.status(), 201, "a new LDP resource is created");

    let get = server
        .handle_request(
            "GET".to_owned(),
            "/card".to_owned(),
            vec!["accept".to_owned(), "text/turtle".to_owned()],
            Vec::new(),
            Some(OWNER.to_owned()),
        )
        .await
        .expect("owner GET must complete");
    assert_eq!(get.status(), 200);
    assert_eq!(get.body(), TURTLE, "PUT bytes must round-trip through GET");

    let denied = server
        .handle_request(
            "GET".to_owned(),
            "/card".to_owned(),
            Vec::new(),
            Vec::new(),
            Some(INTRUDER.to_owned()),
        )
        .await
        .expect("WAC denial is an HTTP response, not a host error");
    assert_eq!(denied.status(), 403, "an authenticated non-owner is denied");

    #[cfg(not(feature = "sparql-endpoint"))]
    {
        let absent = server
            .handle_request(
                "GET".to_owned(),
                "/sparql".to_owned(),
                Vec::new(),
                Vec::new(),
                Some(OWNER.to_owned()),
            )
            .await
            .expect("core-tier route absence is an HTTP response");
        assert_eq!(absent.status(), 404, "core wasm compiles /sparql out");
    }

    #[cfg(feature = "sparql-endpoint")]
    {
        // [GPT-5.6] sq-r1ei8: the full wasm artifact drives the same WAC-scoped
        // handler. The owner can query the card just PUT through LDP.
        let query = server
            .handle_request(
                "POST".to_owned(),
                "/sparql".to_owned(),
                vec![
                    "content-type".to_owned(),
                    "application/sparql-query".to_owned(),
                ],
                b"ASK { GRAPH <https://pod.example/card> { <https://pod.example/card> <http://xmlns.com/foaf/0.1/name> \"Ada\" } }".to_vec(),
                Some(OWNER.to_owned()),
            )
            .await
            .expect("full-tier SPARQL request completes");
        assert_eq!(query.status(), 200);
        assert!(
            String::from_utf8(query.body())
                .expect("SPARQL JSON is UTF-8")
                .contains("\"boolean\":true"),
            "the LDP-created resource is queryable in its authorized named graph"
        );
    }
}
