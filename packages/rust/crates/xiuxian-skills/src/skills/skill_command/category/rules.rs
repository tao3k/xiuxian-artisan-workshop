pub(super) struct CategoryRule {
    pub(super) category: &'static str,
    pub(super) keywords: &'static [&'static str],
}

pub(super) const CATEGORY_RULES: &[CategoryRule] = &[
    CategoryRule {
        category: "version_control",
        keywords: &["git", "version", "commit"],
    },
    CategoryRule {
        category: "filesystem",
        keywords: &["file", "fs", "path"],
    },
    CategoryRule {
        category: "engineering",
        keywords: &["code", "engineering", "refactor", "debug"],
    },
    CategoryRule {
        category: "writing",
        keywords: &["writer", "write", "edit", "document"],
    },
    CategoryRule {
        category: "search",
        keywords: &["search", "grep", "query", "find"],
    },
    CategoryRule {
        category: "testing",
        keywords: &["test", "qa", "coverage", "lint"],
    },
    CategoryRule {
        category: "data",
        keywords: &["data", "database", "db", "sql"],
    },
    CategoryRule {
        category: "shell",
        keywords: &["shell", "exec", "run", "command"],
    },
    CategoryRule {
        category: "network",
        keywords: &["api", "http", "network", "web"],
    },
];

pub(super) fn infer_category_keyword_match(skill_name: &str) -> Option<&'static str> {
    CATEGORY_RULES
        .iter()
        .find(|rule| {
            rule.keywords
                .iter()
                .any(|keyword| skill_name.contains(keyword))
        })
        .map(|rule| rule.category)
}
