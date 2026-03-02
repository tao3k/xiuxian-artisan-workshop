# agent/cli/commands/skill/manage.py
"""
Management commands for skill CLI.

Contains: run, test, check commands.
(install/update unavailable in thin client model)
"""

from __future__ import annotations

import typer
from rich.panel import Panel
from rich.table import Table

from .base import cli_log_handler, err_console, run_skills, skill_app


@skill_app.command("run")
def skill_run(
    command: str = typer.Argument(..., help="Skill command in format 'skill.command'"),
    args_json: str | None = typer.Argument(None, help="JSON arguments for the command"),
    json_output: bool = typer.Option(
        False, "--json", "-j", help="Output raw JSON instead of markdown content"
    ),
    reuse_process: bool = typer.Option(
        True,
        "--reuse-process/--no-reuse-process",
        help="Use persistent local runner daemon for lower repeated latency (default: enabled).",
    ),
):
    """Execute a skill command."""
    commands = [command]
    if args_json:
        commands.append(args_json)
    if json_output:
        from omni.agent.cli.runner_json import run_skills_json

        exit_code = run_skills_json(commands, reuse_process=reuse_process)
        if exit_code != 0:
            raise typer.Exit(exit_code)
        return
    run_skills(
        commands,
        json_output=False,
        log_handler=cli_log_handler,
        reuse_process=reuse_process,
    )


runner_app = typer.Typer(help="Manage persistent local skill runner daemon")
skill_app.add_typer(runner_app, name="runner")


def _safe_int(value: object, default: int = 0) -> int:
    """Parse integer-ish values from JSON report payloads."""
    if isinstance(value, bool):
        return default
    if isinstance(value, int):
        return value
    if isinstance(value, str):
        try:
            return int(value)
        except ValueError:
            return default
    return default


def _extract_skill_name_from_nodeid(nodeid: str, known_skill_names: set[str]) -> str | None:
    """Extract skill name from pytest nodeid path segments ending with `<skill>/tests/...`."""
    normalized = str(nodeid).replace("\\", "/")
    parts = [part for part in normalized.split("/") if part and part != "."]
    for index in range(len(parts) - 1):
        if parts[index + 1] == "tests" and parts[index] in known_skill_names:
            return parts[index]
    return None


@runner_app.command("status")
def skill_runner_status(
    json_output: bool = typer.Option(False, "--json", "-j", help="Output status as JSON"),
):
    """Show runner daemon status."""
    from omni.agent.cli.runner_json import get_runner_daemon_status
    from omni.foundation.utils import json_codec as json

    status = get_runner_daemon_status()
    if json_output:
        typer.echo(json.dumps(status, indent=2, ensure_ascii=False))
        return
    if status.get("running") is True:
        pid = status.get("pid", "unknown")
        err_console.print(Panel(f"Runner daemon is running (pid={pid}).", title="Runner Status"))
        return
    message = str(status.get("error") or "Runner daemon is not running.")
    err_console.print(Panel(message, title="Runner Status", style="yellow"))


@runner_app.command("start")
def skill_runner_start(
    json_output: bool = typer.Option(False, "--json", "-j", help="Output status as JSON"),
):
    """Start runner daemon and wait for readiness."""
    from omni.agent.cli.runner_json import start_runner_daemon
    from omni.foundation.utils import json_codec as json

    status = start_runner_daemon()
    if json_output:
        typer.echo(json.dumps(status, indent=2, ensure_ascii=False))
        return
    if status.get("running") is True:
        started = bool(status.get("started", False))
        message = "Runner daemon started." if started else "Runner daemon already running."
        err_console.print(Panel(message, title="Runner Start", style="green"))
        return
    message = str(status.get("error") or "Failed to start runner daemon.")
    err_console.print(Panel(message, title="Runner Start", style="red"))
    raise typer.Exit(1)


