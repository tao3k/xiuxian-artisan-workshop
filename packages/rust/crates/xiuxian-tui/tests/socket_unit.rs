//! Integration tests for `xiuxian-tui` socket runtime.

use std::fmt::Display;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tempfile::TempDir;
use xiuxian_tui::socket::{SocketEvent, SocketServer, send_event};

fn must_ok<T, E: Display>(result: Result<T, E>, context: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("{context}: {error}"),
    }
}

fn must_some<T>(value: Option<T>, context: &str) -> T {
    match value {
        Some(inner) => inner,
        None => panic!("{context}"),
    }
}

fn lock_or_panic<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(error) => panic!("failed to lock mutex: {error}"),
    }
}

#[test]
fn test_socket_server_start_stop() {
    let temp_dir = must_ok(TempDir::new(), "failed to create temporary directory");
    let socket_path = temp_dir.path().join("test.sock");
    let socket_path_str = must_some(socket_path.to_str(), "socket path is not valid UTF-8");

    let server = SocketServer::new(socket_path_str);
    must_ok(server.start(), "failed to start socket server");

    assert!(server.is_running());
    assert!(Path::new(&socket_path).exists());

    server.stop();
    assert!(!server.is_running());
}

#[test]
fn test_send_and_receive_event() {
    let temp_dir = must_ok(TempDir::new(), "failed to create temporary directory");
    let socket_path = temp_dir.path().join("test.sock");
    let socket_path_str = must_some(socket_path.to_str(), "socket path is not valid UTF-8");

    let received = Arc::new(Mutex::new(Vec::new()));
    let received_clone = Arc::clone(&received);

    let server = SocketServer::new(socket_path_str);
    server.set_event_callback(Box::new(move |event| {
        let mut r = lock_or_panic(&received_clone);
        r.push(event);
    }));

    must_ok(server.start(), "failed to start socket server");

    std::thread::sleep(Duration::from_millis(100));

    let event = SocketEvent {
        source: "test".to_string(),
        topic: "test/event".to_string(),
        payload: serde_json::json!({"message": "hello"}),
        timestamp: "2026-01-31T00:00:00".to_string(),
    };
    must_ok(
        send_event(socket_path_str, &event),
        "failed to send socket event",
    );

    std::thread::sleep(Duration::from_millis(200));
    server.stop();

    let r = lock_or_panic(&received);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].source, "test");
    assert_eq!(r[0].topic, "test/event");
}
