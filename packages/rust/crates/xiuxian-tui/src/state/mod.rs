//! Application state management for TUI
//!
//! Provides:
//! - `TaskItem`: Individual task representation
//! - `ExecutionState`: Task graph and execution tracking
//! - `LogWindow`: Bounded rolling log (max 1000 lines)
//! - `AppState`: Main application state with mpsc event receiver

use crate::components::{FoldablePanel, TuiApp};
use crate::socket::{SocketEvent, SocketServer};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// Maximum number of log lines to keep in the rolling window
pub const MAX_LOG_LINES: usize = 1000;

/// Task status enum for visualization
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    #[serde(rename = "pending")]
    /// Task has not started yet.
    Pending,
    #[serde(rename = "running")]
    /// Task is currently executing.
    Running,
    #[serde(rename = "success")]
    /// Task finished successfully.
    Success,
    #[serde(rename = "failed")]
    /// Task failed with an error.
    Failed,
    #[serde(rename = "retry")]
    /// Task is scheduled for a retry.
    Retry,
}

/// Individual task item for the task list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskItem {
    /// Stable task identifier.
    pub id: String,
    /// Human-readable task description.
    pub description: String,
    /// Command associated with this task.
    pub command: String,
    /// Current lifecycle status.
    pub status: TaskStatus,
    /// Execution duration in milliseconds when available.
    pub duration_ms: Option<f64>,
    /// Short output preview for UI rendering.
    pub output_preview: Option<String>,
    /// Number of retries already attempted.
    pub retry_count: usize,
    /// Last known error message, if any.
    pub error: Option<String>,
}

impl TaskItem {
    /// Create a new pending task
    #[must_use]
    pub fn new(id: String, description: String, command: String) -> Self {
        Self {
            id,
            description,
            command,
            status: TaskStatus::Pending,
            duration_ms: None,
            output_preview: None,
            retry_count: 0,
            error: None,
        }
    }

    /// Get status symbol for display
    #[must_use]
    pub fn status_symbol(&self) -> &'static str {
        match self.status {
            TaskStatus::Pending => "○",
            TaskStatus::Running => "◉",
            TaskStatus::Success => "✓",
            TaskStatus::Failed => "✗",
            TaskStatus::Retry => "↻",
        }
    }

    /// Get status color
    #[must_use]
    pub fn status_color(&self) -> Color {
        match self.status {
            TaskStatus::Pending => Color::DarkGray,
            TaskStatus::Running => Color::Yellow,
            TaskStatus::Success => Color::Green,
            TaskStatus::Failed => Color::Red,
            TaskStatus::Retry => Color::Magenta,
        }
    }
}

/// Execution state for tracking task graph execution
#[derive(Debug, Clone, Default)]
pub struct ExecutionState {
    /// All tasks in the execution
    pub tasks: Vec<TaskItem>,
    /// Map from `task_id` to index in tasks vector (O(1) lookup)
    pub task_index: std::collections::HashMap<String, usize>,
    /// Current execution ID
    pub execution_id: Option<String>,
    /// Total tasks count
    pub total_tasks: usize,
    /// Completed tasks count
    pub completed_tasks: usize,
    /// Failed tasks count
    pub failed_tasks: usize,
    /// Current executing group
    pub current_group: Option<String>,
    /// Execution start time
    pub start_time: Option<std::time::Instant>,
    /// Whether execution is complete
    pub is_complete: bool,
}

