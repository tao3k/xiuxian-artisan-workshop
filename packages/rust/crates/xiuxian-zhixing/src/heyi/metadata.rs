use serde_json::Value;

pub(super) fn parse_carryover_count(value: &Value) -> Option<i32> {
    if let Some(raw) = value.as_i64() {
        return i32::try_from(raw).ok();
    }
    value.as_str()?.parse::<i32>().ok()
}
