// [GPT-5.6] sq-6xasp.1: execute the router proof in Node's wasm runtime.
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
async fn routes_one_ldp_request_under_wasm_bindgen_futures() {
    let body = sparq_lws_wasm_router_probe::route_one_ldp_request_wasm()
        .await
        .expect("the wasm request future must complete");
    assert!(body.contains("foaf/0.1/name"));
    assert!(body.contains("\"Ada\""));
}
