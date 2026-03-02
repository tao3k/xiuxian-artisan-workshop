//! Robustness tests for skill indexing: empty scan must not drop existing table.
//!
//! Prevents regression where `index_skill_tools` / `index_skill_tools_dual` would drop
//! the table before scanning; if scan returned empty, we'd end up with empty table.

use anyhow::Result;
use omni_vector::VectorStore;
use std::path::PathBuf;

fn create_temp_db() -> Result<(tempfile::TempDir, PathBuf)> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test_db");
    Ok((temp_dir, db_path))
}

async fn add_test_tool(
    store: &VectorStore,
    id: &str,
    skill_name: &str,
    tool_name: &str,
) -> Result<()> {
    let metadata = serde_json::json!({
        "type": "command",
        "skill_name": skill_name,
        "tool_name": tool_name,
        "command": tool_name,
        "file_path": format!("{}/scripts/test.py", skill_name),
        "function_name": tool_name,
        "keywords": [skill_name, tool_name],
        "intents": [],
        "file_hash": "test_hash_123",
        "input_schema": "{}",
        "docstring": "Test tool"
    })
    .to_string();

    store
        .add_documents(
            "skills",
            vec![id.to_string()],
            vec![vec![0.1; 1024]],
            vec![format!("Test tool {}", tool_name)],
            vec![metadata],
        )
        .await?;
    Ok(())
}

/// `index_skill_tools_dual` with empty scan (path has no skills) must preserve existing table.
#[tokio::test]
async fn test_index_skill_tools_dual_empty_scan_preserves_table() -> Result<()> {
    let (temp_dir, db_path) = create_temp_db()?;
    let empty_skills_dir = temp_dir.path().join("empty_skills");
    std::fs::create_dir_all(&empty_skills_dir)?;
    let db_path_str = db_path.to_string_lossy().into_owned();
    let empty_skills_dir_str = empty_skills_dir.to_string_lossy().into_owned();

    let mut store =
        VectorStore::new_with_keyword_index(db_path_str.as_str(), Some(1024), true, None, None)
            .await?;

    add_test_tool(&store, "git.commit", "git", "commit").await?;
    assert_eq!(store.count("skills").await?, 1);

    // Scan returns empty (empty dir has no SKILL.md); must NOT drop
    let (skills_count, router_count) = store
        .index_skill_tools_dual(empty_skills_dir_str.as_str(), "skills", "skills")
        .await?;

    assert_eq!(skills_count, 0, "scan returned 0 tools");
    assert_eq!(router_count, 0, "scan returned 0 tools");
    assert_eq!(
        store.count("skills").await?,
        1,
        "index_skill_tools_dual with empty scan must preserve existing table"
    );
    Ok(())
}

/// `index_skill_tools` with empty scan (non-existent path) must preserve existing table.
#[tokio::test]
async fn test_index_skill_tools_empty_scan_preserves_table() -> Result<()> {
    let (temp_dir, db_path) = create_temp_db()?;
    let non_existent_path = temp_dir.path().join("does_not_exist");
    let db_path_str = db_path.to_string_lossy().into_owned();
    let non_existent_path_str = non_existent_path.to_string_lossy().into_owned();

    let mut store =
        VectorStore::new_with_keyword_index(db_path_str.as_str(), Some(1024), true, None, None)
            .await?;

    add_test_tool(&store, "git.commit", "git", "commit").await?;
    assert_eq!(store.count("skills").await?, 1);

    // Path does not exist -> scan returns empty; must NOT drop
    store
        .index_skill_tools(non_existent_path_str.as_str(), "skills")
        .await?;

    assert_eq!(
        store.count("skills").await?,
        1,
        "index_skill_tools with non-existent path must preserve existing table"
    );
    Ok(())
}
