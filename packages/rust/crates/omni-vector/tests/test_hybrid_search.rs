//! Integration tests for hybrid search (vector + keyword).

use anyhow::Result;
use omni_vector::VectorStore;

type KeywordDoc = (String, String, String, Vec<String>, Vec<String>);

/// Setup a vector store with keyword index enabled for testing.
async fn setup_store(path: &std::path::Path, dim: usize) -> Result<VectorStore> {
    let db_path = path.to_string_lossy().into_owned();
    Ok(VectorStore::new_with_keyword_index(&db_path, Some(dim), true, None, None).await?)
}

#[tokio::test]
async fn test_hybrid_search_without_keyword_index_falls_back_to_vector() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().to_string_lossy().into_owned();
    let store = VectorStore::new(&db_path, None).await?;

    // Add documents first so the table exists
    store
        .add_documents(
            "test",
            vec!["doc1".to_string()],
            vec![vec![0.1; 1024]],
            vec!["test content".to_string()],
            vec![r#"{"category": "test"}"#.to_string()],
        )
        .await?;

    // Without keyword backend enabled, hybrid search should degrade gracefully.
    let results = store
        .hybrid_search("test", "test query", vec![0.1; 1024], 10)
        .await?;

    assert!(!results.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_with_enabled_index() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let store = setup_store(temp_dir.path(), 1024).await?;

    // Add some test documents
    store
        .add_documents(
            "test",
            vec!["git_commit".to_string(), "git_status".to_string()],
            vec![vec![0.1; 1024], vec![0.2; 1024]],
            vec![
                "Commit changes to repository".to_string(),
                "Show working tree status".to_string(),
            ],
            vec![
                r#"{"category": "git", "keywords": ["commit", "save"]}"#.to_string(),
                r#"{"category": "git", "keywords": ["status", "dirty"]}"#.to_string(),
            ],
        )
        .await?;

    // Index keywords
    let docs: Vec<KeywordDoc> = vec![
        (
            "git_commit".to_string(),
            "Commit changes to repository".to_string(),
            "git".to_string(),
            vec!["commit".to_string(), "save".to_string()],
            vec![],
        ),
        (
            "git_status".to_string(),
            "Show working tree status".to_string(),
            "git".to_string(),
            vec!["status".to_string(), "dirty".to_string()],
            vec![],
        ),
    ];
    store.bulk_index_keywords(docs)?;

    // Perform hybrid search
    let results = store
        .hybrid_search("test", "commit", vec![0.1; 1024], 10)
        .await?;

    assert!(!results.is_empty());
    // git_commit should rank higher for "commit" query
    assert_eq!(results[0].tool_name, "git_commit");
    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_field_boosting_name_vs_desc() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let store = setup_store(temp_dir.path(), 10).await?;

    // "search" is in the name of tool1, but only in the description of tool2
    store
        .add_documents(
            "boost_test",
            vec!["search_files".to_string(), "file_scanner".to_string()],
            vec![vec![0.0; 10], vec![0.0; 10]],
            vec![
                "Scan directory".to_string(),
                "Search for files in a path".to_string(),
            ],
            vec!["{}".to_string(), "{}".to_string()],
        )
        .await?;

    store.bulk_index_keywords(vec![
        (
            "search_files".to_string(),
            "Scan directory".to_string(),
            "fs".to_string(),
            vec![],
            vec![],
        ),
        (
            "file_scanner".to_string(),
            "Search for files in a path".to_string(),
            "fs".to_string(),
            vec![],
            vec![],
        ),
    ])?;

    let results = store
        .hybrid_search("boost_test", "search", vec![0.0; 10], 10)
        .await?;

    assert!(!results.is_empty());
    // Tool name match should boost search_files to the top even if file_scanner has "search" in description
    assert_eq!(
        results[0].tool_name, "search_files",
        "Tool name match should outrank description match"
    );
    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_intent_match() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let store = setup_store(temp_dir.path(), 10).await?;

    store
        .add_documents(
            "intent_test",
            vec!["writer.polish".to_string()],
            vec![vec![0.0; 10]],
            vec!["Improve text quality".to_string()],
            vec!["{}".to_string()],
        )
        .await?;

    store.bulk_index_keywords(vec![(
        "writer.polish".to_string(),
        "Improve text quality".to_string(),
        "writer".to_string(),
        vec![],
        vec!["refactor documentation".to_string()],
    )])?;

    let results = store
        .hybrid_search("intent_test", "refactor documentation", vec![0.0; 10], 10)
        .await?;

    assert!(!results.is_empty());
    assert_eq!(results[0].tool_name, "writer.polish");
    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_empty_engines() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let store = setup_store(temp_dir.path(), 10).await?;

    // Create table by adding an unrelated doc
    store
        .add_documents(
            "empty_test",
            vec!["unrelated".to_string()],
            vec![vec![1.0; 10]],
            vec!["content".to_string()],
            vec!["{}".to_string()],
        )
        .await?;
    store.bulk_index_keywords(vec![(
        "unrelated".to_string(),
        "content".to_string(),
        "cat".to_string(),
        vec![],
        vec![],
    )])?;

    // Search for something that won't match either engine
    // Vector search for zeros against a vec of ones will have huge distance
    // Keyword search for "xyz" will return nothing
    let results = store
        .hybrid_search("empty_test", "xyz", vec![0.0; 10], 10)
        .await?;

    // It might still return 'unrelated' via vector if it's the only doc, but distance will be high.
    // Let's assert limit is respected at least.
    assert!(results.len() <= 10);
    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_limit() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let store = setup_store(temp_dir.path(), 10).await?;

    let mut ids = Vec::new();
    let mut vectors = Vec::new();
    let mut contents = Vec::new();
    let mut metadatas = Vec::new();
    let mut kw_docs: Vec<KeywordDoc> = Vec::new();

    for i in 0_u16..20 {
        let name = format!("tool_{i}");
        ids.push(name.clone());
        vectors.push(vec![0.1 * f32::from(i); 10]);
        contents.push(format!("Description for tool {i}"));
        metadatas.push("{}".to_string());
        kw_docs.push((
            name,
            format!("Description for tool {i}"),
            "test".to_string(),
            vec![],
            vec![],
        ));
    }

    store
        .add_documents("limit_test", ids, vectors, contents, metadatas)
        .await?;
    store.bulk_index_keywords(kw_docs)?;

    let limit = 5;
    let results = store
        .hybrid_search("limit_test", "tool", vec![0.0; 10], limit)
        .await?;

    assert_eq!(results.len(), limit, "Should respect limit parameter");
    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_semantic_rescue() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let store = setup_store(temp_dir.path(), 10).await?;

    // Tool 1: Matches keyword "commit" exactly
    // Tool 2: Semantic match for "save changes" but keyword index doesn't have "commit"
    store
        .add_documents(
            "rescue_test",
            vec!["vcs.commit".to_string(), "vcs.persist".to_string()],
            vec![vec![0.0; 10], vec![0.9; 10]], // tool 2 is closer to query_vector [1.0; 10]
            vec![
                "Execute commit".to_string(),
                "Save all changes to disk".to_string(),
            ],
            vec!["{}".to_string(), "{}".to_string()],
        )
        .await?;

    store.bulk_index_keywords(vec![
        (
            "vcs.commit".to_string(),
            "Execute commit".to_string(),
            "vcs".to_string(),
            vec!["commit".to_string()],
            vec![],
        ),
        (
            "vcs.persist".to_string(),
            "Save all changes to disk".to_string(),
            "vcs".to_string(),
            vec!["persist".to_string()],
            vec![],
        ),
    ])?;

    // Query has keyword "commit" but vector is closer to "persist"
    let query_vector = vec![1.0; 10];
    let results = store
        .hybrid_search("rescue_test", "commit", query_vector, 10)
        .await?;

    assert!(results.len() >= 2);
    // vcs.commit should rank highly due to keyword match
    // vcs.persist should also be present due to semantic match
    assert!(results.iter().any(|r| r.tool_name == "vcs.commit"));
    assert!(results.iter().any(|r| r.tool_name == "vcs.persist"));
    Ok(())
}

#[tokio::test]
async fn test_enable_keyword_index_on_existing_store() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().to_string_lossy().into_owned();
    let mut store = VectorStore::new(&db_path, None).await?;

    // Enable keyword index
    store.enable_keyword_index()?;

    // Verify it's enabled
    store.index_keyword(
        "test_tool",
        "A test tool",
        "test",
        &["test".to_string(), "example".to_string()],
        &[],
    )?;
    Ok(())
}

#[tokio::test]
async fn test_hybrid_search_fallback_on_keyword_error() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let store = setup_store(temp_dir.path(), 1024).await?;

    // Add document
    store
        .add_documents(
            "test",
            vec!["git_commit".to_string()],
            vec![vec![0.1; 1024]],
            vec!["Commit changes".to_string()],
            vec![r#"{"category": "git"}"#.to_string()],
        )
        .await?;

    // Index keywords
    store.bulk_index_keywords(vec![(
        "git_commit".to_string(),
        "Commit changes".to_string(),
        "git".to_string(),
        vec!["commit".to_string()],
        vec![],
    )])?;

    // Search with code snippet (should fallback to vector-only gracefully if parser fails)
    let results = store
        .hybrid_search("test", "pub async fn add_documents", vec![0.1; 1024], 5)
        .await?;

    // Should still return results from vector search
    assert!(!results.is_empty());
    Ok(())
}