impl ExecutionState {
    /// Create new execution state
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            task_index: std::collections::HashMap::new(),
            execution_id: None,
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            current_group: None,
            start_time: None,
            is_complete: false,
        }
    }

    /// Clear all state for new execution
    pub fn clear(&mut self) {
        self.tasks.clear();
        self.task_index.clear();
        self.execution_id = None;
        self.total_tasks = 0;
        self.completed_tasks = 0;
        self.failed_tasks = 0;
        self.current_group = None;
        self.start_time = None;
        self.is_complete = false;
    }

    /// Initialize from cortex/start event payload
    pub fn init_from_payload(&mut self, payload: &serde_json::Value) {
        self.clear();

        if let Some(exec_id) = payload.get("execution_id").and_then(|v| v.as_str()) {
            self.execution_id = Some(exec_id.to_string());
        }
        if let Some(total) = payload
            .get("total_tasks")
            .and_then(serde_json::Value::as_u64)
        {
            self.total_tasks = usize::try_from(total).unwrap_or(usize::MAX);
        }

        self.start_time = Some(std::time::Instant::now());

        // Log initialization
        let execution_id = &self.execution_id;
        log::info!("Execution started: {execution_id:?}");
    }

    /// Add a task to the execution state
    pub fn add_task(&mut self, task: TaskItem) {
        let index = self.tasks.len();
        let task_id = task.id.clone();
        self.tasks.push(task);
        self.task_index.insert(task_id, index);
    }

    /// Find task by ID and return mutable reference
    pub fn find_task_mut(&mut self, task_id: &str) -> Option<&mut TaskItem> {
        if let Some(&index) = self.task_index.get(task_id) {
            self.tasks.get_mut(index)
        } else {
            None
        }
    }

    /// Find task by ID (immutable)
    #[must_use]
    pub fn find_task(&self, task_id: &str) -> Option<&TaskItem> {
        if let Some(&index) = self.task_index.get(task_id) {
            self.tasks.get(index)
        } else {
            None
        }
    }

    /// Update task status by ID
    pub fn update_task_status(&mut self, task_id: &str, status: TaskStatus) {
        if let Some(task) = self.find_task_mut(task_id) {
            task.status = status;
            log::info!("Task {task_id} status: {status:?}");
        }
    }

    /// Mark task as complete
    pub fn complete_task(&mut self, task_id: &str, payload: &serde_json::Value) {
        if let Some(task) = self.find_task_mut(task_id) {
            task.status = TaskStatus::Success;
            if let Some(duration) = payload
                .get("duration_ms")
                .and_then(serde_json::Value::as_f64)
            {
                task.duration_ms = Some(duration);
            }
            if let Some(output) = payload.get("output_preview").and_then(|v| v.as_str()) {
                task.output_preview = Some(output.to_string());
            }
            self.completed_tasks += 1;
            log::info!("Task {task_id} completed");
        }
    }

    /// Mark task as failed
    pub fn fail_task(&mut self, task_id: &str, payload: &serde_json::Value) {
        if let Some(task) = self.find_task_mut(task_id) {
            task.status = TaskStatus::Failed;
            if let Some(error) = payload.get("error").and_then(|v| v.as_str()) {
                task.error = Some(error.to_string());
            }
            if let Some(retry) = payload
                .get("retry_count")
                .and_then(serde_json::Value::as_u64)
            {
                task.retry_count = usize::try_from(retry).unwrap_or(usize::MAX);
            }
            self.failed_tasks += 1;
            log::info!("Task {task_id} failed");
        }
    }

    /// Get progress percentage
    #[must_use]
    pub fn progress(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            let completed = u32::try_from(self.completed_tasks).unwrap_or(u32::MAX);
            let failed = u32::try_from(self.failed_tasks).unwrap_or(u32::MAX);
            let total = u32::try_from(self.total_tasks).unwrap_or(u32::MAX);

            f64::from(completed.saturating_add(failed)) / f64::from(total)
        }
    }

    /// Get execution duration so far
    #[must_use]
    pub fn duration_ms(&self) -> Option<f64> {
        self.start_time.map(|start| {
            let elapsed = start.elapsed();
            elapsed.as_secs_f64() * 1000.0
        })
    }
}

/// Bounded rolling log window
#[derive(Debug, Clone, Default)]
pub struct LogWindow {
    /// Bounded deque of log lines
    lines: VecDeque<String>,
    /// Maximum lines to keep
    max_lines: usize,
    /// Current log level filter (None = all)
    level_filter: Option<String>,
}

