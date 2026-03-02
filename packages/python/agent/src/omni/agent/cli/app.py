"""
Omni Agent CLI Entry Point (Refactored)

Responsibilities:
1. Bootstrap Environment: Parse --conf and set PRJ_CONFIG_HOME.
2. Initialize Infrastructure: Logging, Settings.
3. Dispatch Commands.
"""

from __future__ import annotations

import importlib
import os
import subprocess
import sys
from contextlib import suppress
from pathlib import Path
from typing import Any

import typer

from omni.agent.cli.load_requirements import register_requirements
from omni.agent.runtime.decommission import assert_rust_runtime_or_raise
from omni.foundation.config.dirs import PRJ_DIRS, PRJ_RUNTIME
from omni.foundation.config.logging import configure_logging
from omni.foundation.config.settings import get_settings
from omni.foundation.runtime.gitops import get_project_root

app = typer.Typer(
    name="omni-agent",
    help="Omni Dev Fusion Agent CLI - The Neural Nexus",
    no_args_is_help=True,
    add_completion=False,
    invoke_without_command=True,
)

# Global verbose flag (set by entry_point before any command runs)
_verbose_flag: bool = False

# Lazily register command groups to keep startup for `omni skill run` minimal.
_COMMAND_REGISTRY: dict[str, tuple[str, str]] = {
    "completions": (
        "omni.agent.cli.commands.completions",
        "register_completions_command",
    ),
    "dashboard": (
        "omni.agent.cli.commands.dashboard",
        "register_dashboard_command",
    ),
    "db": (
        "omni.agent.cli.commands.db",
        "register_db_command",
    ),
    "skill": (
        "omni.agent.cli.commands.skill",
        "register_skill_command",
    ),
    "mcp": (
        "omni.agent.cli.commands.mcp",
        "register_mcp_command",
    ),
    "route": (
        "omni.agent.cli.commands.route",
        "register_route_command",
    ),
    "run": (
        "omni.agent.cli.commands.run",
        "register_run_command",
    ),
    "gateway": (
        "omni.agent.cli.commands.gateway_agent",
        "register_gateway_command",
    ),
    "agent": (
        "omni.agent.cli.commands.gateway_agent",
        "register_agent_command",
    ),
    "channel": (
        "omni.agent.cli.commands.gateway_agent",
        "register_channel_command",
    ),
    "sync": (
        "omni.agent.cli.commands.sync",
        "register_sync_command",
    ),
    "knowledge": (
        "omni.agent.cli.commands.knowledge",
        "register_knowledge_command",
    ),
    "reindex": (
        "omni.agent.cli.commands.reindex",
        "register_reindex_command",
    ),
}
_REGISTERED_COMMANDS: set[str] = set()
_SKILL_EMBED_OVERRIDE_INSTALLED = False

# Declarative bootstrap requirements for local commands.
register_requirements("version", ollama=False, embedding_index=False)


def _get_git_commit() -> str:
    """Get the current git commit hash."""
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            cwd=get_project_root(),
        )
        return result.stdout.strip()[:8] if result.returncode == 0 else "unknown"
    except Exception:
        return "unknown"


def _get_rust_version() -> str:
    """Get Rust version."""
    try:
        result = subprocess.run(
            ["rustc", "--version"],
            capture_output=True,
            text=True,
        )
        return result.stdout.strip() if result.returncode == 0 else "unknown"
    except Exception:
        return "unknown"


def _get_package_version(package_name: str) -> str:
    """Get installed package version using importlib.metadata."""
    try:
        from importlib.metadata import version

        return version(package_name)
    except Exception:
        return "not installed"


@app.command()
def version():
    """
    Display version information and debug details.
    """
    from omni import __version__

    git_commit = _get_git_commit()
    rust_version = _get_rust_version()
    python_version = f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}"

    # Key dependencies
    omni_core_version = _get_package_version("omni-core")
    omni_mcp_version = _get_package_version("omni-mcp")
    lance_version = _get_package_version("lance")

    typer.echo("=" * 60)
    typer.echo("Omni Dev Fusion - Version Info")
    typer.echo("=" * 60)
    typer.echo(f"  Omni Agent:      {__version__}")
    typer.echo(f"  Git Commit:      {git_commit}")
    typer.echo(f"  Python:          {python_version}")
    typer.echo(f"  Rust:            {rust_version}")
    typer.echo("-" * 60)
    typer.echo("  Dependencies:")
    typer.echo(f"    omni-core:     {omni_core_version}")
    typer.echo(f"    omni-mcp:      {omni_mcp_version}")
    typer.echo(f"    lance:         {lance_version}")
    typer.echo("=" * 60)

    # Debug info
    typer.secho("\nDebug Info:", bold=True)
    typer.echo(f"  Executable: {sys.executable}")
    typer.echo(f"  Path: {Path(sys.executable).parent}")
    typer.echo(f"  Platform: {sys.platform}")

    # Settings location
    settings = get_settings()
    typer.echo(f"\nConfig Location: {getattr(settings, '_config_dir', 'default')}")
    typer.echo(f"Cache Location: {PRJ_RUNTIME('agent')}")
    typer.echo("")


