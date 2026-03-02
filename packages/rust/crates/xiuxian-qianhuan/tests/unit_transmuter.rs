//! Unit tests for tone transmutation behavior.

use std::collections::HashMap;
use xiuxian_qianhuan::{MockTransmuter, PersonaProfile, ToneTransmuter};

#[tokio::test]
async fn test_mock_transmute() {
    let transmuter = MockTransmuter;

    // Test with Artisan ID
    let artisan = PersonaProfile {
        id: "artisan-test".to_string(),
        name: "Artisan".to_string(),
        voice_tone: "Tone".to_string(),
        style_anchors: vec!["Precision".to_string()],
        cot_template: "CoT".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
        background: None,
        guidelines: vec![],
    };

    let result = match transmuter.transmute("fact", &artisan).await {
        Ok(output) => output,
        Err(error) => panic!("artisan transmute should succeed: {error}"),
    };
    assert!(result.contains("Artisan Report"));
    assert!(result.contains("fact"));

    // Test with Cultivator ID
    let cultivator = PersonaProfile {
        id: "cultivator-test".to_string(),
        name: "Cultivator".to_string(),
        voice_tone: "Zen".to_string(),
        style_anchors: vec!["Dao".to_string()],
        cot_template: "CoT".to_string(),
        forbidden_words: vec![],
        metadata: HashMap::new(),
        background: None,
        guidelines: vec![],
    };

    let result = match transmuter.transmute("fact", &cultivator).await {
        Ok(output) => output,
        Err(error) => panic!("cultivator transmute should succeed: {error}"),
    };
    assert!(result.contains("The Dao reveals"));
    assert!(result.contains("fact"));
}
