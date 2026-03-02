#!/usr/bin/env python3
"""Selection/filter helpers for command-events case catalog."""

from __future__ import annotations

from typing import Any


def select_cases(
    cases: tuple[Any, ...],
    suites: tuple[str, ...],
    case_ids: tuple[str, ...],
) -> list[Any]:
    """Filter cases by explicit ids or suite selection."""
    if case_ids:
        requested = set(case_ids)
        selected = [case for case in cases if case.case_id in requested]
        missing = sorted(requested - {case.case_id for case in selected})
        if missing:
            raise ValueError(
                "Unknown case id(s): "
                + ", ".join(missing)
                + ". Use --list-cases to inspect available ids."
            )
        return selected

    suite_set = set(suites)
    if "all" in suite_set:
        return list(cases)
    return [case for case in cases if any(suite in suite_set for suite in case.suites)]
