//! Integration tests for `omni_vector::ops::column_read`.

use std::sync::Arc;

use lance::deps::arrow_array::types::Int32Type;
use lance::deps::arrow_array::{DictionaryArray, Int32Array, StringArray};
use omni_vector::ops::column_read::get_utf8_at;

#[test]
fn get_utf8_at_string_array() {
    let arr = StringArray::from(vec!["a", "bb", "ccc"]);
    assert_eq!(get_utf8_at(&arr, 0), "a");
    assert_eq!(get_utf8_at(&arr, 1), "bb");
    assert_eq!(get_utf8_at(&arr, 2), "ccc");
}

#[test]
fn get_utf8_at_string_array_null() {
    let arr = StringArray::from(vec![Some("x"), None, Some("z")]);
    assert_eq!(get_utf8_at(&arr, 0), "x");
    assert_eq!(get_utf8_at(&arr, 1), "");
    assert_eq!(get_utf8_at(&arr, 2), "z");
}

#[test]
fn get_utf8_at_dictionary_array() {
    let values = StringArray::from(vec!["git", "writer", "knowledge"]);
    let keys = Int32Array::from(vec![0, 1, 0, 2]);
    let dict = match DictionaryArray::<Int32Type>::try_new(keys, Arc::new(values)) {
        Ok(dict) => dict,
        Err(error) => panic!("dictionary creation should succeed: {error}"),
    };
    assert_eq!(get_utf8_at(&dict, 0), "git");
    assert_eq!(get_utf8_at(&dict, 1), "writer");
    assert_eq!(get_utf8_at(&dict, 2), "git");
    assert_eq!(get_utf8_at(&dict, 3), "knowledge");
}

#[test]
fn get_utf8_at_dictionary_low_cardinality() {
    let values = StringArray::from(vec!["cat_a", "cat_b"]);
    let keys = Int32Array::from(vec![0, 0, 1, 0]);
    let dict = match DictionaryArray::<Int32Type>::try_new(keys, Arc::new(values)) {
        Ok(dict) => dict,
        Err(error) => panic!("dictionary creation should succeed: {error}"),
    };
    assert_eq!(get_utf8_at(&dict, 0), "cat_a");
    assert_eq!(get_utf8_at(&dict, 2), "cat_b");
}
