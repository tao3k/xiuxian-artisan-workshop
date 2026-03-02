use crate::channels::telegram::runtime_config::TelegramRuntimeConfig;

pub(super) fn print_foreground_config(
    runtime_config: &TelegramRuntimeConfig,
    session_gate_backend: &str,
) {
    let inbound_queue = runtime_config.inbound_queue_capacity;
    let queue = runtime_config.foreground_queue_capacity;
    let in_flight = runtime_config.foreground_max_in_flight_messages;
    let timeout_secs = runtime_config.foreground_turn_timeout_secs;
    let queue_mode = runtime_config.foreground_queue_mode;
    println!(
        "Foreground config: inbound_queue={inbound_queue} queue={queue} in_flight={in_flight} timeout={timeout_secs}s queue_mode={queue_mode}"
    );
    println!("Session gate backend: {session_gate_backend}");
}

pub(super) fn print_managed_commands_help() {
    println!("Help command: /help [json]");
    println!("Background commands: /bg <prompt>, /job <id> [json], /jobs [json]");
    println!(
        "Session commands: /session [json], /session budget [json], /session memory [json], /session feedback up|down [json], /session admin [list|set|add|remove|clear] [json], /session partition|scope [mode|on|off] [json], /feedback up|down [json], /reset, /clear, /resume, /resume drop, /stop"
    );
}
