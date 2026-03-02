use crate::config::xiuxian::load_xiuxian_config_from_bases;
use std::fs;
use tempfile::tempdir;

type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn test_unified_config_loading_priority() -> TestResult {
    let tmp = tempdir()?;
    let system_base = tmp.path().join("system");
    let user_base = tmp.path().join("user");
    fs::create_dir_all(&system_base)?;
    fs::create_dir_all(&user_base)?;

    fs::write(
        user_base.join("xiuxian.toml"),
        r#"
[wendao.zhixing]
notebook_path = "/custom/unified"
"#,
    )?;

    let config = load_xiuxian_config_from_bases(&system_base, &user_base);
    assert_eq!(
        config.wendao.zhixing.notebook_path.as_deref(),
        Some("/custom/unified")
    );
    Ok(())
}

#[test]
fn test_modular_wendao_fallback() -> TestResult {
    let tmp = tempdir()?;
    let system_base = tmp.path().join("system");
    let user_base = tmp.path().join("user");
    fs::create_dir_all(&system_base)?;
    fs::create_dir_all(&user_base)?;

    fs::write(
        user_base.join("wendao.toml"),
        r#"
[zhixing]
notebook_path = "/modular/fallback"
"#,
    )?;

    let config = load_xiuxian_config_from_bases(&system_base, &user_base);

    assert_eq!(
        config.wendao.zhixing.notebook_path.as_deref(),
        Some("/modular/fallback")
    );
    Ok(())
}
