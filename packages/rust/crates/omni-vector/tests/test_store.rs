//! Tests for `VectorStore` - delete operations and core functionality.

use anyhow::Result;
use omni_vector::VectorStore;

#[tokio::test]
async fn test_delete_by_file_path_with_underscores() -> Result<()> {
    // Create a temporary directory for the test database
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_delete");
    let db_path_str = db_path.to_string_lossy();

    // Create vector store (VectorStore::new is async)
    let store = VectorStore::new(db_path_str.as_ref(), Some(1536)).await?;

    // Add a document with a path containing underscores
    let test_id = "test_tool.test_function";
    let test_content = "Test content for delete";
    let test_path = "temp_skill/scripts/hello.py"; // Contains underscore
    let test_metadata = serde_json::json!({
        "skill_name": "test_tool",
        "tool_name": "test_function",
        "file_path": test_path,
        "function_name": "test_function",
        "keywords": ["test"],
        "file_hash": "abc123",
        "input_schema": "{}",
        "docstring": "Test function"
    })
    .to_string();

    // Add the document (use add_documents with single element)
    store
        .add_documents(
            "test_table",
            vec![test_id.to_string()],
            vec![vec![0.1; 1536]],
            vec![test_content.to_string()],
            vec![test_metadata],
        )
        .await?;

    // Verify it's there
    let count_before = store.count("test_table").await?;
    assert_eq!(count_before, 1, "Document should be added");

    // Delete by file path (with underscore)
    store
        .delete_by_file_path("test_table", vec![test_path.to_string()])
        .await?;

    // Verify it's deleted
    let count_after = store.count("test_table").await?;
    assert_eq!(
        count_after, 0,
        "Document should be deleted after calling delete_by_file_path"
    );

    Ok(())
}

#[tokio::test]
async fn test_delete_by_file_path_multiple_paths() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_multi_delete");
    let db_path_str = db_path.to_string_lossy();

    let store = VectorStore::new(db_path_str.as_ref(), Some(1536)).await?;

    // Add multiple documents with different path formats
    let paths_and_ids = vec![
        ("path_with_underscore/file.py", "skill1.func1"),
        ("path/with/slashes/file.py", "skill2.func2"),
        ("path%with%percent/file.py", "skill3.func3"),
    ];

    for (path, id) in &paths_and_ids {
        let metadata = serde_json::json!({
            "file_path": path,
            "skill_name": "test",
            "tool_name": "test",
        })
        .to_string();

        store
            .add_documents(
                "multi_test",
                vec![id.to_string()],
                vec![vec![0.1; 1536]],
                vec!["content".to_string()],
                vec![metadata],
            )
            .await?;
    }

    let count_before = store.count("multi_test").await?;
    assert_eq!(count_before, 3);

    // Delete all paths
    let paths: Vec<String> = paths_and_ids.iter().map(|(p, _)| p.to_string()).collect();
    store.delete_by_file_path("multi_test", paths).await?;

    let count_after = store.count("multi_test").await?;
    assert_eq!(count_after, 0);

    Ok(())
}

#[tokio::test]
async fn test_delete_regression_sql_like_patterns() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_regression");
    let db_path_str = db_path.to_string_lossy();

    let store = VectorStore::new(db_path_str.as_ref(), Some(1536)).await?;

    // These paths contain characters that have special meaning in SQL LIKE
    let problematic_paths = [
        "my_skill/scripts/utils.py",
        "path%with%percent/script.js",
        "dir.with.dots/config.yaml",
    ];

    for (i, path) in problematic_paths.iter().enumerate() {
        let metadata = serde_json::json!({
            "file_path": path,
            "skill_name": format!("skill_{}", i),
            "tool_name": "test_func",
        })
        .to_string();

        store
            .add_documents(
                "regression_test",
                vec![format!("skill_{}.test_func", i)],
                vec![vec![0.1; 1536]],
                vec!["content".to_string()],
                vec![metadata],
            )
            .await?;
    }

    // Delete all problematic paths
    let paths: Vec<String> = problematic_paths
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    store.delete_by_file_path("regression_test", paths).await?;

    // Verify all deleted
    let count = store.count("regression_test").await?;
    assert_eq!(
        count, 0,
        "All paths with SQL-like special chars should be deleted"
    );

    Ok(())
}

/// Robustness: `replace_documents` with empty batch must not drop the table.
#[tokio::test]
async fn test_replace_documents_empty_batch_preserves_table() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_replace_empty");
    let db_path_str = db_path.to_string_lossy();

    let mut store = VectorStore::new(db_path_str.as_ref(), Some(1536)).await?;

    // Add initial data
    store
        .add_documents(
            "skills",
            vec!["id1".to_string()],
            vec![vec![0.1; 1536]],
            vec!["content1".to_string()],
            vec!["{}".to_string()],
        )
        .await?;
    assert_eq!(store.count("skills").await?, 1);

    // replace_documents with empty batch must not drop (robustness: avoid empty table)
    store
        .replace_documents("skills", vec![], vec![], vec![], vec![])
        .await?;

    // Table must still have the original data
    assert_eq!(
        store.count("skills").await?,
        1,
        "replace_documents with empty batch must preserve existing table"
    );

    Ok(())
}

#[tokio::test]
async fn test_replace_documents_rebuilds_table_snapshot() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_replace_docs");
    let db_path_str = db_path.to_string_lossy();

    let mut store = VectorStore::new(db_path_str.as_ref(), Some(1536)).await?;

    store
        .add_documents(
            "skills",
            vec!["id1".to_string(), "id2".to_string()],
            vec![vec![0.1; 1536], vec![0.2; 1536]],
            vec!["content1".to_string(), "content2".to_string()],
            vec!["{}".to_string(), "{}".to_string()],
        )
        .await?;
    assert_eq!(store.count("skills").await?, 2);

    store
        .replace_documents(
            "skills",
            vec!["id3".to_string()],
            vec![vec![0.3; 1536]],
            vec!["content3".to_string()],
            vec!["{}".to_string()],
        )
        .await?;

    // Old snapshot should be fully replaced.
    assert_eq!(store.count("skills").await?, 1);

    Ok(())
}

#[tokio::test]
async fn test_delete_by_metadata_source() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_delete_by_source");
    let db_path_str = db_path.to_string_lossy();

    let store = VectorStore::new(db_path_str.as_ref(), Some(64)).await?;

    let source = "/tmp/docs/2602.12108.pdf";
    let metadatas = vec![
        serde_json::json!({"source": source, "chunk_index": 0, "title": "doc"}).to_string(),
        serde_json::json!({"source": source, "chunk_index": 1, "title": "doc"}).to_string(),
        serde_json::json!({"source": source, "chunk_index": 2, "title": "doc"}).to_string(),
    ];
    store
        .add_documents(
            "knowledge_chunks",
            vec![
                "chunk-0".to_string(),
                "chunk-1".to_string(),
                "chunk-2".to_string(),
            ],
            vec![vec![0.1; 64], vec![0.2; 64], vec![0.3; 64]],
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            metadatas,
        )
        .await?;

    assert_eq!(store.count("knowledge_chunks").await?, 3);

    let deleted = store
        .delete_by_metadata_source("knowledge_chunks", source)
        .await?;
    assert_eq!(deleted, 3);
    assert_eq!(store.count("knowledge_chunks").await?, 0);

    Ok(())
}