@runner_app.command("stop")
def skill_runner_stop(
    json_output: bool = typer.Option(False, "--json", "-j", help="Output status as JSON"),
):
    """Stop runner daemon if it is running."""
    from omni.agent.cli.runner_json import stop_runner_daemon
    from omni.foundation.utils import json_codec as json

    status = stop_runner_daemon()
    if json_output:
        typer.echo(json.dumps(status, indent=2, ensure_ascii=False))
        return
    if status.get("stopped") is True:
        err_console.print(Panel("Runner daemon stopped.", title="Runner Stop", style="green"))
        return
    message = str(status.get("error") or "Runner daemon is not running.")
    err_console.print(Panel(message, title="Runner Stop", style="yellow"))


# Remote install/update are intentionally unavailable in thin client mode.
@skill_app.command("install")
def skill_install(
    url: str = typer.Argument(..., help="Git repository URL"),
    name: str | None = typer.Argument(None, help="Skill name (derived from URL if not provided)"),
    version: str = typer.Option("main", "--version", "-v", help="Git ref (default: main)"),
):
    """Install a skill from a remote repository (unavailable in thin client mode)."""
    from omni.foundation.config.dirs import get_skills_dir

    err_console.print(
        Panel(
            "Remote skill installation is not available in thin client mode.\n"
            f"Skills are loaded from {get_skills_dir()}/ automatically.",
            title="Unavailable",
            style="blue",
        )
    )


@skill_app.command("update")
def skill_update(
    name: str = typer.Argument(..., help="Skill name"),
    version: str = typer.Option("main", "--version", "-v", help="Git ref"),
):
    """Update an installed skill (unavailable in thin client mode)."""
    from omni.foundation.config.dirs import get_skills_dir

    err_console.print(
        Panel(
            "Remote skill updates are not available in thin client mode.\n"
            f"Skills are loaded from {get_skills_dir()}/ automatically.",
            title="Unavailable",
            style="blue",
        )
    )


