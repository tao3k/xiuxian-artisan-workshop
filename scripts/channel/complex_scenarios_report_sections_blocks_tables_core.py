#!/usr/bin/env python3
"""Primary table builders for complex scenario report sections."""

from __future__ import annotations


def append_step_table(lines: list[str], scenario: dict[str, object]) -> None:
    """Append per-step status table."""
    lines.extend(
        [
            "",
            "| Step | Session | Wave | Event | Result | Duration (ms) |",
            "|---|---|---:|---|---|---:|",
        ]
    )
    for step in scenario["steps"]:
        status = "PASS" if step["passed"] else ("SKIP" if step["skipped"] else "FAIL")
        lines.append(
            "| `{step}` | `{session}` | {wave} | `{event}` | {status} | {duration} |".format(
                step=step["step_id"],
                session=step["session_key"],
                wave=step["wave_index"],
                event=step["event"] or "-",
                status=status,
                duration=step["duration_ms"],
            )
        )


def append_natural_language_trace(lines: list[str], scenario: dict[str, object]) -> None:
    """Append natural-language prompt/reply trace table."""
    lines.extend(
        [
            "",
            "Natural-language trace:",
            "",
            "| Step | Prompt | Bot Reply |",
            "|---|---|---|",
        ]
    )
    for step in scenario["steps"]:
        if str(step["prompt"]).strip().startswith("/"):
            continue
        prompt = str(step["prompt"]).replace("|", "\\|")
        bot_excerpt = str(step.get("bot_excerpt") or "").replace("|", "\\|")
        lines.append(f"| `{step['step_id']}` | {prompt} | {bot_excerpt} |")


def append_memory_adaptation(lines: list[str], scenario: dict[str, object]) -> None:
    """Append memory adaptation evidence table."""
    lines.extend(
        [
            "",
            "Memory adaptation evidence:",
            "",
            "| Step | planned_bias | decision | recall_credit_count | decay_count | cmd_feedback_delta | heuristic_feedback_delta |",
            "|---|---:|---|---:|---:|---:|---:|",
        ]
    )
    for step in scenario["steps"]:
        planned_bias = step.get("memory_planned_bias")
        decision = step.get("memory_decision") or "-"
        recall_credit_count = int(step.get("memory_recall_credit_count") or 0)
        decay_count = int(step.get("memory_decay_count") or 0)
        cmd_delta = step.get("feedback_command_bias_delta")
        heur_delta = step.get("feedback_heuristic_bias_delta")
        lines.append(
            "| `{sid}` | {pb} | {dec} | {rc} | {de} | {cd} | {hd} |".format(
                sid=step["step_id"],
                pb=f"{planned_bias:.6f}" if isinstance(planned_bias, (int, float)) else "-",
                dec=decision,
                rc=recall_credit_count,
                de=decay_count,
                cd=f"{cmd_delta:.6f}" if isinstance(cmd_delta, (int, float)) else "-",
                hd=f"{heur_delta:.6f}" if isinstance(heur_delta, (int, float)) else "-",
            )
        )
