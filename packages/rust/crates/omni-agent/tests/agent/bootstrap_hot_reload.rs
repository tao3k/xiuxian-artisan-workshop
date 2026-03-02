use omni_agent::{Agent, AgentConfig};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

const CHILD_ENV_KEY: &str = "OMNI_AGENT_BOOTSTRAP_HOT_RELOAD_CHILD";
const HOT_RELOAD_TEST_NAME: &str =
    "agent::bootstrap_hot_reload::service_mount_records_include_hot_reload_targets";

fn write_seed_notebook(
    notebook_root: &Path,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let journal_dir = notebook_root.join("journal");
    let agenda_dir = notebook_root.join("agenda");
    fs::create_dir_all(&journal_dir)?;
    fs::create_dir_all(&agenda_dir)?;
    fs::write(
        journal_dir.join("2026-02-26.md"),
        "## Reflection\nBootstrap hot-reload mount smoke.\n",
    )?;
    fs::write(
        agenda_dir.join("2026-02-26.md"),
        "- [ ] Validate mount records <!-- id: mount-1, journal:carryover: 0 -->\n",
    )?;
    Ok(())
}

fn run_child_probe(notebook_root: &Path) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let test_binary = env::current_exe()?;
    let output = Command::new(test_binary)
        .arg("--exact")
        .arg(HOT_RELOAD_TEST_NAME)
        .arg("--nocapture")
        .env(CHILD_ENV_KEY, "1")
        .env("XIUXIAN_HOT_RELOAD_ENABLED", "1")
        .env(
            "XIUXIAN_WENDAO_NOTEBOOK_PATH",
            notebook_root.to_string_lossy().as_ref(),
        )
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "hot-reload child probe failed exit_code={:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ))
        .into());
    }
    Ok(())
}

#[tokio::test]
async fn service_mount_records_include_hot_reload_targets()
-> std::result::Result<(), Box<dyn std::error::Error>> {
    if env::var(CHILD_ENV_KEY).ok().as_deref() != Some("1") {
        let notebook_tmp = tempdir()?;
        let notebook_root = notebook_tmp.path().join("notebook");
        write_seed_notebook(&notebook_root)?;
        run_child_probe(&notebook_root)?;
        return Ok(());
    }

    let config = AgentConfig {
        inference_url: "http://127.0.0.1:4000/v1/chat/completions".to_string(),
        memory: None,
        mcp_servers: Vec::new(),
        ..AgentConfig::default()
    };

    let agent = Agent::from_config(config).await?;
    let mounts = agent.service_mount_records().await;

    let Some(qianhuan_target) = mounts
        .iter()
        .find(|record| record.service == "hot_reload.target.qianhuan.manifestation")
    else {
        return Err(std::io::Error::other("missing qianhuan hot-reload mount record").into());
    };
    assert!(
        qianhuan_target
            .detail
            .as_deref()
            .is_some_and(|detail| detail.contains("id=xiuxian_qianhuan.manifestation.templates")),
        "qianhuan hot-reload mount should include target id detail"
    );

    let Some(wendao_target) = mounts
        .iter()
        .find(|record| record.service == "hot_reload.target.wendao.index")
    else {
        return Err(std::io::Error::other("missing wendao hot-reload mount record").into());
    };
    assert!(
        wendao_target
            .detail
            .as_deref()
            .is_some_and(|detail| detail.contains("mode=heyi_sync_incremental_or_full")),
        "wendao hot-reload mount should declare sync mode"
    );

    let Some(driver) = mounts
        .iter()
        .find(|record| record.service == "hot_reload.driver")
    else {
        return Err(std::io::Error::other("missing hot reload driver mount record").into());
    };
    let detail = driver.detail.as_deref().unwrap_or_default();
    assert!(
        detail.contains(
            "targets=xiuxian_qianhuan.manifestation.templates,xiuxian_wendao.link_graph.index"
        ),
        "hot reload driver detail should include both target ids, got: {detail}"
    );

    Ok(())
}
