//! Integration tests for unified cascading skill structure configuration.

use std::path::{Path, PathBuf};
use tempfile::TempDir;
use xiuxian_skills::SkillStructure;

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("create parent {}: {error}", parent.display()));
    }
    std::fs::write(path, content)
        .unwrap_or_else(|error| panic!("write fixture {}: {error}", path.display()));
}

fn prepare_workspace() -> (TempDir, PathBuf) {
    let temp = TempDir::new().unwrap_or_else(|error| panic!("create temp fixture root: {error}"));
    let root = temp.path().to_path_buf();
    std::fs::create_dir_all(root.join(".config/xiuxian-artisan-workshop")).unwrap_or_else(
        |error| panic!("create .config/xiuxian-artisan-workshop in fixture root: {error}"),
    );
    (temp, root)
}

#[test]
fn skill_structure_prefers_xiuxian_namespace_override() {
    let (_temp, root) = prepare_workspace();

    write_text(
        root.join(".config/xiuxian-artisan-workshop/xiuxian.toml")
            .as_path(),
        r"
[skills.validation]
strict_mode = false
enforce_references_folder = false
prohibit_logic_in_skill_md = false
",
    );

    let structure =
        SkillStructure::load_with_paths(Some(root.as_path()), Some(root.join(".config").as_path()))
            .unwrap_or_else(|error| panic!("expected skill structure to load: {error}"));
    assert!(
        !structure.validation.structure.strict_mode,
        "expected xiuxian [skills.validation] override to disable strict_mode"
    );
    assert!(
        !structure.validation.structure.enforce_references_folder,
        "expected xiuxian [skills.validation] override to disable enforce_references_folder"
    );
}

#[test]
fn skill_structure_ignores_orphan_file_when_orphan_support_is_disabled() {
    let (_temp, root) = prepare_workspace();

    write_text(
        root.join(".config/xiuxian-artisan-workshop/xiuxian.toml")
            .as_path(),
        r"
[skills.validation]
strict_mode = false
",
    );
    write_text(
        root.join(".config/xiuxian-artisan-workshop/ignored.toml")
            .as_path(),
        r"
[validation]
strict_mode = true
",
    );

    let structure =
        SkillStructure::load_with_paths(Some(root.as_path()), Some(root.join(".config").as_path()))
            .unwrap_or_else(|error| panic!("expected skill structure to load: {error}"));
    assert!(
        !structure.validation.structure.strict_mode,
        "expected orphan config to be ignored when orphan_file support is disabled"
    );
}

#[test]
fn skill_structure_ignores_orphan_when_xiuxian_is_absent_and_orphan_support_is_disabled() {
    let (_temp, root) = prepare_workspace();

    write_text(
        root.join(".config/xiuxian-artisan-workshop/ignored.toml")
            .as_path(),
        r"
[validation]
strict_mode = false
enforce_references_folder = false
",
    );

    let structure =
        SkillStructure::load_with_paths(Some(root.as_path()), Some(root.join(".config").as_path()))
            .unwrap_or_else(|error| panic!("expected skill structure to load: {error}"));
    assert!(
        structure.validation.structure.strict_mode,
        "expected embedded strict_mode when orphan_file is disabled and xiuxian.toml is absent"
    );
    assert!(
        structure.validation.structure.enforce_references_folder,
        "expected embedded enforce_references_folder when orphan_file is disabled and xiuxian.toml is absent"
    );
}
