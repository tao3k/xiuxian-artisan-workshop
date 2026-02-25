#!/usr/bin/env python3
"""Scenario summary block builders for complex scenario markdown reports."""

from __future__ import annotations

from complex_scenarios_report_sections_support import behavioral_evidence_summary


def append_scenario_header(lines: list[str], scenario: dict[str, object]) -> None:
    """Append scenario-level summary block."""
    lines.extend(
        [
            f"### {scenario['scenario_id']}",
            f"- description: {scenario['description']}",
            f"- result: `{'PASS' if scenario['passed'] else 'FAIL'}`",
            f"- duration_ms: `{scenario['duration_ms']}`",
            "- behavioral_evidence: " + behavioral_evidence_summary(scenario),
            (
                "- complexity: "
                "steps={steps}, edges={edges}, critical_path={critical}, "
                "parallel_waves={parallel}, score={score}"
            ).format(
                steps=scenario["complexity"]["step_count"],
                edges=scenario["complexity"]["dependency_edges"],
                critical=scenario["complexity"]["critical_path_len"],
                parallel=scenario["complexity"]["parallel_waves"],
                score=scenario["complexity"]["complexity_score"],
            ),
            (
                "- requirement: "
                "steps>={steps}, edges>={edges}, critical_path>={critical}, "
                "parallel_waves>={parallel}"
            ).format(
                steps=scenario["requirement"]["steps"],
                edges=scenario["requirement"]["dependency_edges"],
                critical=scenario["requirement"]["critical_path_len"],
                parallel=scenario["requirement"]["parallel_waves"],
            ),
            (
                "- quality: "
                "error_signals={es}, negative_feedback_events={ne}, "
                "correction_checks={cc}, successful_corrections={sc}, "
                "planned_hits={ph}, natural_language_steps={nl}, "
                "recall_credit_events={rc}, decay_events={de}, score={score}"
            ).format(
                es=scenario["quality"]["error_signal_steps"],
                ne=scenario["quality"]["negative_feedback_events"],
                cc=scenario["quality"]["correction_check_steps"],
                sc=scenario["quality"]["successful_corrections"],
                ph=scenario["quality"]["planned_hits"],
                nl=scenario["quality"]["natural_language_steps"],
                rc=scenario["quality"]["recall_credit_events"],
                de=scenario["quality"]["decay_events"],
                score=scenario["quality"]["quality_score"],
            ),
            (
                "- quality_requirement: "
                "error_signals>={es}, negative_feedback_events>={ne}, "
                "correction_checks>={cc}, successful_corrections>={sc}, "
                "planned_hits>={ph}, natural_language_steps>={nl}, "
                "recall_credit_events>={rc}, decay_events>={de}"
            ).format(
                es=scenario["quality_requirement"]["min_error_signals"],
                ne=scenario["quality_requirement"]["min_negative_feedback_events"],
                cc=scenario["quality_requirement"]["min_correction_checks"],
                sc=scenario["quality_requirement"]["min_successful_corrections"],
                ph=scenario["quality_requirement"]["min_planned_hits"],
                nl=scenario["quality_requirement"]["min_natural_language_steps"],
                rc=scenario["quality_requirement"]["min_recall_credit_events"],
                de=scenario["quality_requirement"]["min_decay_events"],
            ),
        ]
    )

    if scenario["complexity_failures"]:
        lines.append("- complexity_failures:")
        for issue in scenario["complexity_failures"]:
            lines.append(f"  - {issue}")
    if scenario["quality_failures"]:
        lines.append("- quality_failures:")
        for issue in scenario["quality_failures"]:
            lines.append(f"  - {issue}")
