use super::infer_category_from_skill;

#[test]
fn test_infer_category_git() {
    assert_eq!(infer_category_from_skill("git"), "version_control");
    assert_eq!(infer_category_from_skill("github"), "version_control");
    assert_eq!(
        infer_category_from_skill("version_control"),
        "version_control"
    );
}

#[test]
fn test_infer_category_filesystem() {
    assert_eq!(infer_category_from_skill("filesystem"), "filesystem");
    assert_eq!(infer_category_from_skill("files"), "filesystem");
    assert_eq!(infer_category_from_skill("path_utils"), "filesystem");
}

#[test]
fn test_infer_category_engineering() {
    assert_eq!(infer_category_from_skill("code"), "engineering");
    assert_eq!(infer_category_from_skill("engineering"), "engineering");
    assert_eq!(infer_category_from_skill("refactor"), "engineering");
}

#[test]
fn test_infer_category_writing() {
    assert_eq!(infer_category_from_skill("writer"), "writing");
    assert_eq!(infer_category_from_skill("write"), "writing");
    assert_eq!(infer_category_from_skill("editor"), "writing");
}

#[test]
fn test_infer_category_search() {
    assert_eq!(infer_category_from_skill("search"), "search");
    assert_eq!(infer_category_from_skill("grep"), "search");
    assert_eq!(infer_category_from_skill("find_utils"), "search");
}

#[test]
fn test_infer_category_testing() {
    assert_eq!(infer_category_from_skill("test"), "testing");
    assert_eq!(infer_category_from_skill("qa"), "testing");
    assert_eq!(infer_category_from_skill("lint"), "testing");
}

#[test]
fn test_infer_category_data() {
    assert_eq!(infer_category_from_skill("data"), "data");
    assert_eq!(infer_category_from_skill("database"), "data");
    assert_eq!(infer_category_from_skill("sql"), "data");
}

#[test]
fn test_infer_category_shell() {
    assert_eq!(infer_category_from_skill("shell"), "shell");
    assert_eq!(infer_category_from_skill("exec"), "shell");
    assert_eq!(infer_category_from_skill("runner"), "shell");
}

#[test]
fn test_infer_category_network() {
    assert_eq!(infer_category_from_skill("api"), "network");
    assert_eq!(infer_category_from_skill("http"), "network");
    assert_eq!(infer_category_from_skill("web"), "network");
}

#[test]
fn test_infer_category_unknown() {
    assert_eq!(infer_category_from_skill("unknown_skill"), "unknown_skill");
    assert_eq!(infer_category_from_skill("xyz123"), "xyz123");
}
