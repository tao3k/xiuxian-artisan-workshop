//! Tests for `list_all_tools` and dictionary-encoded metadata behavior.

use anyhow::Result;
use omni_vector::VectorStore;
use xiuxian_skills::skills::{ToolAnnotations, ToolRecord};

async fn create_store(
    path_name: &str,
    dimension: usize,
) -> Result<(tempfile::TempDir, VectorStore)> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join(path_name);
    let db_path_str = db_path.to_string_lossy().into_owned();
    let store = VectorStore::new(db_path_str.as_str(), Some(dimension)).await?;
    Ok((temp_dir, store))
}

fn parse_rows(result: &str) -> Result<Vec<serde_json::Value>> {
    Ok(serde_json::from_str(result)?)
}

/// Test that `list_all_tools` correctly reads tool records from a table
/// with dictionary-encoded columns (`skill_name`, `category`, `tool_name`, etc).
#[tokio::test]
async fn test_list_all_tools_with_dictionary_columns() -> Result<()> {
    let (_temp_dir, store) = create_store("test_list_tools", 1024).await?;

    // Create sample tool records (using the correct ToolRecord structure)
    let tools = vec![
        ToolRecord {
            tool_name: "commit".to_string(),
            description: "Create a git commit with proper message".to_string(),
            skill_name: "git".to_string(),
            file_path: "git/scripts/commit.py".to_string(),
            function_name: "commit".to_string(),
            execution_mode: "script".to_string(),
            keywords: vec!["git".to_string(), "commit".to_string()],
            intents: vec!["Save code changes".to_string()],
            file_hash: "abc123".to_string(),
            input_schema: r#"{"type": "object", "properties": {"message": {"type": "string"}}}"#
                .to_string(),
            docstring: "Commit changes".to_string(),
            category: "version_control".to_string(),
            annotations: ToolAnnotations::default(),
            parameters: vec!["message".to_string()],
            skill_tools_refers: vec![],
            resource_uri: String::new(),
        },
        ToolRecord {
            tool_name: "push".to_string(),
            description: "Push commits to remote".to_string(),
            skill_name: "git".to_string(),
            file_path: "git/scripts/push.py".to_string(),
            function_name: "push".to_string(),
            execution_mode: "script".to_string(),
            keywords: vec!["git".to_string(), "push".to_string()],
            intents: vec!["Upload code".to_string()],
            file_hash: "def456".to_string(),
            input_schema: r#"{"type": "object"}"#.to_string(),
            docstring: "Push to remote".to_string(),
            category: "version_control".to_string(),
            annotations: ToolAnnotations::default(),
            parameters: vec![],
            skill_tools_refers: vec![],
            resource_uri: String::new(),
        },
        ToolRecord {
            tool_name: "save".to_string(),
            description: "Save content to file".to_string(),
            skill_name: "writer".to_string(),
            file_path: "writer/scripts/save.py".to_string(),
            function_name: "save".to_string(),
            execution_mode: "script".to_string(),
            keywords: vec!["write".to_string(), "save".to_string()],
            intents: vec!["Write to file".to_string()],
            file_hash: "ghi789".to_string(),
            input_schema: r#"{"type": "object", "properties": {"path": {"type": "string"}}}"#
                .to_string(),
            docstring: "Save file".to_string(),
            category: "file_editor".to_string(),
            annotations: ToolAnnotations::default(),
            parameters: vec!["path".to_string()],
            skill_tools_refers: vec![],
            resource_uri: String::new(),
        },
    ];

    // Add tools to table (this uses dictionary encoding for skill_name, category, tool_name)
    store.add("test_tools", tools).await?;

    // Verify count
    let count = store.count("test_tools").await?;
    assert_eq!(count, 3, "Should have 3 tools");

    // CRITICAL TEST: list_all_tools should correctly read dictionary-encoded columns
    let result = store.list_all_tools("test_tools", None, None).await?;

    // Parse the JSON result
    let tools_list = parse_rows(&result)?;
    assert_eq!(tools_list.len(), 3, "Should return 3 tools");

    // Verify each tool has correct skill_name (this is where the bug manifested!)
    // Shape is { id, content, metadata }; skill_name is in metadata for tool tables.
    let mut skill_names: Vec<String> = tools_list
        .iter()
        .map(|t| {
            t.get("metadata")
                .and_then(|m| m.get("skill_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        })
        .collect();
    skill_names.sort();

    assert_eq!(skill_names, vec!["git", "git", "writer"]);

    // Verify tool names are correctly extracted
    let tool_names: Vec<String> = tools_list
        .iter()
        .map(|t| {
            t.get("metadata")
                .and_then(|m| m.get("tool_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        })
        .collect();
    assert!(tool_names.contains(&"commit".to_string()));
    assert!(tool_names.contains(&"push".to_string()));
    assert!(tool_names.contains(&"save".to_string()));
    Ok(())
}

/// Test that `list_all_tools` handles an empty table gracefully.
#[tokio::test]
async fn test_list_all_tools_empty_table() -> Result<()> {
    let (_temp_dir, store) = create_store("test_empty", 1024).await?;

    // Query non-existent table - should return empty array
    let result = store.list_all_tools("non_existent", None, None).await?;
    let tools_list = parse_rows(&result)?;
    assert!(tools_list.is_empty());
    Ok(())
}

/// Test that `list_all_tools` returns the correct `content` field (from description).
#[tokio::test]
async fn test_list_all_tools_content_field() -> Result<()> {
    let (_temp_dir, store) = create_store("test_content", 1024).await?;

    let tools = vec![ToolRecord {
        tool_name: "tool".to_string(),
        description: "This is the description for embedding".to_string(),
        skill_name: "test".to_string(),
        file_path: "test/tool.py".to_string(),
        function_name: "tool".to_string(),
        execution_mode: "script".to_string(),
        keywords: vec![],
        intents: vec![],
        file_hash: "hash".to_string(),
        input_schema: "{}".to_string(),
        docstring: "Test".to_string(),
        category: "test".to_string(),
        annotations: ToolAnnotations::default(),
        parameters: vec![],
        skill_tools_refers: vec![],
        resource_uri: String::new(),
    }];

    store.add("content_test", tools).await?;

    let result = store.list_all_tools("content_test", None, None).await?;
    let tools_list = parse_rows(&result)?;

    // Content is at top level; shape is { id, content, metadata }
    assert_eq!(
        tools_list[0]["content"],
        "This is the description for embedding"
    );
    Ok(())
}

/// Test that `list_all_tools` handles multiple skills with the same tool name.
#[tokio::test]
async fn test_list_all_tools_multiple_skills_same_tool_name() -> Result<()> {
    let (_temp_dir, store) = create_store("test_multi_skill", 1024).await?;

    // Create tools from different skills with same tool name (e.g., "status")
    let tools = vec![
        ToolRecord {
            tool_name: "status".to_string(),
            description: "Git status command".to_string(),
            skill_name: "git".to_string(),
            file_path: "git/scripts/status.py".to_string(),
            function_name: "status".to_string(),
            execution_mode: "script".to_string(),
            keywords: vec!["git".to_string(), "status".to_string()],
            intents: vec!["Check status".to_string()],
            file_hash: "abc123".to_string(),
            input_schema: r#"{"type": "object"}"#.to_string(),
            docstring: "Show git status".to_string(),
            category: "version_control".to_string(),
            annotations: ToolAnnotations::default(),
            parameters: vec![],
            skill_tools_refers: vec![],
            resource_uri: String::new(),
        },
        ToolRecord {
            tool_name: "status".to_string(),
            description: "Database status command".to_string(),
            skill_name: "database".to_string(),
            file_path: "database/scripts/status.py".to_string(),
            function_name: "status".to_string(),
            execution_mode: "script".to_string(),
            keywords: vec!["db".to_string(), "status".to_string()],
            intents: vec!["Check DB status".to_string()],
            file_hash: "def456".to_string(),
            input_schema: r#"{"type": "object"}"#.to_string(),
            docstring: "Show database status".to_string(),
            category: "database".to_string(),
            annotations: ToolAnnotations::default(),
            parameters: vec![],
            skill_tools_refers: vec![],
            resource_uri: String::new(),
        },
    ];

    store.add("multi_skill", tools).await?;

    let result = store.list_all_tools("multi_skill", None, None).await?;
    let tools_list = parse_rows(&result)?;

    assert_eq!(tools_list.len(), 2);

    // Verify both skills are correctly identified (shape: { id, content, metadata })
    let skill_names: Vec<String> = tools_list
        .iter()
        .map(|t| {
            t.get("metadata")
                .and_then(|m| m.get("skill_name"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        })
        .collect();
    assert!(skill_names.contains(&"git".to_string()));
    assert!(skill_names.contains(&"database".to_string()));
    Ok(())
}

/// Test that `row_limit` caps returned records for `list_all_tools`.
#[tokio::test]
async fn test_list_all_tools_row_limit_caps_results() -> Result<()> {
    let (_temp_dir, store) = create_store("test_row_limit", 1024).await?;

    let tools: Vec<ToolRecord> = (0..5)
        .map(|idx| ToolRecord {
            tool_name: format!("tool_{idx}"),
            description: format!("tool {idx} description"),
            skill_name: "limit_test".to_string(),
            file_path: format!("limit/scripts/tool_{idx}.py"),
            function_name: format!("tool_{idx}"),
            execution_mode: "script".to_string(),
            keywords: vec!["limit".to_string()],
            intents: vec!["limit test".to_string()],
            file_hash: format!("hash_{idx}"),
            input_schema: r#"{"type":"object"}"#.to_string(),
            docstring: format!("tool {idx}"),
            category: "test".to_string(),
            annotations: ToolAnnotations::default(),
            parameters: vec![],
            skill_tools_refers: vec![],
            resource_uri: String::new(),
        })
        .collect();

    store.add("row_limit", tools).await?;

    let result = store.list_all_tools("row_limit", None, Some(2)).await?;
    let tools_list = parse_rows(&result)?;

    assert_eq!(tools_list.len(), 2);
    Ok(())
}

/// Test that `source_filter` supports multi-term union via `a||b`.
#[tokio::test]
async fn test_list_all_tools_source_filter_supports_multi_terms() -> Result<()> {
    let (_temp_dir, store) = create_store("test_source_filter_multi", 1024).await?;

    let tools = vec![
        ToolRecord {
            tool_name: "a".to_string(),
            description: "tool a".to_string(),
            skill_name: "source_filter".to_string(),
            file_path: "docs/a.md".to_string(),
            function_name: "a".to_string(),
            execution_mode: "script".to_string(),
            keywords: vec!["a".to_string()],
            intents: vec!["a".to_string()],
            file_hash: "hash_a".to_string(),
            input_schema: r#"{"type":"object"}"#.to_string(),
            docstring: "a".to_string(),
            category: "test".to_string(),
            annotations: ToolAnnotations::default(),
            parameters: vec![],
            skill_tools_refers: vec![],
            resource_uri: String::new(),
        },
        ToolRecord {
            tool_name: "b".to_string(),
            description: "tool b".to_string(),
            skill_name: "source_filter".to_string(),
            file_path: "docs/b.md".to_string(),
            function_name: "b".to_string(),
            execution_mode: "script".to_string(),
            keywords: vec!["b".to_string()],
            intents: vec!["b".to_string()],
            file_hash: "hash_b".to_string(),
            input_schema: r#"{"type":"object"}"#.to_string(),
            docstring: "b".to_string(),
            category: "test".to_string(),
            annotations: ToolAnnotations::default(),
            parameters: vec![],
            skill_tools_refers: vec![],
            resource_uri: String::new(),
        },
        ToolRecord {
            tool_name: "c".to_string(),
            description: "tool c".to_string(),
            skill_name: "source_filter".to_string(),
            file_path: "docs/c.md".to_string(),
            function_name: "c".to_string(),
            execution_mode: "script".to_string(),
            keywords: vec!["c".to_string()],
            intents: vec!["c".to_string()],
            file_hash: "hash_c".to_string(),
            input_schema: r#"{"type":"object"}"#.to_string(),
            docstring: "c".to_string(),
            category: "test".to_string(),
            annotations: ToolAnnotations::default(),
            parameters: vec![],
            skill_tools_refers: vec![],
            resource_uri: String::new(),
        },
    ];

    store.add("source_filter_multi", tools).await?;

    let result = store
        .list_all_tools("source_filter_multi", Some("a.md||c.md"), None)
        .await?;
    let tools_list = parse_rows(&result)?;

    assert_eq!(tools_list.len(), 2);
    let mut file_paths: Vec<String> = tools_list
        .iter()
        .map(|t| {
            t.get("metadata")
                .and_then(|m| m.get("file_path"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        })
        .collect();
    file_paths.sort();
    assert_eq!(
        file_paths,
        vec!["docs/a.md".to_string(), "docs/c.md".to_string()]
    );
    Ok(())
}

/// Test `source_filter` matching when `file_path` is empty but metadata contains `source`.
#[tokio::test]
async fn test_list_all_tools_source_filter_matches_metadata_when_file_path_empty() -> Result<()> {
    let (_temp_dir, store) = create_store("test_source_filter_metadata_only", 8).await?;

    let ids = vec!["doc-a".to_string(), "doc-b".to_string()];
    let vectors = vec![vec![0.1_f32; 8], vec![0.2_f32; 8]];
    let contents = vec!["content a".to_string(), "content b".to_string()];
    let metadatas = vec![
        serde_json::json!({
            "type": "chunk",
            "source": "docs/a.md",
            "chunk_index": 0,
        })
        .to_string(),
        serde_json::json!({
            "type": "chunk",
            "source": "docs/b.md",
            "chunk_index": 0,
        })
        .to_string(),
    ];

    store
        .add_documents(
            "source_filter_metadata_only",
            ids,
            vectors,
            contents,
            metadatas,
        )
        .await?;

    let result = store
        .list_all_tools("source_filter_metadata_only", Some("docs/a.md"), None)
        .await?;
    let rows = parse_rows(&result)?;

    assert_eq!(rows.len(), 1);
    let source = rows[0]
        .get("metadata")
        .and_then(|m| m.get("source"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(source, "docs/a.md");
    Ok(())
}
