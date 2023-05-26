#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

use js_sys::Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use mvt_reader::wasm::Reader;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn simple_fixture_test() {
  let data = include_bytes!("../mvt-fixtures/fixtures/032/tile.mvt");
  let reader = Reader::new(data.to_vec());

  assert!(array_contains_string(reader.get_layer_names(), "hello"));
}

fn array_contains_string(js_array: JsValue, search_string: &str) -> bool {
  if let Some(array) = js_array.dyn_ref::<Array>() {
    for i in 0..array.length() {
      let element = array.get(i);

      if let Some(string) = element.as_string() {
        if string == search_string {
          return true;
        }
      }
    }
  }
  false
}