impl LogWindow {
    /// Create new log window with max lines
    #[must_use]
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
            level_filter: None,
        }
    }

    /// Add a log line
    pub fn add_line(&mut self, level: &str, message: &str, timestamp: &str) {
        // Apply filter if set
        if let Some(ref filter) = self.level_filter
            && level != filter
        {
            return;
        }

        let upper_level = level.to_uppercase();
        let line = format!("[{timestamp}] [{upper_level}] {message}");
        self.lines.push_back(line);

        // Maintain bounded size
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
    }

    /// Add a log line from payload
    pub fn add_from_payload(&mut self, payload: &serde_json::Value) {
        let level = payload
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info");
        let message = payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let timestamp = payload
            .get("timestamp")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        self.add_line(level, message, timestamp);
    }

    /// Get all lines as Vec
    #[must_use]
    pub fn get_lines(&self) -> Vec<&str> {
        self.lines.iter().map(String::as_str).collect()
    }

    /// Get all lines as Strings
    #[must_use]
    pub fn get_lines_owned(&self) -> Vec<String> {
        self.lines.iter().cloned().collect()
    }

    /// Get line count
    #[must_use]
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Check if empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Set level filter
    pub fn set_level_filter(&mut self, level: Option<String>) {
        self.level_filter = level;
    }

    /// Clear all lines
    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

/// Type of panel for categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelType {
    /// Execution result panel
    Result,
    /// Log output panel
    Log,
    /// Progress indicator
    Progress,
    /// Error display
    Error,
    /// Custom panel
    Custom(&'static str),
}

/// Received event from socket (legacy, for backward compat)
#[derive(Debug, Clone)]
pub struct ReceivedEvent {
    /// Producer identifier for the event stream.
    pub source: String,
    /// Event topic/category name.
    pub topic: String,
    /// Structured event payload.
    pub payload: serde_json::Value,
    /// Event timestamp encoded as string.
    pub timestamp: String,
}

/// Main application state
#[derive(Debug)]
pub struct AppState {
    title: String,
    app: Option<TuiApp>,
    should_quit: bool,
    status_message: Option<String>,
    event_tx: Option<mpsc::Sender<String>>,
    event_bus: Option<broadcast::Sender<super::TuiEvent>>,
    socket_server: Option<SocketServer>,
    received_events: Arc<Mutex<Vec<ReceivedEvent>>>,

    /// Execution state for task tracking
    execution_state: Option<ExecutionState>,
    /// Rolling log window
    log_window: LogWindow,
    /// Event receiver for mpsc channel (new IPC bridge)
    event_receiver: Option<mpsc::Receiver<SocketEvent>>,
    /// Processed count for legacy event processing
    processed_count: Arc<Mutex<usize>>,
}