@skill_app.command("test")
def skill_test(
    skill_name: str | None = typer.Argument(None, help="Skill name to test (default: all skills)"),
    all_skills: bool = typer.Option(False, "--all", help="Test all skills with tests/ directory"),
):
    """Test skills using the testing framework."""
    import json
    import subprocess
    import tempfile

    from omni.foundation.config.skills import SKILLS_DIR

    skills_dir = SKILLS_DIR()

    if not skill_name and not all_skills:
        err_console.print(
            Panel(
                "Specify a skill name or use --all to test all skills",
                title="Info: Usage",
                style="blue",
            )
        )
        return

    if all_skills:
        # Collect all test directories
        test_dirs = []
        for skill_path in sorted(skills_dir.iterdir()):
            if skill_path.is_dir() and not skill_path.name.startswith("_"):
                tests_dir = skill_path / "tests"
                if tests_dir.exists() and list(tests_dir.glob("test_*.py")):
                    test_dirs.append(str(tests_dir))

        if not test_dirs:
            err_console.print(Panel("No skill tests found", title="Info", style="blue"))
            return

        # Run all tests together with JSON output
        with tempfile.NamedTemporaryFile(mode="w", suffix=".json", delete=False) as f:
            json_output_path = f.name

        try:
            # Run pytest with JSON report - output flows through to terminal
            err_console.print(
                f"[bold]Running tests in {len(test_dirs)} skill test directories...[/]"
            )
            result = subprocess.run(
                [
                    "uv",
                    "run",
                    "pytest",
                    "-v",
                    "--tb=short",
                    "--json-report",
                    f"--json-report-file={json_output_path}",
                    "--import-mode=importlib",  # Support implicit namespace packages (no __init__.py)
                    *test_dirs,
                ],
                cwd=str(skills_dir),
                stdout=None,  # Inherit parent stdout (show pytest output)
                stderr=None,  # Inherit parent stderr
            )

            # Parse JSON report and display results
            try:
                with open(json_output_path) as f:
                    report = json.load(f)

                # Build results by skill
                skill_results: dict[str, dict[str, object]] = {}
                known_skill_names = {
                    path.name
                    for path in skills_dir.iterdir()
                    if path.is_dir() and not path.name.startswith("_")
                }
                for test in report.get("tests", []):
                    nodeid = test.get("nodeid", "")
                    outcome = test.get("outcome", "unknown")

                    # Skip if skill name not found
                    detected_skill = _extract_skill_name_from_nodeid(nodeid, known_skill_names)
                    if detected_skill is None:
                        continue

                    # Initialize skill results dict
                    if detected_skill not in skill_results:
                        skill_results[detected_skill] = {
                            "passed": 0,
                            "failed": 0,
                            "skipped": 0,
                            "errors": [],
                        }

                    # Count by outcome
                    if outcome == "passed":
                        skill_results[detected_skill]["passed"] += 1
                    elif outcome == "failed":
                        skill_results[detected_skill]["failed"] += 1
                        shortrepr = test.get("shortrepr", "")
                        errors = skill_results[detected_skill]["errors"]
                        if isinstance(errors, list):
                            errors.append(shortrepr)
                    elif outcome == "skipped":
                        skill_results[detected_skill]["skipped"] += 1

                # Display results table
                from rich.table import Table

                table = Table(title="🧪 Skill Test Results", show_header=True)
                table.add_column("Skill", style="bold")
                table.add_column("Passed", justify="right")
                table.add_column("Failed", justify="right")
                table.add_column("Skipped", justify="right")
                table.add_column("Status")

                total_passed = total_failed = total_skipped = 0
                for skill, stats in sorted(skill_results.items()):
                    passed = _safe_int(stats.get("passed"))
                    failed = _safe_int(stats.get("failed"))
                    skipped = _safe_int(stats.get("skipped"))
                    total_passed += passed
                    total_failed += failed
                    total_skipped += skipped

                    status = "✅ PASS" if failed == 0 else "❌ FAIL"
                    style = "green" if failed == 0 else "red"

                    errors_obj = stats.get("errors")
                    errors = errors_obj if isinstance(errors_obj, list) else []
                    if failed > 0:
                        errors_text = ", ".join(str(item) for item in errors[:2])
                        if len(errors) > 2:
                            errors_text += f"... +{len(errors) - 2} more"
                        details = f"[{style}]{status}[/] ({errors_text})"
                    else:
                        details = f"[{style}]{status}[/]"

                    table.add_row(
                        skill,
                        str(passed),
                        str(failed),
                        str(skipped),
                        details,
                    )

                report_summary = report.get("summary")
                report_summary_obj = report_summary if isinstance(report_summary, dict) else {}
                report_total = _safe_int(report_summary_obj.get("total"))
                report_passed = _safe_int(report_summary_obj.get("passed"))
                report_failed = _safe_int(report_summary_obj.get("failed"))
                report_skipped = _safe_int(report_summary_obj.get("skipped"))
                if report_skipped == 0 and report_total > 0:
                    collected = _safe_int(report_summary_obj.get("collected"))
                    if collected > 0 and collected >= report_passed + report_failed:
                        report_skipped = max(0, collected - report_passed - report_failed)

                # Fallback path: some pytest json-report setups omit per-test rows.
                if not skill_results and report_total > 0:
                    summary_status = "✅ PASS" if report_failed == 0 else "❌ FAIL"
                    summary_style = "green" if report_failed == 0 else "red"
                    table.add_row(
                        "all-skills",
                        str(report_passed),
                        str(report_failed),
                        str(report_skipped),
                        f"[{summary_style}]{summary_status}[/] (summary fallback)",
                    )
                    total_passed = report_passed
                    total_failed = report_failed
                    total_skipped = report_skipped

                # Safety net: keep summary aligned with pytest json totals.
                if report_total > 0 and (total_passed + total_failed + total_skipped) == 0:
                    total_passed = report_passed
                    total_failed = report_failed
                    total_skipped = report_skipped

                err_console.print(table)

                # Summary
                total = total_passed + total_failed + total_skipped
                summary_style = "green" if total_failed == 0 else "red"
                err_console.print(
                    Panel(
                        f"Total: {total} | Passed: {total_passed} | Failed: {total_failed} | Skipped: {total_skipped}",
                        title="📊 Summary",
                        style=summary_style,
                    )
                )

            except (json.JSONDecodeError, FileNotFoundError) as e:
                err_console.print(
                    Panel(f"Failed to parse test results: {e}", title="❌ Error", style="red")
                )
                # Fallback to raw output
                if result.stdout:
                    err_console.print(result.stdout)
                if result.stderr:
                    err_console.print(result.stderr)

        finally:
            import os

            if os.path.exists(json_output_path):
                os.unlink(json_output_path)

        raise typer.Exit(0 if result.returncode == 0 else 1)

    elif skill_name:
        skill_path = skills_dir / skill_name
        tests_dir = skill_path / "tests"
        if not skill_path.exists():
            err_console.print(
                Panel(f"Skill '{skill_name}' not found", title="❌ Error", style="red")
            )
            raise typer.Exit(1)
        if not tests_dir.exists():
            err_console.print(
                Panel(f"No tests directory for '{skill_name}'", title="❌ Error", style="red")
            )
            raise typer.Exit(1)
        # Run tests for specific skill - output flows through to terminal
        err_console.print(f"[bold]Running tests for '{skill_name}'...[/]")
        result = subprocess.run(
            ["uv", "run", "pytest", str(tests_dir), "-v", "--tb=short"],
            cwd=str(skills_dir),
            stdout=None,  # Inherit parent stdout
            stderr=None,  # Inherit parent stderr
        )
        raise typer.Exit(result.returncode)