def _bootstrap_configuration(
    conf_path: str | None,
    verbose: bool = False,
) -> None:
    """
    Core Bootstrap Logic.

    Strategy:
    If user provides --conf, we treat that directory as the new PRJ_CONFIG_HOME.
    This effectively "mounts" the user's config directory into the system's
    standard location via environment variables.
    """
    from pathlib import Path

    if conf_path:
        path_obj = Path(conf_path).resolve()
        if not path_obj.exists():
            typer.secho(f"Warning: Config directory not found: {path_obj}", fg=typer.colors.YELLOW)

        # 1. Set the global environment pointer
        os.environ["PRJ_CONFIG_HOME"] = str(path_obj)

        # 2. Clear Directory Cache (Crucial!)
        PRJ_DIRS.clear_cache()

        # 3. Reload Settings
        get_settings().reload()

        typer.secho(f"Configuration loaded from: {path_obj}", fg=typer.colors.BRIGHT_BLACK)

    # Configure Logging (always run)
    settings = get_settings()
    # Auto-derive override mode from provider when user did not explicitly pin it.
    if not os.environ.get("OMNI_EMBED_OVERRIDE_ENABLED", "").strip():
        provider = str(settings.get("embedding.provider", "")).strip().lower()
        os.environ["OMNI_EMBED_OVERRIDE_ENABLED"] = "1" if provider in {"", "client"} else "0"

    log_level = settings.get("logging.level", "INFO")
    if verbose:
        log_level = "DEBUG"

    # Force reconfigure so foundation verbosity state always matches CLI -v.
    # Without force, early logger init can leave is_verbose() stale (INFO) and
    # skip monitor output in skill runner.
    configure_logging(level=log_level, verbose=verbose, force=True)

    # Cross-layer verbose signal for shared/common code (core/foundation).
    os.environ["OMNI_CLI_VERBOSE"] = "1" if verbose else "0"

    # Store verbose flag globally for subcommands to check
    global _verbose_flag
    _verbose_flag = verbose

    # Ensure Runtime Directories Exist
    try:
        PRJ_RUNTIME.ensure_dir("logs")
        PRJ_RUNTIME.ensure_dir("sockets")
        PRJ_RUNTIME.ensure_dir("pids")
    except Exception as e:
        if verbose:
            typer.secho(f"Warning: Could not create runtime dirs: {e}", fg=typer.colors.YELLOW)


def _is_verbose() -> bool:
    """Check if verbose mode is enabled (checks global flag)."""
    return _verbose_flag


def _embedding_override_enabled() -> bool:
    """Return True when CLI skill embedding override should be installed."""
    raw = os.environ.get("OMNI_EMBED_OVERRIDE_ENABLED", "").strip().lower()
    if raw in {"1", "true", "yes", "on"}:
        return True
    if raw in {"0", "false", "no", "off"}:
        return False
    try:
        provider = str(get_settings().get("embedding.provider", "")).strip().lower()
    except Exception:
        return True
    # Override is primarily useful for client/auto provider modes.
    return provider in {"", "client"}


def _verbose_callback(ctx: typer.Context, param: Any, value: bool) -> None:
    """Callback to set verbose flag when --verbose/-v is used."""
    global _verbose_flag
    if value:
        _verbose_flag = True
        os.environ["OMNI_CLI_VERBOSE"] = "1"
        # Reconfigure logging immediately if already configured
        with suppress(Exception):
            configure_logging(level="DEBUG", verbose=True, force=True)


@app.callback()
def main(
    ctx: typer.Context,
    conf: str | None = typer.Option(
        None,
        "--conf",
        "-c",
        help="Path to custom configuration directory (Sets PRJ_CONFIG_HOME)",
        envvar="OMNI_CONF",
    ),
    verbose: bool = typer.Option(
        False,
        "--verbose",
        "-v",
        help="Enable debug logging",
        is_eager=True,
        callback=_verbose_callback,
    ),
):
    """
    Initialize the Agent Environment.

    Global Options:
        --conf, -c     Custom configuration directory
        --verbose, -v  Enable debug logging
    """
    # Bootstrap is handled in entry_point() for proper parameter passing
    pass


def _extract_top_level_command(argv: list[str]) -> str | None:
    """Return the first non-option token from argv (top-level command)."""
    for token in argv:
        if token.startswith("-"):
            continue
        return token.strip().lower()
    return None


def _ensure_skill_embedding_override_installed() -> None:
    """Install skill embedding override once (for skill execution paths)."""
    global _SKILL_EMBED_OVERRIDE_INSTALLED
    if _SKILL_EMBED_OVERRIDE_INSTALLED:
        return
    from omni.agent.embedding_override import install_skill_embedding_override

    install_skill_embedding_override()
    _SKILL_EMBED_OVERRIDE_INSTALLED = True


