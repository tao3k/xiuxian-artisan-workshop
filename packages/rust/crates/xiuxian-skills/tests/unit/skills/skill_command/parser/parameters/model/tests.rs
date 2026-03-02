use serde_json::json;

use super::ParsedParameter;

fn param(type_annotation: Option<&str>, default_value: Option<&str>) -> ParsedParameter {
    ParsedParameter {
        name: "value".to_string(),
        type_annotation: type_annotation.map(str::to_string),
        has_default: default_value.is_some(),
        default_value: default_value.map(str::to_string),
    }
}

#[test]
fn test_infer_json_type_for_scalar() {
    assert_eq!(param(Some("int"), None).infer_json_type(), json!("integer"));
    assert_eq!(
        param(Some("float"), None).infer_json_type(),
        json!("number")
    );
    assert_eq!(
        param(Some("bool"), None).infer_json_type(),
        json!("boolean")
    );
}

#[test]
fn test_infer_json_type_for_collections() {
    assert_eq!(
        param(Some("list[int]"), None).infer_json_type(),
        json!({"type":"array","items":{"type":"integer"}})
    );
    assert_eq!(
        param(Some("Dict[str, int]"), None).infer_json_type(),
        json!({"type":"object","additionalProperties":true})
    );
}

#[test]
fn test_infer_json_type_for_literal() {
    assert_eq!(
        param(Some("Literal['fast', 'slow']"), None).infer_json_type(),
        json!({"type":"string","enum":["fast","slow"]})
    );
}

#[test]
fn test_to_json_schema_property_with_and_without_default() {
    assert_eq!(
        param(Some("str"), Some("'abc'")).to_json_schema_property(),
        json!({"type":"string","default":"'abc'"})
    );
    assert_eq!(
        param(Some("str"), Some("None")).to_json_schema_property(),
        json!({"type":"string"})
    );
}
