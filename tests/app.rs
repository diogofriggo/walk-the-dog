// use futures::prelude::*;
// use wasm_bindgen::JsValue;
// use wasm_bindgen_futures::JsFuture;
// use wasm_bindgen_test::wasm_bindgen_test;

use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

// This runs a unit test in native Rust, so it can only use Rust APIs.
#[test]
fn rust_test() {
    assert_eq!(1, 1);
}
