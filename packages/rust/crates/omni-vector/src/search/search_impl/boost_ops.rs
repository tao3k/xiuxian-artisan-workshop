use super::{KEYWORD_BOOST, VectorSearchResult, VectorStore};

impl VectorStore {
    /// Apply keyword boosting to search results.
    pub fn apply_keyword_boost(results: &mut [VectorSearchResult], keywords: &[String]) {
        if keywords.is_empty() {
            return;
        }
        let mut query_keywords: Vec<String> = Vec::new();
        for s in keywords {
            let lowered = s.to_lowercase();
            for w in lowered.split_whitespace() {
                query_keywords.push(w.to_string());
            }
        }

        for result in results {
            let mut keyword_score = 0.0;

            // 1. Boost from routing_keywords (Arrow-native or metadata fallback)
            let keywords_to_check: Vec<String> = if !result.routing_keywords.is_empty() {
                result
                    .routing_keywords
                    .split_whitespace()
                    .map(str::to_lowercase)
                    .collect()
            } else if let Some(keywords_arr) = result
                .metadata
                .get("routing_keywords")
                .and_then(|v| v.as_array())
            {
                keywords_arr
                    .iter()
                    .filter_map(|k| k.as_str().map(str::to_lowercase))
                    .collect()
            } else {
                vec![]
            };
            for kw in &query_keywords {
                if keywords_to_check.iter().any(|k| k.contains(kw)) {
                    keyword_score += KEYWORD_BOOST;
                }
            }

            // 2. Boost from intents (Arrow-native or metadata fallback)
            let intents_to_check: Vec<String> = if !result.intents.is_empty() {
                result
                    .intents
                    .split(" | ")
                    .map(|s| s.trim().to_lowercase())
                    .collect()
            } else if let Some(intents_arr) =
                result.metadata.get("intents").and_then(|v| v.as_array())
            {
                intents_arr
                    .iter()
                    .filter_map(|k| k.as_str().map(str::to_lowercase))
                    .collect()
            } else {
                vec![]
            };
            for kw in &query_keywords {
                if intents_to_check.iter().any(|k| k.contains(kw)) {
                    keyword_score += KEYWORD_BOOST * 1.2; // Intents are higher signal
                }
            }

            let tool_name_lower = if result.tool_name.is_empty() {
                result.id.to_lowercase()
            } else {
                result.tool_name.to_lowercase()
            };
            let content_lower = result.content.to_lowercase();
            for kw in &query_keywords {
                if tool_name_lower.contains(kw) {
                    keyword_score += KEYWORD_BOOST * 0.5;
                }
                if content_lower.contains(kw) {
                    keyword_score += KEYWORD_BOOST * 0.3;
                }
            }
            let keyword_bonus = keyword_score * 0.3f32;
            result.distance = (result.distance - f64::from(keyword_bonus)).max(0.0);
        }
    }
}
