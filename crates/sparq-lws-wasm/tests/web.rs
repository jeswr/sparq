// [GPT-5.6] sq-6xasp.3: execute the real LWS router through the exported wasm entry.
// [SONNET-4.6] sq-wubkf: and the linear-memory admission refusal.
#![cfg(target_arch = "wasm32")]

use sparq_lws_wasm::memory;
use sparq_lws_wasm::SolidServer;
use wasm_bindgen_test::wasm_bindgen_test;

const BASE_URL: &str = "https://pod.example";
const OWNER: &str = "https://id.example/alice#me";
const INTRUDER: &str = "https://id.example/mallory#me";
const TURTLE: &[u8] = b"<https://pod.example/card> <http://xmlns.com/foaf/0.1/name> \"Ada\" .\n";

// [SONNET-4.6] sq-wubkf: the bounded allocator is only installed on wasm32, so this is the only
// place the live-byte accounting and the 507 refusal can be exercised end to end. A ceiling below
// the module's own live total must refuse the very next request with a clean HTTP 507 — an HTTP
// answer, not the `unreachable` trap an allocation failure at the real ceiling would produce.
#[wasm_bindgen_test]
async fn a_ceiling_below_the_live_total_refuses_the_next_request_with_507() {
    let server = SolidServer::new(BASE_URL.to_owned(), OWNER.to_owned())
        .expect("the in-memory pod must provision its owner ACL");
    let restore = memory::ceiling_bytes();

    assert!(
        memory::live_bytes() > 0,
        "the bounded allocator must be installed on wasm32 and counting"
    );
    assert!(
        memory::peak_bytes() >= memory::live_bytes(),
        "the high-water mark never trails the live total"
    );

    // A ceiling of one byte cannot admit anything: the projection always adds the fixed headroom.
    memory::set_ceiling_bytes(1);
    let refused = server
        .handle_request(
            "PUT".to_owned(),
            "/card".to_owned(),
            vec!["content-type".to_owned(), "text/turtle".to_owned()],
            TURTLE.to_vec(),
            Some(OWNER.to_owned()),
        )
        .await
        .expect("an exhausted linear memory is an HTTP response, not a host error");
    assert_eq!(refused.status(), 507, "insufficient storage, not a trap");
    assert_eq!(
        String::from_utf8(refused.body()).expect("the 507 body is UTF-8"),
        "insufficient storage",
        "byte-identical to the store-quota 507 the host already handles"
    );

    memory::set_ceiling_bytes(restore as usize);
    let admitted = server
        .handle_request(
            "PUT".to_owned(),
            "/card".to_owned(),
            vec!["content-type".to_owned(), "text/turtle".to_owned()],
            TURTLE.to_vec(),
            Some(OWNER.to_owned()),
        )
        .await
        .expect("owner PUT must complete once the ceiling is restored");
    assert_eq!(
        admitted.status(),
        201,
        "the refusal is admission control, not a poisoned instance"
    );
}

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
