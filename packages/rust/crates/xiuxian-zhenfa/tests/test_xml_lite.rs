use xiuxian_zhenfa::{extract_tag_f32, extract_tag_value};

#[test]
fn extract_tag_value_returns_trimmed_payload() {
    let value = extract_tag_value("<score> 0.85 </score>", "score");
    assert_eq!(value, Some("0.85"));
}

#[test]
fn extract_tag_value_returns_none_when_tag_missing() {
    assert_eq!(extract_tag_value("<decision>run</decision>", "score"), None);
}

#[test]
fn extract_tag_f32_parses_numeric_payload() {
    let score = extract_tag_f32("<score>0.42</score>", "score");
    assert_eq!(score, Some(0.42));
}
