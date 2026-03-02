use super::ZhixingHeyi;
use serde_json::{Map, Value, json};

const DEFAULT_ZHIXING_DOMAIN: &str = "zhixing.agenda";
const MARKDOWN_V2_SPECIAL_CHARS: [char; 19] = [
    '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!', '\\',
];

impl ZhixingHeyi {
    pub(super) fn render_with_qianhuan_context(
        &self,
        template_name: &str,
        payload: Value,
        state_context: &str,
    ) -> crate::Result<String> {
        let mut root = match payload {
            Value::Object(map) => map,
            value => {
                let mut map = Map::new();
                map.insert("payload".to_string(), value);
                map
            }
        };

        let persona_json = self.active_persona.as_ref().map(|persona| {
            json!({
                "id": persona.id,
                "name": persona.name,
                "voice_tone": persona.voice_tone,
                "style_anchors": persona.style_anchors,
            })
        });

        root.insert(
            "qianhuan".to_string(),
            json!({
                "state_context": state_context,
                "injected_context": self.manifestation.inject_context(state_context),
                "persona": persona_json,
                "persona_id": self.active_persona.as_ref().map(|persona| persona.id.as_str()),
                "domain": DEFAULT_ZHIXING_DOMAIN,
            }),
        );

        self.manifestation
            .render_template(template_name, Value::Object(root))
            .map_err(|error| {
                crate::Error::Internal(format!(
                    "Failed to render template `{template_name}` with qianhuan context: {error}"
                ))
            })
    }
}

pub(super) fn escape_markdown_v2(raw: &str) -> String {
    let mut escaped = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if MARKDOWN_V2_SPECIAL_CHARS.contains(&ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}
