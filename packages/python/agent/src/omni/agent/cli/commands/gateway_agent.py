"""Gateway and agent commands: single-loop stdio and/or HTTP webhook with shared kernel."""

from __future__ import annotations

import os
from typing import Annotated

import typer
from rich.console import Console

from omni.foundation.config.settings import get_setting
from omni.foundation.utils.common import setup_import_paths

setup_import_paths()
console = Console()

# Default session for stdio
STDIO_SESSION_ID = "stdio:default"


def _exec_omni_agent(args: list[str]) -> None:
    """Replace current process with `omni-agent` from PATH."""
    try:
        os.execvp("omni-agent", ["omni-agent", *args])
    except FileNotFoundError as exc:
        console.print(
            "[red]omni-agent not found in PATH.[/red] "
            "Build/install it first (for example: [bold]cargo build -p omni-agent[/bold])."
        )
        raise typer.Exit(1) from exc


async def _webhook_loop(port: int, host: str = "127.0.0.1") -> None:
    """Legacy shim removed. Gateway dispatch is Rust-only."""
    del port, host
    raise RuntimeError(
        "Python webhook loop is decommissioned. "
        "Use Rust gateway: `omni-agent gateway --bind <host:port>`."
    )


async def _stdio_loop(session_id: str) -> None:
    """Legacy shim removed. REPL dispatch is Rust-only."""
    del session_id
    raise RuntimeError(
        "Python stdio loop is decommissioned. Use Rust REPL: `omni-agent repl --session-id <id>`."
    )


def register_gateway_command(parent_app: typer.Typer) -> None:
    """Register `omni gateway`: Rust-only gateway interface."""
    from omni.agent.cli.load_requirements import register_requirements

    register_requirements("gateway", ollama=True, embedding_index=True)

    @parent_app.command()
    def gateway(
        session_id: Annotated[
            str,
            typer.Option("--session", "-s", help="Session ID (default: stdio:default)"),
        ] = STDIO_SESSION_ID,
        webhook_port: Annotated[
            int | None,
            typer.Option(
                "--webhook-port",
                "-w",
                help="Start HTTP webhook on this port (e.g. 8080); POST /message",
            ),
        ] = None,
        webhook_host: Annotated[
            str,
            typer.Option("--webhook-host", help="Bind webhook to this host (default: 127.0.0.1)"),
        ] = "127.0.0.1",
    ):
        """Run Rust gateway (`omni-agent`) in stdio or webhook mode."""
        if webhook_port is not None:
            bind = f"{webhook_host}:{webhook_port}"
            args = ["gateway", "--bind", bind]
        else:
            args = ["stdio", "--session-id", session_id]
        _exec_omni_agent(args)


def register_agent_command(parent_app: typer.Typer) -> None:
    """Register `omni agent`: Rust-only interactive chat interface."""
    from omni.agent.cli.load_requirements import register_requirements

    register_requirements("agent", ollama=True, embedding_index=True)

    @parent_app.command()
    def agent(
        session_id: Annotated[
            str,
            typer.Option("--session", "-s", help="Session ID (default: stdio:default)"),
        ] = STDIO_SESSION_ID,
    ):
        """Interactive chat via Rust `omni-agent repl`."""
        _exec_omni_agent(["repl", "--session-id", session_id])


def register_channel_command(parent_app: typer.Typer) -> None:
    """Register `omni channel`: run Telegram channel (Rust agent only)."""
    from omni.agent.cli.load_requirements import register_requirements

    register_requirements("channel", ollama=True, embedding_index=True)

    @parent_app.command()
    def channel(
        bot_token: Annotated[
            str | None,
            typer.Option(
                "--bot-token", "-t", help="Telegram bot token (or TELEGRAM_BOT_TOKEN env)"
            ),
        ] = None,
    ):
        """Run Telegram channel via Rust `omni-agent channel`."""
        token = bot_token or os.environ.get("TELEGRAM_BOT_TOKEN")
        if not token:
            console.print(
                "[red]Telegram bot token not found.[/red]\n"
                "Set one of:\n"
                "  • [bold]--bot-token[/bold] or [bold]-t[/bold]\n"
                "  • [bold]TELEGRAM_BOT_TOKEN[/bold] env"
            )
            raise typer.Exit(1)
        max_rounds = get_setting("telegram.max_tool_rounds") or 30
        os.environ["OMNI_AGENT_MAX_TOOL_ROUNDS"] = str(int(max_rounds))
        args = ["channel", "--bot-token", token]
        _exec_omni_agent(args)


__all__ = ["register_agent_command", "register_channel_command", "register_gateway_command"]