def _register_command_group(command: str) -> None:
    """Register one top-level command group lazily."""
    command = (command or "").strip().lower()
    if not command or command in _REGISTERED_COMMANDS:
        return
    spec = _COMMAND_REGISTRY.get(command)
    if not spec:
        return

    module_path, register_name = spec
    module = importlib.import_module(module_path)
    register_fn = getattr(module, register_name)
    register_fn(app)
    _REGISTERED_COMMANDS.add(command)

    if command == "skill" and _embedding_override_enabled():
        _ensure_skill_embedding_override_installed()


def _register_commands_for(command: str | None) -> None:
    """Register commands needed for current invocation.

    - known command: register only that command group
    - no command / unknown command: register all groups (help and fallback behavior)
    """
    normalized = (command or "").strip().lower()
    if normalized == "version":
        return
    if normalized in _COMMAND_REGISTRY:
        _register_command_group(normalized)
        return

    for name in _COMMAND_REGISTRY:
        _register_command_group(name)


def _try_fast_skill_run(argv: list[str]) -> bool:
    """Fast path for `omni skill run ...` without Typer command graph setup."""
    if len(argv) < 2:
        return False
    if argv[0] != "skill" or argv[1] != "run":
        return False
    # Keep Typer's native help/usage rendering for `omni skill run --help`.
    if any(token in {"--help", "-h"} for token in argv[2:]):
        return False

    json_output = False
    # Default to daemon reuse for lower repeated latency.
    reuse_process = True
    commands: list[str] = []
    i = 2
    while i < len(argv):
        token = argv[i]
        if token in {"--json", "-j"}:
            json_output = True
            i += 1
            continue
        if token == "--reuse-process":
            reuse_process = True
            i += 1
            continue
        if token == "--no-reuse-process":
            reuse_process = False
            i += 1
            continue
        commands.append(token)
        i += 1

    # Let Typer render proper usage/help when no command is supplied.
    if not commands:
        return False

    if _embedding_override_enabled():
        _ensure_skill_embedding_override_installed()

    if json_output:
        from omni.agent.cli.runner_json import run_skills_json

        exit_code = int(run_skills_json(commands, reuse_process=reuse_process))
        if exit_code != 0:
            raise SystemExit(exit_code)
        return True

    from omni.agent.cli.console import cli_log_handler
    from omni.agent.cli.runner import run_skills

    run_skills(
        commands,
        json_output=False,
        log_handler=cli_log_handler,
        reuse_process=reuse_process,
    )
    return True


def entry_point():
    """Entry point for CLI (used by pyproject.toml entry_points).

    Pre-parses global options (--verbose, -v, --conf, -c) before Typer takes over,
    ensuring logging is configured BEFORE any command runs.
    """
    import os
    import sys

    # Disable tqdm progress bars (LanceDB/lance) for cleaner MCP/CLI output
    os.environ.setdefault("TQDM_DISABLE", "1")

    # Pre-parse to detect --verbose and --conf before Typer takes over
    conf = None
    verbose = False
    argv = sys.argv[1:] if len(sys.argv) > 1 else ["--help"]

    i = 0
    while i < len(argv):
        arg = argv[i]
        if arg in ("--verbose", "-v"):
            verbose = True
            argv.pop(i)
            continue
        elif arg in ("--conf", "-c") and i + 1 < len(argv):
            conf = argv[i + 1]
            argv.pop(i + 1)
            argv.pop(i)
            continue
        elif arg.startswith("--conf="):
            conf = arg.split("=", 1)[1]
            argv.pop(i)
            continue
        i += 1

    # Bootstrap configuration (logging) BEFORE any command runs
    _bootstrap_configuration(conf, verbose)
    assert_rust_runtime_or_raise("omni.cli.entry_point")

    top_command = _extract_top_level_command(argv)

    # Fast path for tool invocation: bypass Typer command graph initialization.
    if _try_fast_skill_run(argv):
        return

    # Lazily register only the command group needed for this invocation.
    _register_commands_for(top_command)

    # Declarative on-demand loading: each command registers its requirements via register_requirements.
    # Skip expensive startup services for help/no-command/unknown-command paths.
    from omni.agent.cli.load_requirements import get_requirements

    should_load_bootstrap_services = top_command is not None and top_command in (
        set(_COMMAND_REGISTRY) | {"version"}
    )
    if should_load_bootstrap_services:
        reqs = get_requirements(top_command)
        if reqs.ollama:
            try:
                from omni.agent.ollama_lifecycle import ensure_ollama_for_embedding

                ensure_ollama_for_embedding()
            except Exception:
                pass
        if reqs.embedding_index:
            try:
                from omni.agent.services.reindex import ensure_embedding_index_compatibility

                ensure_embedding_index_compatibility(auto_fix=True)
            except Exception:
                pass

    # Restore argv and invoke app
    sys.argv = ["omni", *argv]

    # Typer may call sys.exit(), let it pass through.
    with suppress(SystemExit):
        app()


# Test compatibility: unit tests invoke `app` directly via CliRunner without entry_point().
# In that mode we eagerly register all command groups.
if "pytest" in sys.modules:
    _register_commands_for(None)


if __name__ == "__main__":
    entry_point()


__all__ = ["_is_verbose", "app", "entry_point", "main"]
