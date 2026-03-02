use serde_json::json;

pub(crate) fn format_slash_help() -> String {
    [
        "## Bot Slash Help".to_string(),
        "Use slash commands in chat to inspect session state, run background jobs, and recover context.".to_string(),
        String::new(),
        "### General".to_string(),
        "- `/help` or `/slash help`: show this command guide.".to_string(),
        "- `/help json`: machine-readable command catalog.".to_string(),
        String::new(),
        "### Session".to_string(),
        "- `/session [json]`: current session window/snapshot status.".to_string(),
        "- `/session budget [json]`: context-budget diagnostics.".to_string(),
        "- `/session memory [json]`: memory recall trigger/result/runtime status.".to_string(),
        "- `/session feedback up|down [json]`: adjust recall feedback bias.".to_string(),
        "- `/session admin [list|set|add|remove|clear] [json]`: delegated admins for current group/topic (admin).".to_string(),
        "- `/session partition|scope [mode|on|off] [json]`: session key mode (admin)."
            .to_string(),
        "- `/feedback up|down [json]`: short alias of `/session feedback ...`.".to_string(),
        "- `/reset` or `/clear`: clear active session context (admin).".to_string(),
        "- `/resume`, `/resume status`, `/resume drop`: restore/check/drop saved snapshot.".to_string(),
        "- `/stop` (or `/cancel`): interrupt current foreground generation for this session.".to_string(),
        String::new(),
        "### Background".to_string(),
        "- `/bg <prompt>`: submit prompt as background job.".to_string(),
        "- `/job <id> [json]`: inspect one background job.".to_string(),
        "- `/jobs [json]`: background queue health summary.".to_string(),
        String::new(),
        "### Notes".to_string(),
        "- Some commands can be blocked by slash ACL or admin policy.".to_string(),
        "- Add `json` when you need script-friendly output.".to_string(),
    ]
    .join("\n")
}

pub(crate) fn format_slash_help_json() -> String {
    json!({
        "kind": "slash_help",
        "commands": {
            "general": [
                {"usage": "/help", "description": "Show slash command guide"},
                {"usage": "/help json", "description": "Machine-readable guide payload"},
                {"usage": "/slash help", "description": "Alias of /help"},
            ],
            "session": [
                {"usage": "/session [json]", "description": "Session window/snapshot status"},
                {"usage": "/session budget [json]", "description": "Context-budget diagnostics"},
                {"usage": "/session memory [json]", "description": "Memory recall trigger/result/runtime status"},
                {"usage": "/session feedback up|down [json]", "description": "Adjust recall feedback bias"},
                {"usage": "/session admin [list|set|add|remove|clear] [json]", "description": "Delegated admins for current group/topic (admin)"},
                {"usage": "/session partition|scope [mode|on|off] [json]", "description": "Session partition mode (admin)"},
                {"usage": "/feedback up|down [json]", "description": "Alias of /session feedback"},
                {"usage": "/reset | /clear", "description": "Reset current session context (admin)"},
                {"usage": "/resume | /resume status | /resume drop", "description": "Restore/check/drop saved context snapshot"},
                {"usage": "/stop | /cancel", "description": "Interrupt current foreground generation for this session"},
            ],
            "background": [
                {"usage": "/bg <prompt>", "description": "Submit background job"},
                {"usage": "/job <id> [json]", "description": "Inspect one background job"},
                {"usage": "/jobs [json]", "description": "Background queue health summary"},
            ],
        },
        "notes": [
            "Some commands can be blocked by slash ACL or admin policy.",
            "Add json for script-friendly responses."
        ],
    })
    .to_string()
}
