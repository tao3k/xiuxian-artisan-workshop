//! Integration tests for `xiuxian_tui::event`.

use std::time::Duration;

use xiuxian_tui::event::{EventHandlerConfig, TuiEvent};

#[test]
fn test_tui_event_quit() {
    assert!(TuiEvent::Quit.is_quit());
    assert!(!TuiEvent::TogglePanel.is_quit());
}

#[test]
fn test_event_handler_config() {
    let config = EventHandlerConfig::default();
    assert_eq!(config.tick_rate, Duration::from_millis(250));
    assert!(!config.enable_mouse);
}
