//! Test coverage for omni-agent behavior.

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;

#[path = "upload_api/media_group.rs"]
mod media_group;
#[path = "upload_api/photo.rs"]
mod photo;
#[path = "upload_api/server_bootstrap.rs"]
mod server_bootstrap;

use server_bootstrap::{spawn_upload_media_group_server, spawn_upload_photo_server};

#[derive(Clone, Debug)]
pub struct UploadCall {
    pub method: String,
    pub field_names: Vec<String>,
    pub text_fields: serde_json::Map<String, serde_json::Value>,
    pub media_json: Option<serde_json::Value>,
}

#[derive(Clone, Default)]
pub struct MockTelegramUploadState {
    pub calls: Arc<Mutex<Vec<UploadCall>>>,
    pub field_names: Arc<Mutex<Vec<String>>>,
    pub text_fields: Arc<Mutex<serde_json::Map<String, serde_json::Value>>>,
    pub media_json: Arc<Mutex<Option<serde_json::Value>>>,
    pub first_markdown_error: Arc<Mutex<Option<String>>>,
}

impl MockTelegramUploadState {
    fn with_markdown_error(first_markdown_error: Option<&str>) -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
            field_names: Arc::new(Mutex::new(Vec::new())),
            text_fields: Arc::new(Mutex::new(serde_json::Map::new())),
            media_json: Arc::new(Mutex::new(None)),
            first_markdown_error: Arc::new(Mutex::new(
                first_markdown_error.map(std::string::ToString::to_string),
            )),
        }
    }
}

pub async fn spawn_mock_telegram_upload_api()
-> Result<Option<(String, MockTelegramUploadState, tokio::task::JoinHandle<()>)>> {
    spawn_mock_telegram_upload_api_with_markdown_error(None).await
}

pub async fn spawn_mock_telegram_upload_api_with_markdown_error(
    first_markdown_error: Option<&str>,
) -> Result<Option<(String, MockTelegramUploadState, tokio::task::JoinHandle<()>)>> {
    let state = MockTelegramUploadState::with_markdown_error(first_markdown_error);
    spawn_upload_photo_server(state).await
}

pub async fn spawn_mock_telegram_media_group_upload_api()
-> Result<Option<(String, MockTelegramUploadState, tokio::task::JoinHandle<()>)>> {
    let state = MockTelegramUploadState::default();
    spawn_upload_media_group_server(state).await
}

fn lint_symbol_probe() {
    let _ = spawn_mock_telegram_upload_api;
    let _ = spawn_mock_telegram_upload_api_with_markdown_error;
    let _ = spawn_mock_telegram_media_group_upload_api;
    let _ = media_group::handle_upload_media_group;
    let _ = photo::handle_upload_photo;
    let _ = server_bootstrap::spawn_upload_photo_server;
    let _ = server_bootstrap::spawn_upload_media_group_server;

    let upload_call = UploadCall {
        method: String::new(),
        field_names: Vec::new(),
        text_fields: serde_json::Map::new(),
        media_json: None,
    };
    let _ = (
        &upload_call.method,
        &upload_call.field_names,
        &upload_call.text_fields,
        &upload_call.media_json,
    );

    let state = MockTelegramUploadState::default();
    let _ = (
        &state.calls,
        &state.field_names,
        &state.text_fields,
        &state.media_json,
        &state.first_markdown_error,
    );
}

const _: fn() = lint_symbol_probe;
