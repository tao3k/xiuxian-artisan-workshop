use anyhow::{Result, bail};
use redis::Value;
use std::collections::HashMap;

use super::types::MemoryStreamEvent;

pub(super) fn parse_xreadgroup_reply(reply: Value) -> Result<Vec<MemoryStreamEvent>> {
    match reply {
        Value::Nil => Ok(Vec::new()),
        Value::Array(streams) => Ok(parse_streams_array(streams)),
        Value::Map(streams) => Ok(parse_streams_map(streams)),
        other => bail!("unexpected xreadgroup reply value type: {other:?}"),
    }
}

fn parse_streams_array(streams: Vec<Value>) -> Vec<MemoryStreamEvent> {
    let mut events = Vec::new();
    for stream in streams {
        let Value::Array(stream_entry) = stream else {
            continue;
        };
        if stream_entry.len() < 2 {
            continue;
        }
        events.extend(parse_event_entries(stream_entry.get(1)));
    }
    events
}

fn parse_streams_map(streams: Vec<(Value, Value)>) -> Vec<MemoryStreamEvent> {
    let mut events = Vec::new();
    for (_, stream_entries) in streams {
        events.extend(parse_event_entries(Some(&stream_entries)));
    }
    events
}

fn parse_event_entries(entries: Option<&Value>) -> Vec<MemoryStreamEvent> {
    let Some(entries) = entries else {
        return Vec::new();
    };
    let Value::Array(entries) = entries else {
        return Vec::new();
    };

    let mut events = Vec::with_capacity(entries.len());
    for entry in entries {
        let Value::Array(parts) = entry else {
            continue;
        };
        if parts.len() < 2 {
            continue;
        }
        let Some(event_id) = value_to_string(parts.first()) else {
            continue;
        };
        let fields = parse_fields(parts.get(1));
        events.push(MemoryStreamEvent {
            id: event_id,
            fields,
        });
    }
    events
}

fn parse_fields(value: Option<&Value>) -> HashMap<String, String> {
    let Some(value) = value else {
        return HashMap::new();
    };

    match value {
        Value::Map(entries) => {
            let mut fields = HashMap::with_capacity(entries.len());
            for (field, field_value) in entries {
                let Some(field_name) = value_to_string(Some(field)) else {
                    continue;
                };
                let value = value_to_string(Some(field_value)).unwrap_or_default();
                fields.insert(field_name, value);
            }
            fields
        }
        Value::Array(parts) => {
            let mut fields = HashMap::with_capacity(parts.len() / 2);
            for pair in parts.chunks(2) {
                let Some(field_name) = value_to_string(pair.first()) else {
                    continue;
                };
                let value = value_to_string(pair.get(1)).unwrap_or_default();
                fields.insert(field_name, value);
            }
            fields
        }
        _ => HashMap::new(),
    }
}

fn value_to_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::BulkString(bytes) => Some(String::from_utf8_lossy(bytes).to_string()),
        Value::SimpleString(value) => Some(value.clone()),
        Value::Okay => Some("OK".to_string()),
        Value::Int(value) => Some(value.to_string()),
        Value::Double(value) => Some(value.to_string()),
        Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    }
}
