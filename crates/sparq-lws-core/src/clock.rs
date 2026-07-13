//! Target-portable wall clock for request and in-memory-store timestamps.

use std::time::SystemTime;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn now() -> SystemTime {
    SystemTime::now()
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn now() -> SystemTime {
    use std::time::{Duration, UNIX_EPOCH};

    let millis = js_sys::Date::now();
    if millis.is_finite() && millis >= 0.0 {
        UNIX_EPOCH + Duration::from_millis(millis as u64)
    } else {
        // A missing or invalid host clock must not trap the request handler. Epoch is conservative:
        // a later age calculation against a valid timestamp saturates to zero and keeps the blob.
        UNIX_EPOCH
    }
}
