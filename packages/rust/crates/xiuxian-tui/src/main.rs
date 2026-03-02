//! main.rs - Production binary entry point for xiuxian-tui
//!
//! Acts as a headless-compatible renderer that visualizes state from Python.
//! Supports two connection modes:
//! - server (legacy): Binds socket and waits for Python to connect
//! - client (reverse): Connects to Python's socket (recommended)
//!
//! Usage:
//!   Server mode: xiuxian-tui --socket /path/to/sock --role server
//!   Client mode: xiuxian-tui --socket /path/to/sock --role client --pid `<parent_pid>`
//!
//! Can also run in headless mode (no TUI rendering) for testing or CI:
//!   xiuxian-tui --socket /path/to/sock --headless

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{self, Event as CEvent, KeyCode};
use log::{info, warn};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use xiuxian_tui::{
    TuiRenderer,
    socket::{SocketClient, SocketEvent, SocketServer},
    state::{AppState, ExecutionState},
};

mod cli_args;

use cli_args::Args;

/// Run the event processing loop (with or without TUI)
fn run_event_loop(state: &mut AppState, server_handle: thread::JoinHandle<()>, headless: bool) {
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Only handle input if not headless
        if !headless {
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());

            if event::poll(timeout).unwrap_or(false)
                && let Ok(CEvent::Key(key)) = event::read()
            {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        info!("User pressed quit");
                        break;
                    }
                    _ => {}
                }
            }
        }

        // Process IPC events (non-blocking drain)
        state.process_ipc_events();

        // Update tick
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    let _ = server_handle.join();
}

fn main() -> Result<()> {
    // Initialize logging
    xiuxian_tui::init_logger();

    let args = Args::parse();

    info!("Starting xiuxian-tui renderer");
    info!("Socket path: {}", args.socket);
    info!("Role: {}", args.role);
    info!("Headless: {}", args.headless);
    if let Some(pid) = args.pid {
        info!("Parent PID: {pid}");
    }

    // Create mpsc channel for IPC bridge
    let (event_tx, event_rx) = mpsc::channel::<SocketEvent>();

    // Start socket connection based on role
    let server_handle = if args.role == "client" {
        // Reverse mode: Connect to Python's socket
        info!("Connecting to Python socket as client...");
        SocketClient::connect(&args.socket, event_tx.clone())
    } else {
        // Legacy mode: Bind and listen
        info!("Binding socket as server...");
        let server = SocketServer::new(&args.socket);
        let tx_clone = event_tx.clone();
        server.set_event_callback(Box::new(move |event: SocketEvent| {
            let _ = tx_clone.send(event);
        }));
        server.start().context("Failed to start socket server")?
    };

    info!("Socket connection established");

    // Create app state with execution state
    let mut state = AppState::new("Omni Agent".to_string());
    state.set_execution_state(ExecutionState::new());
    state.set_event_receiver(event_rx);

    // Try to initialize TUI if not headless
    let renderer = if args.headless {
        info!("Running in headless mode (--headless flag set)");
        None
    } else {
        match TuiRenderer::new() {
            Ok(r) => {
                info!("TUI renderer initialized successfully");
                Some(r)
            }
            Err(e) => {
                warn!("TUI init failed: {e}. Switching to headless mode.");
                None
            }
        }
    };

    // Log the mode we're running in
    if renderer.is_some() {
        info!("Starting TUI rendering loop");
    } else {
        info!("Starting headless event processing loop");
    }

    // Run the event loop
    run_event_loop(
        &mut state,
        server_handle,
        args.headless || renderer.is_none(),
    );

    // Cleanup
    info!("Cleaning up...");

    info!("xiuxian-tui shutdown complete");
    Ok(())
}
