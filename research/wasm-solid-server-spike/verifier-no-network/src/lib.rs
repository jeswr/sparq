// [GPT-5.6] sq-6xasp.1: isolated negative dependency probe; this is not shipped.
#![forbid(unsafe_code)]
//! Forces the pinned verifier's no-network public core into a wasm build.

use solid_oidc_verifier::{InMemoryReplayStore, StaticJwksProvider};

/// Name both in-memory verifier seams so the dependency cannot be optimized away.
pub fn core_seam_type_names() -> (&'static str, &'static str) {
    (
        std::any::type_name::<StaticJwksProvider>(),
        std::any::type_name::<InMemoryReplayStore>(),
    )
}