impl AppState {
    /// Create a new application state
    #[must_use]
    pub fn new(title: String) -> Self {
        let title_clone = title.clone();
        Self {
            title,
            app: Some(TuiApp::new(title_clone)),
            should_quit: false,
            status_message: None,
            event_tx: None,
            event_bus: None,
            socket_server: None,
            received_events: Arc::new(Mutex::new(Vec::new())),
            execution_state: None,
            log_window: LogWindow::new(MAX_LOG_LINES),
            event_receiver: None,
            processed_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Create state without app (for testing)
    #[must_use]
    pub fn empty() -> Self {
        Self {
            title: "Omni TUI".to_string(),
            app: None,
            should_quit: false,
            status_message: None,
            event_tx: None,
            event_bus: None,
            socket_server: None,
            received_events: Arc::new(Mutex::new(Vec::new())),
            execution_state: None,
            log_window: LogWindow::new(MAX_LOG_LINES),
            event_receiver: None,
            processed_count: Arc::new(Mutex::new(0)),
        }
    }

    /// Get title
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get status message
    #[must_use]
    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    /// Set status message
    pub fn set_status(&mut self, message: &str) {
        self.status_message = Some(message.to_string());
    }

    /// Get reference to app
    #[must_use]
    pub fn app(&self) -> Option<&TuiApp> {
        self.app.as_ref()
    }

    /// Get mutable reference to app
    pub fn app_mut(&mut self) -> Option<&mut TuiApp> {
        self.app.as_mut()
    }

    /// Set the app
    pub fn set_app(&mut self, app: TuiApp) {
        self.app = Some(app);
    }

    /// Check if should quit
    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Request quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Add a result panel
    pub fn add_result<S: Into<String>, C: Into<String>>(&mut self, title: S, content: C) {
        if let Some(app) = self.app.as_mut() {
            app.add_result(title, content);
        }
    }

    /// Add a panel with specific type
    pub fn add_panel(&mut self, panel: FoldablePanel, _panel_type: PanelType) {
        if let Some(app) = self.app.as_mut() {
            app.add_panel(panel);
        }
    }

    /// Set execution state
    pub fn set_execution_state(&mut self, state: ExecutionState) {
        self.execution_state = Some(state);
    }

    /// Get execution state reference
    #[must_use]
    pub fn execution_state(&self) -> Option<&ExecutionState> {
        self.execution_state.as_ref()
    }

    /// Get execution state mutable reference
    pub fn execution_state_mut(&mut self) -> Option<&mut ExecutionState> {
        self.execution_state.as_mut()
    }

    /// Set event receiver for mpsc channel
    pub fn set_event_receiver(&mut self, receiver: mpsc::Receiver<SocketEvent>) {
        self.event_receiver = Some(receiver);
    }

    /// Get log window reference
    #[must_use]
    pub fn log_window(&self) -> &LogWindow {
        &self.log_window
    }

    /// Get log window mutable reference
    pub fn log_window_mut(&mut self) -> &mut LogWindow {
        &mut self.log_window
    }

    /// Process events from mpsc channel (non-blocking)
    pub fn process_ipc_events(&mut self) {
        // Take the receiver out to avoid borrow issues, then put it back
        let Some(receiver) = self.event_receiver.take() else {
            return;
        };

        // Non-blocking try_iter to drain the queue every frame
        for event in receiver.try_iter() {
            self.reduce(&event);
        }

        // Put the receiver back
        self.event_receiver = Some(receiver);
    }

    /// Reducer: Process `SocketEvent` and update state.
    /// This is the core algorithm for state transitions
    fn reduce(&mut self, event: &SocketEvent) {
        log::debug!(
            "Reducing event: {source} -> {topic}",
            source = event.source,
            topic = event.topic
        );

        if self.reduce_system_event(event)
            || self.reduce_cortex_event(event)
            || self.reduce_task_event(event)
            || self.reduce_log_event(event)
        {
            return;
        }

        log::debug!("Unknown event topic: {topic}", topic = event.topic);
    }

    fn reduce_system_event(&mut self, event: &SocketEvent) -> bool {
        match event.topic.as_str() {
            "system/init" => {
                self.set_status("Connected to omni-agent");
                if let Some(version) = event.payload.get("version") {
                    log::info!("Agent version: {version}");
                }
                true
            }
            "system/ready" => {
                self.set_status("TUI ready");
                true
            }
            _ => false,
        }
    }

    fn reduce_cortex_event(&mut self, event: &SocketEvent) -> bool {
        match event.topic.as_str() {
            "cortex/start" => {
                self.handle_cortex_start(&event.payload);
                true
            }
            "cortex/group/start" => {
                self.handle_cortex_group_start(&event.payload);
                true
            }
            "cortex/group/complete" => {
                self.handle_cortex_group_complete(&event.payload);
                true
            }
            "cortex/complete" => {
                self.handle_cortex_complete(&event.payload);
                true
            }
            "cortex/error" => {
                self.handle_cortex_error(&event.payload);
                true
            }
            _ => false,
        }
    }

    fn handle_cortex_start(&mut self, payload: &serde_json::Value) {
        if let Some(state) = self.execution_state_mut() {
            state.init_from_payload(payload);
        }
        self.set_status("Execution started");
        self.add_result(
            "Execution Started",
            format!(
                "ID: {:?}\nTotal Tasks: {}",
                self.execution_state()
                    .and_then(|state| state.execution_id.as_ref()),
                self.execution_state().map_or(0, |state| state.total_tasks)
            ),
        );
    }

    fn handle_cortex_group_start(&mut self, payload: &serde_json::Value) {
        let group_name = Self::payload_str(payload, "name", "?");
        let task_count = Self::payload_u64(payload, "task_count", 0);

        if let Some(state) = self.execution_state_mut() {
            state.current_group = Some(group_name.to_string());
        }

        self.set_status(&format!("Group: {group_name} ({task_count} tasks)"));
    }

    fn handle_cortex_group_complete(&mut self, payload: &serde_json::Value) {
        let group_name = Self::payload_str(payload, "name", "?");
        let completed = Self::payload_u64(payload, "completed", 0);
        let failed = Self::payload_u64(payload, "failed", 0);

        self.set_status(&format!(
            "Group {group_name} complete: {completed} done, {failed} failed"
        ));
    }

    fn handle_cortex_complete(&mut self, payload: &serde_json::Value) {
        let success = Self::payload_bool(payload, "success", false);
        let duration_ms = Self::payload_f64(payload, "duration_ms", 0.0);

        if let Some(state) = self.execution_state_mut() {
            state.is_complete = true;
        }

        self.set_status(&format!(
            "Execution {status} in {duration_ms:.0}ms",
            status = if success { "succeeded" } else { "failed" },
        ));
    }

    fn handle_cortex_error(&mut self, payload: &serde_json::Value) {
        let error = Self::payload_str(payload, "error", "Unknown");
        self.add_result("Execution Error", format!("Error: {error}"));
        self.set_status("Execution error occurred");
    }

    fn reduce_task_event(&mut self, event: &SocketEvent) -> bool {
        match event.topic.as_str() {
            "task/start" => {
                self.handle_task_start(&event.payload);
                true
            }
            "task/complete" => {
                self.handle_task_complete(&event.payload);
                true
            }
            "task/retry" => {
                self.handle_task_retry(&event.payload);
                true
            }
            "task/fail" => {
                self.handle_task_fail(&event.payload);
                true
            }
            _ => false,
        }
    }

    fn handle_task_start(&mut self, payload: &serde_json::Value) {
        let task_id = Self::payload_str(payload, "task_id", "?");
        let description = Self::payload_str(payload, "description", "");
        let command = Self::payload_str(payload, "command", "");

        if let Some(state) = self.execution_state_mut() {
            state.add_task(TaskItem::new(
                task_id.to_string(),
                description.to_string(),
                command.to_string(),
            ));
            state.update_task_status(task_id, TaskStatus::Running);
        }

        self.set_status(&format!("Task: {description}"));
    }

    fn handle_task_complete(&mut self, payload: &serde_json::Value) {
        let task_id = Self::payload_str(payload, "task_id", "?");
        if let Some(state) = self.execution_state_mut() {
            state.complete_task(task_id, payload);
        }
    }

    fn handle_task_retry(&mut self, payload: &serde_json::Value) {
        let task_id = Self::payload_str(payload, "task_id", "?");
        let attempt = Self::payload_u64(payload, "attempt", 1);
        if let Some(state) = self.execution_state_mut()
            && let Some(task) = state.find_task_mut(task_id)
        {
            task.status = TaskStatus::Retry;
            task.retry_count = usize::try_from(attempt).unwrap_or(usize::MAX);
        }
        self.set_status(&format!("Retry task {task_id} (attempt {attempt})"));
    }

    fn handle_task_fail(&mut self, payload: &serde_json::Value) {
        let task_id = Self::payload_str(payload, "task_id", "?");
        let error = Self::payload_str(payload, "error", "Unknown");
        if let Some(state) = self.execution_state_mut() {
            state.fail_task(task_id, payload);
        }
        self.add_result(format!("Task {task_id} Failed"), format!("Error: {error}"));
    }

    fn reduce_log_event(&mut self, event: &SocketEvent) -> bool {
        match event.topic.as_str() {
            "log" | "system/log" => {
                self.log_window.add_from_payload(&event.payload);
                true
            }
            _ => false,
        }
    }

    fn payload_str<'a>(payload: &'a serde_json::Value, key: &str, default: &'a str) -> &'a str {
        payload
            .get(key)
            .and_then(serde_json::Value::as_str)
            .unwrap_or(default)
    }

    fn payload_u64(payload: &serde_json::Value, key: &str, default: u64) -> u64 {
        payload
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(default)
    }

    fn payload_f64(payload: &serde_json::Value, key: &str, default: f64) -> f64 {
        payload
            .get(key)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(default)
    }

    fn payload_bool(payload: &serde_json::Value, key: &str, default: bool) -> bool {
        payload
            .get(key)
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(default)
    }

    /// Handle tick event (called periodically)
    pub fn on_tick(&mut self) {
        // Process new IPC events
        self.process_ipc_events();

        // Also process legacy events (for backward compatibility)
        let (new_events, new_count) = {
            let received = match self.received_events.lock() {
                Ok(guard) => guard,
                Err(err) => {
                    log::error!("Failed to lock received_events: {err}");
                    return;
                }
            };
            let processed = match self.processed_count.lock() {
                Ok(guard) => *guard,
                Err(err) => {
                    log::error!("Failed to lock processed_count: {err}");
                    return;
                }
            };

            if processed >= received.len() {
                return;
            }

            (
                received.iter().skip(processed).cloned().collect::<Vec<_>>(),
                received.len(),
            )
        };

        for event in new_events {
            self.on_socket_event(&event);
        }

        match self.processed_count.lock() {
            Ok(mut processed) => {
                *processed = new_count;
            }
            Err(err) => {
                log::error!("Failed to update processed_count: {err}");
            }
        }
    }

    /// Handle custom event from xiuxian-event
    pub fn on_custom_event(&mut self, data: &[u8]) {
        self.set_status(&format!(
            "Received custom event: {len} bytes",
            len = data.len()
        ));
    }

    /// Connect to event bus for receiving xiuxian-event
    pub fn connect_event_bus(&mut self, tx: broadcast::Sender<super::TuiEvent>) {
        self.event_bus = Some(tx);
    }

    /// Send event to UI
    pub fn send_event(&self, event: &str) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event.to_string());
        }
    }

    /// Start Unix socket server for receiving Python events
    ///
    /// # Errors
    /// Returns an error if socket startup fails.
    pub fn start_socket_server(
        &mut self,
        socket_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let server = SocketServer::new(socket_path);
        let received_events = self.received_events.clone();

        server.set_event_callback(Box::new(move |event: SocketEvent| {
            let received = ReceivedEvent {
                source: event.source,
                topic: event.topic,
                payload: event.payload,
                timestamp: event.timestamp,
            };
            match received_events.lock() {
                Ok(mut events) => {
                    events.push(received);
                    if events.len() > 100 {
                        events.remove(0);
                    }
                }
                Err(err) => {
                    log::error!("Failed to lock received_events in socket callback: {err}");
                }
            }
        }));

        server.start()?;
        self.socket_server = Some(server);
        self.set_status(&format!("Socket server listening on {socket_path}"));
        Ok(())
    }

    /// Stop the socket server
    pub fn stop_socket_server(&mut self) {
        if let Some(server) = self.socket_server.take() {
            server.stop();
            self.set_status("Socket server stopped");
        }
    }

    /// Get received events
    #[must_use]
    pub fn received_events(&self) -> Vec<ReceivedEvent> {
        match self.received_events.lock() {
            Ok(events) => events.clone(),
            Err(err) => {
                log::error!("Failed to lock received_events: {err}");
                Vec::new()
            }
        }
    }

    /// Check if socket server is running
    #[must_use]
    pub fn is_socket_running(&self) -> bool {
        self.socket_server
            .as_ref()
            .is_some_and(SocketServer::is_running)
    }

    /// Handle socket event - update UI based on topic
    pub fn on_socket_event(&mut self, event: &ReceivedEvent) {
        let message = format!(
            "[{source}] {topic}",
            source = event.source,
            topic = event.topic
        );
        self.set_status(&message);

        if event.topic.starts_with("omega/mission/") {
            let payload_str = serde_json::to_string_pretty(&event.payload).unwrap_or_default();
            let topic = &event.topic;
            self.add_result(format!("Mission: {topic}"), payload_str);
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new("Omni TUI".to_string())
    }
}