@skill_app.command("check")
def skill_check(
    skill_name: str | None = typer.Argument(None, help="Skill name to check (default: all skills)"),
    show_example: bool = typer.Option(
        False, "--example", "-e", help="Show _template skill example"
    ),
):
    """Validate skill structure or show template example."""

    from omni.foundation.config.skills import SKILLS_DIR

    # Handle --example option
    if show_example:
        template_path = SKILLS_DIR() / "_template" / "SKILL.md"
        if template_path.exists():
            content = template_path.read_text()
            err_console.print(
                Panel(
                    content,
                    title="_template/SKILL.md",
                    subtitle=f"Path: {template_path}",
                    expand=False,
                )
            )
        else:
            err_console.print(
                Panel(
                    f"Template not found at: {template_path}",
                    title="❌ Error",
                    style="red",
                )
            )
        return

    skills_dir = SKILLS_DIR()

    def check_skill(name: str) -> tuple[bool, list[str], list[str]]:
        """Check a single skill and return (success, found, missing)."""
        skill_dir = skills_dir / name

        if not skill_dir.exists():
            return False, [], [f"Skill directory not found: {name}"]

        required_files = ["SKILL.md"]
        optional_files = ["scripts/", "tests/", "prompts.md", "README.md"]

        missing = []
        found = []

        for f in required_files:
            path = skill_dir / f
            if path.exists():
                found.append(f)
            else:
                missing.append(f)

        for f in optional_files:
            path = skill_dir / f
            if path.exists():
                found.append(f)

        return len(missing) == 0, found, missing

    if skill_name:
        success, found, missing = check_skill(skill_name)
        if success:
            err_console.print(
                Panel(
                    f"✅ Skill '{skill_name}' is valid\nFound: {', '.join(found)}",
                    title="✅ Valid",
                    style="green",
                )
            )
        else:
            err_console.print(
                Panel(
                    f"❌ Missing: {', '.join(missing)}\nFound: {', '.join(found)}",
                    title="❌ Invalid",
                    style="red",
                )
            )
    else:
        # Check all skills
        table = Table(title="🔍 Skill Structure Check", show_header=True)
        table.add_column("Skill", style="bold")
        table.add_column("Status")
        table.add_column("Details")

        for skill_path in sorted(skills_dir.iterdir()):
            if skill_path.is_dir() and not skill_path.name.startswith("_"):
                success, found, missing = check_skill(skill_path.name)
                status = "✅ Valid" if success else "❌ Invalid"
                style = "green" if success else "red"
                details = (
                    f"Found: {len(found)}" if success else f"Missing: {', '.join(missing[:2])}"
                )
                table.add_row(skill_path.name, f"[{style}]{status}[/]", details)

        err_console.print(table)
