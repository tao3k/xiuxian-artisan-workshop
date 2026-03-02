use std::path::Path;

use super::PromptScanner;

#[test]
fn test_scan_empty_dir() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = PromptScanner::new();
    let prompts = scanner.scan(Path::new("/nonexistent"), "test")?;
    assert!(prompts.is_empty());

    Ok(())
}

#[test]
fn test_scan_finds_prompt() -> Result<(), Box<dyn std::error::Error>> {
    let scanner = PromptScanner::new();
    let files = vec![(
        "/virtual/skill/scripts/prompt.py".to_string(),
        r#"
@prompt(
    name="analyze_code",
    description="Analyze code structure"
)
def analyze_code(file_path: str):
    '''Analyze the given code file.'''
    return f"Please analyze {file_path}"
"#
        .to_string(),
    )];

    let prompts = scanner.scan_paths(&files, "test")?;
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].name, "analyze_code");

    Ok(())
}
