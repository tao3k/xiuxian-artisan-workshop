#!/usr/bin/env python3
"""
Evaluate wendao retrieval quality on a fixed query matrix.

Examples:
  uv run python scripts/evaluate_wendao_retrieval.py
  uv run python scripts/evaluate_wendao_retrieval.py --json
  uv run python scripts/evaluate_wendao_retrieval.py --binary .cache/target-codex-wendao/debug/wendao
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

from omni.foundation.config.paths import get_config_paths
from omni.foundation.runtime.cargo_subprocess_env import prepare_cargo_subprocess_env
from omni.foundation.runtime.gitops import get_project_root

DEFAULT_MATRIX = "docs/testing/wendao-query-regression-matrix.json"
MATRIX_SCHEMA = "xiuxian_wendao.query_matrix.v1"


@dataclass
class CaseResult:
    case_id: str
    query: str
    expected_paths: list[str]
    rank: int | None
    top_hit_path: str | None
    top_hit_score: float | None
    top3: bool
    top10: bool
    error: str | None


def _resolve_project_root() -> Path:
    try:
        return get_project_root().resolve()
    except Exception:
        prj_root = os.environ.get("PRJ_ROOT")
        if prj_root:
            return Path(prj_root).expanduser().resolve()
        raw = subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            text=True,
        ).strip()
        return Path(raw).resolve()


def _resolve_default_config_path() -> Path:
    return get_config_paths().wendao_settings_file.resolve()


def _resolve_env_target_bin(project_root: Path, profile: str) -> Path | None:
    raw_target_dir = os.environ.get("CARGO_TARGET_DIR")
    if not raw_target_dir:
        return None
    target_dir = Path(raw_target_dir).expanduser()
    if not target_dir.is_absolute():
        target_dir = (project_root / target_dir).resolve()
    return target_dir / profile / "wendao"


def _resolve_binary(project_root: Path, binary: str | None, build: bool, release: bool) -> Path:
    if binary:
        resolved = Path(binary).expanduser().resolve()
        if not resolved.exists():
            raise FileNotFoundError(f"wendao binary not found: {resolved}")
        return resolved

    profile = "release" if release else "debug"
    env_target_bin = _resolve_env_target_bin(project_root, profile)
    default_bin = project_root / "target" / profile / "wendao"
    codex_cache_bin = project_root / ".cache" / "target-codex-wendao" / profile / "wendao"

    if not build:
        if env_target_bin is not None and env_target_bin.exists():
            return env_target_bin
        if default_bin.exists():
            return default_bin
        if codex_cache_bin.exists():
            return codex_cache_bin

    if build or (not default_bin.exists() and not codex_cache_bin.exists()):
        cmd = ["cargo", "build", "-p", "xiuxian-wendao", "--bin", "wendao"]
        if release:
            cmd.append("--release")
        env = prepare_cargo_subprocess_env(os.environ)
        subprocess.run(cmd, cwd=project_root, check=True, env=env)
    if env_target_bin is not None and env_target_bin.exists():
        return env_target_bin
    if default_bin.exists():
        return default_bin
    if codex_cache_bin.exists():
        return codex_cache_bin
    raise FileNotFoundError(
        "wendao binary not found after build: "
        f"{default_bin} (also checked {codex_cache_bin} and {env_target_bin})"
    )


def _load_matrix(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("matrix payload must be object")
    schema = payload.get("schema")
    if schema != MATRIX_SCHEMA:
        raise ValueError(f"matrix.schema must be {MATRIX_SCHEMA!r}, got {schema!r}")
    cases = payload.get("cases")
    if not isinstance(cases, list) or not cases:
        raise ValueError("matrix.cases must be a non-empty list")
    return payload


def _run_case(
    *,
    binary: Path,
    root: Path,
    config: Path,
    query: str,
    limit: int,
    timeout_s: float,
) -> tuple[list[dict[str, Any]], str | None]:
    cmd = [
        str(binary),
        "search",
        query,
        "-r",
        str(root),
        "-c",
        str(config),
        "-l",
        str(limit),
        "-o",
        "json",
    ]

    try:
        proc = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=timeout_s,
            check=False,
        )
    except subprocess.TimeoutExpired:
        return [], f"timeout ({timeout_s}s)"

    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or f"exit={proc.returncode}").strip()
        return [], detail

    try:
        payload = json.loads(proc.stdout)
    except Exception as exc:
        return [], f"invalid-json: {exc}"

    hits = payload.get("results")
    if isinstance(hits, list):
        return hits, None
    return [], "missing-results"


def _rank_of_expected(hits: list[dict[str, Any]], expected_paths: list[str]) -> int | None:
    expected = {p.strip() for p in expected_paths if p.strip()}
    if not expected:
        return None
    for idx, hit in enumerate(hits, start=1):
        path = str(hit.get("path", "")).strip()
        if path in expected:
            return idx
    return None


def _evaluate(
    *,
    binary: Path,
    root: Path,
    config: Path,
    matrix: dict[str, Any],
    limit: int,
    timeout_s: float,
    query_prefix: str,
) -> tuple[dict[str, Any], list[CaseResult]]:
    cases_raw = matrix["cases"]
    results: list[CaseResult] = []

    for item in cases_raw:
        case_id = str(item.get("id", "")).strip() or "UNKNOWN"
        query = str(item.get("query", "")).strip()
        expected_paths = [str(v).strip() for v in item.get("expected_paths", []) if str(v).strip()]
        if not query:
            results.append(
                CaseResult(
                    case_id=case_id,
                    query=query,
                    expected_paths=expected_paths,
                    rank=None,
                    top_hit_path=None,
                    top_hit_score=None,
                    top3=False,
                    top10=False,
                    error="empty-query",
                )
            )
            continue

        effective_query = f"{query_prefix}{query}".strip()
        hits, err = _run_case(
            binary=binary,
            root=root,
            config=config,
            query=effective_query,
            limit=limit,
            timeout_s=timeout_s,
        )
        rank = None if err else _rank_of_expected(hits, expected_paths)
        top_hit_path = str(hits[0].get("path")) if hits else None
        top_hit_score = (
            float(hits[0].get("score")) if hits and hits[0].get("score") is not None else None
        )
        results.append(
            CaseResult(
                case_id=case_id,
                query=query,
                expected_paths=expected_paths,
                rank=rank,
                top_hit_path=top_hit_path,
                top_hit_score=top_hit_score,
                top3=rank is not None and rank <= 3,
                top10=rank is not None and rank <= 10,
                error=err,
            )
        )

    total = len(results)
    top1_count = sum(1 for r in results if r.rank == 1)
    top3_count = sum(1 for r in results if r.top3)
    top10_count = sum(1 for r in results if r.top10)
    error_count = sum(1 for r in results if r.error)
    summary = {
        "schema": "xiuxian_wendao.retrieval_eval.v1",
        "total_cases": total,
        "top1_count": top1_count,
        "top3_count": top3_count,
        "top10_count": top10_count,
        "error_count": error_count,
        "top1_rate": round((top1_count / total) if total else 0.0, 4),
        "top3_rate": round((top3_count / total) if total else 0.0, 4),
        "top10_rate": round((top10_count / total) if total else 0.0, 4),
    }
    return summary, results


def main() -> int:
    parser = argparse.ArgumentParser(description="Evaluate wendao retrieval query matrix")
    parser.add_argument("--root", default=".", help="wendao --root")
    parser.add_argument(
        "--config",
        default=None,
        help="wendao config path (defaults to config API: $PRJ_CONFIG_HOME/xiuxian-artisan-workshop/wendao.yaml)",
    )
    parser.add_argument(
        "--matrix-file",
        default=DEFAULT_MATRIX,
        help=f"query matrix JSON path (default: {DEFAULT_MATRIX})",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=10,
        help="wendao search limit used for each query",
    )
    parser.add_argument(
        "--query-prefix",
        default="",
        help="optional prefix prepended to every matrix query (for example: 'scope:mixed ')",
    )
    parser.add_argument("--timeout-s", type=float, default=20.0, help="timeout per query")
    parser.add_argument("--binary", default=None, help="wendao binary path")
    parser.add_argument("--release", action="store_true", help="use target/release/wendao")
    parser.add_argument("--no-build", action="store_true", help="do not auto build wendao")
    parser.add_argument(
        "--min-top3-rate",
        type=float,
        default=0.0,
        help="non-zero: fail when top3_rate < threshold",
    )
    parser.add_argument("--json", action="store_true", help="print JSON report")
    args = parser.parse_args()

    try:
        project_root = _resolve_project_root()
        matrix_path = (project_root / args.matrix_file).resolve()
        if args.config:
            config_arg = Path(args.config).expanduser()
            config_path = (
                config_arg.resolve()
                if config_arg.is_absolute()
                else (project_root / config_arg).resolve()
            )
        else:
            config_path = _resolve_default_config_path()
        root_path = Path(args.root).expanduser().resolve()
        matrix = _load_matrix(matrix_path)
        if not config_path.exists():
            raise FileNotFoundError(f"config not found: {config_path}")
        binary = _resolve_binary(
            project_root,
            args.binary,
            build=not args.no_build,
            release=bool(args.release),
        )
    except Exception as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2

    summary, case_results = _evaluate(
        binary=binary,
        root=root_path,
        config=config_path,
        matrix=matrix,
        limit=max(1, int(args.limit)),
        timeout_s=float(args.timeout_s),
        query_prefix=str(args.query_prefix),
    )

    failed_cases = [r for r in case_results if not r.top3 or r.error]
    gate_failed = args.min_top3_rate > 0 and summary["top3_rate"] < float(args.min_top3_rate)

    payload = {
        "summary": summary,
        "matrix_file": str(matrix_path),
        "config": str(config_path),
        "binary": str(binary),
        "limit": int(args.limit),
        "query_prefix": str(args.query_prefix),
        "failed_cases": [asdict(case) for case in failed_cases],
        "cases": [asdict(case) for case in case_results],
    }

    if args.json:
        print(json.dumps(payload, ensure_ascii=True, indent=2))
    else:
        print("wendao retrieval evaluation")
        print("=" * 56)
        print(f"binary:      {binary}")
        print(f"config:      {config_path}")
        print(f"matrix_file: {matrix_path}")
        print(
            f"metrics: Top1={summary['top1_count']}/{summary['total_cases']} "
            f"({summary['top1_rate']:.2%}), "
            f"Top3={summary['top3_count']}/{summary['total_cases']} "
            f"({summary['top3_rate']:.2%}), "
            f"Top10={summary['top10_count']}/{summary['total_cases']} "
            f"({summary['top10_rate']:.2%})"
        )
        if summary["error_count"] > 0:
            print(f"errors:      {summary['error_count']}")
        print("-" * 56)
        for case in case_results:
            status = "OK" if case.top3 and not case.error else "MISS"
            rank_str = str(case.rank) if case.rank is not None else "-"
            print(
                f"[{status}] {case.case_id} rank={rank_str:>2} query={case.query!r} "
                f"top={case.top_hit_path or '-'}"
            )
            if case.error:
                print(f"       error: {case.error}")
        if failed_cases:
            print("-" * 56)
            print("failed cases (Top3 miss or error):")
            for case in failed_cases:
                print(
                    f"- {case.case_id} query={case.query!r} expected={case.expected_paths} "
                    f"rank={case.rank} top={case.top_hit_path}"
                )

    if gate_failed:
        print(
            f"gate: FAIL (top3_rate {summary['top3_rate']:.4f} < {float(args.min_top3_rate):.4f})",
            file=sys.stderr,
        )
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
