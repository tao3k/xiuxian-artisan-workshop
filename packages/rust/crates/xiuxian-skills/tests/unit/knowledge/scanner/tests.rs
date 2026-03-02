use std::fs::File;
use std::io::Write;
use std::io::{self, ErrorKind};

use tempfile::TempDir;

use super::KnowledgeScanner;
use crate::knowledge::types::KnowledgeCategory;

#[test]
fn test_scan_document_with_frontmatter() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let doc_path = temp_dir.path().join("git-commits.md");

    let content = r#"---
title: "Git Commit Best Practices"
description: "Guidelines for writing effective commit messages"
category: "pattern"
tags: ["git", "commit", "best-practices"]
authors: ["developer@example.com"]
version: "1.0.0"
---

# Git Commit Best Practices

This document describes best practices for git commits.
"#;

    let mut file = File::create(&doc_path)?;
    file.write_all(content.as_bytes())?;

    let entry = KnowledgeScanner::scan_document(&doc_path, temp_dir.path())
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "expected scanned entry"))?;

    assert_eq!(entry.title, "Git Commit Best Practices");
    assert_eq!(
        entry.description,
        "Guidelines for writing effective commit messages"
    );
    assert_eq!(entry.category, KnowledgeCategory::Pattern);
    assert_eq!(entry.tags, vec!["git", "commit", "best-practices"]);
    assert!(!entry.file_hash.is_empty());

    Ok(())
}

#[test]
fn test_scan_document_without_frontmatter() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let doc_path = temp_dir.path().join("readme.md");

    let content = r"# README

This is a simple readme without frontmatter.
";

    let mut file = File::create(&doc_path)?;
    file.write_all(content.as_bytes())?;

    let entry = KnowledgeScanner::scan_document(&doc_path, temp_dir.path())
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "expected scanned entry"))?;

    // Title should be derived from filename
    assert_eq!(entry.title, "readme");
    assert_eq!(entry.category, KnowledgeCategory::Unknown);
    assert!(entry.tags.is_empty());

    Ok(())
}

#[test]
fn test_scan_document_non_markdown() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let doc_path = temp_dir.path().join("data.json");

    std::fs::write(&doc_path, r#"{"key": "value"}"#)?;

    let entry = KnowledgeScanner::scan_document(&doc_path, temp_dir.path());

    assert!(entry.is_none());

    Ok(())
}

#[test]
fn test_scan_all() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create multiple docs
    let docs = [
        (
            "doc1.md",
            r#"---
title: "Document 1"
category: "pattern"
---
# Doc 1
"#,
        ),
        (
            "doc2.md",
            r#"---
title: "Document 2"
category: "note"
---
# Doc 2
"#,
        ),
        (
            "sub/doc3.md",
            r#"---
title: "Document 3"
category: "technique"
---
# Doc 3
"#,
        ),
    ];

    for (name, content) in &docs {
        let path = temp_dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
    }

    let scanner = KnowledgeScanner::new();
    let entries = scanner.scan_all(temp_dir.path(), None)?;

    assert_eq!(entries.len(), 3);
    assert!(entries.iter().any(|e| e.title == "Document 1"));
    assert!(entries.iter().any(|e| e.title == "Document 2"));
    assert!(entries.iter().any(|e| e.title == "Document 3"));

    Ok(())
}

#[test]
fn test_scan_category() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    let docs = [
        (
            "pattern1.md",
            r#"---
title: "Pattern 1"
category: "pattern"
---
"#,
        ),
        (
            "pattern2.md",
            r#"---
title: "Pattern 2"
category: "pattern"
---
"#,
        ),
        (
            "note1.md",
            r#"---
title: "Note 1"
category: "note"
---
"#,
        ),
    ];

    for (name, content) in &docs {
        std::fs::write(temp_dir.path().join(name), content)?;
    }

    let scanner = KnowledgeScanner::new();
    let patterns = scanner.scan_category(temp_dir.path(), "pattern")?;

    assert_eq!(patterns.len(), 2);
    assert!(
        patterns
            .iter()
            .all(|e| e.category == KnowledgeCategory::Pattern)
    );

    Ok(())
}

#[test]
fn test_get_tags() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    let docs = [
        (
            "doc1.md",
            r#"---
title: "Doc 1"
tags: ["rust", "programming"]
---
"#,
        ),
        (
            "doc2.md",
            r#"---
title: "Doc 2"
tags: ["rust", "cargo"]
---
"#,
        ),
        (
            "doc3.md",
            r#"---
title: "Doc 3"
tags: ["python"]
---
"#,
        ),
    ];

    for (name, content) in &docs {
        std::fs::write(temp_dir.path().join(name), content)?;
    }

    let scanner = KnowledgeScanner::new();
    let tags = scanner.get_tags(temp_dir.path())?;

    // rust appears 2 times, programming 1, cargo 1, python 1
    assert_eq!(tags.len(), 4);
    // Should be sorted by count descending
    assert_eq!(tags[0].0, "rust");
    assert_eq!(tags[0].1, 2);

    Ok(())
}
