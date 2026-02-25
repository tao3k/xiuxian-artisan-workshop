#!/usr/bin/env python3
"""Role-oriented embedding performance smoke benchmark for omni-agent.

This script validates that `litellm_rs` (provider routing) and `mistral_local`
(local runtime endpoint) are both operational and reports latency/throughput.
It is not a winner-takes-all benchmark; each role is measured independently.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import math
import os
import shutil
import signal
import statistics
import subprocess
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import httpx

_ROOT = Path(__file__).resolve().parents[2]
_DEFAULT_REPORT = _ROOT / ".run" / "reports" / "omni-agent-embedding-role-perf-smoke.json"


def _percentile(values: list[float], pct: float) -> float:
    if not values:
        return 0.0
    if len(values) == 1:
        return values[0]
    ordered = sorted(values)
    rank = (len(ordered) - 1) * pct
    lower = math.floor(rank)
    upper = math.ceil(rank)
    if lower == upper:
        return ordered[int(rank)]
    weight = rank - lower
    return ordered[lower] * (1.0 - weight) + ordered[upper] * weight


def _stats(latencies_ms: list[float]) -> dict[str, float]:
    return {
        "avg_ms": round(statistics.fmean(latencies_ms), 2),
        "p95_ms": round(_percentile(latencies_ms, 0.95), 2),
        "max_ms": round(max(latencies_ms), 2),
    }


@dataclass
class RoleConfig:
    name: str
    port: int
    model: str
    settings_yaml: str


def _role_configs(base_port: int, upstream_base_url: str, embedding_model: str) -> list[RoleConfig]:
    upstream = upstream_base_url.strip().rstrip("/")
    model = embedding_model.strip()
    litellm_model = f"ollama/{model}"
    return [
        RoleConfig(
            name="litellm_rs",
            port=base_port,
            model=litellm_model,
            settings_yaml=(
                "agent:\n"
                "  llm_backend: litellm_rs\n"
                "embedding:\n"
                "  backend: litellm_rs\n"
                "  batch_max_size: 128\n"
                "  batch_max_concurrency: 1\n"
                f"  litellm_model: {litellm_model}\n"
                f"  model: {litellm_model}\n"
                f"  litellm_api_base: {upstream}\n"
                "memory:\n"
                "  embedding_backend: litellm_rs\n"
                f"  embedding_model: {litellm_model}\n"
                "  persistence_backend: local\n"
                "mcp:\n"
                "  agent_strict_startup: false\n"
            ),
        ),
        RoleConfig(
            name="mistral_local",
            port=base_port + 1,
            model=model,
            settings_yaml=(
                "agent:\n"
                "  llm_backend: mistral_local\n"
                "embedding:\n"
                "  backend: mistral_local\n"
                "  batch_max_size: 128\n"
                "  batch_max_concurrency: 1\n"
                f"  model: {model}\n"
                "memory:\n"
                "  embedding_backend: mistral_local\n"
                f"  embedding_model: {model}\n"
                "  persistence_backend: local\n"
                "mistral:\n"
                "  enabled: false\n"
                "  auto_start: false\n"
                f"  base_url: {upstream}\n"
                "mcp:\n"
                "  agent_strict_startup: false\n"
            ),
        ),
    ]


def _resolve_upstream_model_name(model: str) -> str:
    """Normalize role model into upstream endpoint model token."""
    if model.startswith("ollama/"):
        return model.removeprefix("ollama/")
    return model


def _extract_openai_model_ids(payload: dict[str, Any]) -> set[str]:
    """Extract model IDs from OpenAI-compatible /v1/models payload."""
    ids: set[str] = set()
    raw_items = payload.get("data")
    if isinstance(raw_items, list):
        for item in raw_items:
            if not isinstance(item, dict):
                continue
            for key in ("id", "model", "name"):
                value = item.get(key)
                if isinstance(value, str) and value.strip():
                    ids.add(value.strip())
    return ids


async def _assert_upstream_embedding_ready(
    *,
    base_url: str,
    model: str,
    request_timeout_secs: float,
    warmup_attempts: int,
    warmup_retry_backoff_secs: float,
) -> None:
    """Fail fast when upstream embedding endpoint/model is not available."""
    upstream_model = _resolve_upstream_model_name(model)
    timeout = httpx.Timeout(request_timeout_secs, connect=min(10.0, request_timeout_secs))
    last_error: str | None = None

    async with httpx.AsyncClient(timeout=timeout) as client:
        for attempt in range(1, warmup_attempts + 1):
            try:
                models_resp = await client.get(f"{base_url}/v1/models")
                if models_resp.status_code == 200:
                    try:
                        model_ids = _extract_openai_model_ids(models_resp.json())
                    except json.JSONDecodeError:
                        model_ids = set()
                    if model_ids and upstream_model not in model_ids:
                        available = ", ".join(sorted(model_ids))
                        raise RuntimeError(
                            "upstream model not found: "
                            f"required='{upstream_model}', available=[{available}] "
                            "(hint: verify OLLAMA_MODELS points to project .data/models)"
                        )

                probe_resp = await client.post(
                    f"{base_url}/v1/embeddings",
                    json={"input": ["upstream readiness probe"], "model": upstream_model},
                )
                if probe_resp.status_code == 200:
                    return

                body_preview = (probe_resp.text or "")[:200]
                if probe_resp.status_code == 404:
                    raise RuntimeError(
                        "upstream /v1/embeddings returned 404 for model "
                        f"'{upstream_model}': {body_preview}"
                    )
                last_error = (
                    f"upstream /v1/embeddings status={probe_resp.status_code}, body={body_preview}"
                )
            except Exception as exc:  # noqa: BLE001 - keep probe failure message explicit
                last_error = str(exc)

            if attempt < warmup_attempts:
                await asyncio.sleep(warmup_retry_backoff_secs * attempt)

    raise RuntimeError(
        "upstream embedding endpoint is not ready after retries: "
        f"base_url='{base_url}', model='{upstream_model}', error='{last_error or 'unknown'}'. "
        "Hint: ensure Ollama is running and OLLAMA_MODELS points to project .data/models."
    )


def _ensure_agent_binary(agent_bin: Path) -> None:
    if agent_bin.exists():
        return
    subprocess.run(
        ["cargo", "build", "-p", "omni-agent", "--bin", "omni-agent"],
        cwd=_ROOT,
        check=True,
    )
    if not agent_bin.exists():
        raise RuntimeError(f"omni-agent binary not found after build: {agent_bin}")


def _write_role_conf(temp_root: Path, role: RoleConfig) -> Path:
    conf_dir = temp_root / f"conf-{role.name}" / "omni-dev-fusion"
    conf_dir.mkdir(parents=True, exist_ok=True)
    settings_path = conf_dir / "settings.yaml"
    settings_path.write_text(role.settings_yaml, encoding="utf-8")
    return conf_dir.parent


def _wait_health(base_url: str, timeout_secs: float) -> None:
    deadline = time.monotonic() + timeout_secs
    while time.monotonic() < deadline:
        try:
            response = httpx.get(f"{base_url}/health", timeout=2.0)
            if response.status_code == 200:
                return
        except Exception:
            pass
        time.sleep(0.2)
    raise TimeoutError(f"gateway health not ready within {timeout_secs:.1f}s: {base_url}")


def _start_gateway(
    agent_bin: Path, conf_root: Path, port: int, log_path: Path
) -> subprocess.Popen[str]:
    env = os.environ.copy()
    env["OMNI_AGENT_MCP_STRICT_STARTUP"] = "false"
    env["RUST_LOG"] = env.get("RUST_LOG", "omni_agent=warn")
    command = [
        str(agent_bin),
        "--conf",
        str(conf_root),
        "gateway",
        "--bind",
        f"127.0.0.1:{port}",
        "--mcp-config",
        ".mcp.json",
    ]
    log_file = log_path.open("w", encoding="utf-8")
    process = subprocess.Popen(
        command,
        cwd=_ROOT,
        env=env,
        stdout=log_file,
        stderr=subprocess.STDOUT,
        text=True,
    )
    return process


def _stop_gateway(process: subprocess.Popen[str]) -> None:
    if process.poll() is not None:
        return
    process.send_signal(signal.SIGTERM)
    try:
        process.wait(timeout=8)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)


async def _sequential_runs(
    client: httpx.AsyncClient,
    url: str,
    payload_for_index: Any,
    count: int,
) -> dict[str, Any]:
    latencies: list[float] = []
    errors: list[str] = []
    for index in range(count):
        started = time.perf_counter()
        payload = payload_for_index(index)
        try:
            response = await client.post(url, json=payload)
            if response.status_code != 200:
                errors.append(f"{response.status_code}: {response.text[:200]}")
        except Exception as exc:
            errors.append(str(exc))
        latencies.append((time.perf_counter() - started) * 1000.0)

    result = {
        "count": count,
        "ok": count - len(errors),
        "err": len(errors),
        "errors": errors[:3],
    }
    result.update(_stats(latencies))
    return result


async def _concurrent_runs(
    client: httpx.AsyncClient,
    url: str,
    payload_for_index: Any,
    total_requests: int,
    concurrency: int,
) -> dict[str, Any]:
    semaphore = asyncio.Semaphore(concurrency)
    latencies: list[float] = []
    errors: list[str] = []

    async def _one(index: int) -> None:
        async with semaphore:
            started = time.perf_counter()
            payload = payload_for_index(index)
            try:
                response = await client.post(url, json=payload)
                if response.status_code != 200:
                    errors.append(f"{response.status_code}: {response.text[:160]}")
            except Exception as exc:
                errors.append(str(exc))
            latencies.append((time.perf_counter() - started) * 1000.0)

    bench_started = time.perf_counter()
    await asyncio.gather(*(_one(index) for index in range(total_requests)))
    elapsed_secs = max(time.perf_counter() - bench_started, 1e-9)

    result = {
        "count": total_requests,
        "ok": total_requests - len(errors),
        "err": len(errors),
        "concurrency": concurrency,
        "rps": round(total_requests / elapsed_secs, 2),
        "errors": errors[:3],
    }
    result.update(_stats(latencies))
    return result


async def _post_with_retry(
    client: httpx.AsyncClient,
    url: str,
    payload: dict[str, Any],
    *,
    max_attempts: int,
    backoff_secs: float,
) -> httpx.Response:
    """Send a single request with bounded retry for cold-start warming."""
    for attempt in range(1, max_attempts + 1):
        try:
            return await client.post(url, json=payload)
        except (
            httpx.ConnectError,
            httpx.ConnectTimeout,
            httpx.ReadTimeout,
            httpx.RemoteProtocolError,
        ):
            if attempt >= max_attempts:
                raise
            await asyncio.sleep(backoff_secs * attempt)
    raise RuntimeError("unreachable retry loop in _post_with_retry")


async def _run_role_benchmark(
    role: RoleConfig,
    agent_bin: Path,
    temp_root: Path,
    upstream_base_url: str,
    health_timeout_secs: float,
    single_runs: int,
    batch_runs: int,
    concurrent_total: int,
    concurrent_width: int,
    request_timeout_secs: float,
    warmup_attempts: int,
    warmup_retry_backoff_secs: float,
) -> dict[str, Any]:
    conf_root = _write_role_conf(temp_root, role)
    log_path = temp_root / f"omni-agent-{role.name}.log"
    await _assert_upstream_embedding_ready(
        base_url=upstream_base_url,
        model=role.model,
        request_timeout_secs=request_timeout_secs,
        warmup_attempts=warmup_attempts,
        warmup_retry_backoff_secs=warmup_retry_backoff_secs,
    )
    process = _start_gateway(agent_bin, conf_root, role.port, log_path)
    base_url = f"http://127.0.0.1:{role.port}"
    try:
        _wait_health(base_url, timeout_secs=health_timeout_secs)
        endpoint = f"{base_url}/v1/embeddings"

        def single_payload(index: int) -> dict[str, Any]:
            return {
                "input": [f"role benchmark single request #{index}"],
                "model": role.model,
            }

        def batch_payload(index: int) -> dict[str, Any]:
            return {
                "input": [
                    f"role benchmark batch request #{index} item #{item_idx}"
                    for item_idx in range(8)
                ],
                "model": role.model,
            }

        limits = httpx.Limits(max_connections=128, max_keepalive_connections=32)
        timeout = httpx.Timeout(request_timeout_secs, connect=min(10.0, request_timeout_secs))
        async with httpx.AsyncClient(timeout=timeout, limits=limits) as client:
            # warm-up
            for warmup_index in (-2, -1):
                response = await _post_with_retry(
                    client,
                    endpoint,
                    single_payload(warmup_index),
                    max_attempts=warmup_attempts,
                    backoff_secs=warmup_retry_backoff_secs,
                )
                response.raise_for_status()

            single_result = await _sequential_runs(client, endpoint, single_payload, single_runs)
            batch_result = await _sequential_runs(client, endpoint, batch_payload, batch_runs)
            concurrent_result = await _concurrent_runs(
                client,
                endpoint,
                single_payload,
                concurrent_total,
                concurrent_width,
            )

        return {
            "role": role.name,
            "endpoint": endpoint,
            "model": role.model,
            "single": single_result,
            "batch8": batch_result,
            "concurrent_single": concurrent_result,
            "log_file": str(log_path),
        }
    finally:
        _stop_gateway(process)


def _validate_role_result(
    role_result: dict[str, Any],
    max_single_p95_ms: float | None,
    max_batch8_p95_ms: float | None,
    min_concurrent_rps: float | None,
) -> list[str]:
    failures: list[str] = []
    role = str(role_result["role"])
    single = role_result["single"]
    batch8 = role_result["batch8"]
    concurrent = role_result["concurrent_single"]

    if int(single["err"]) > 0:
        failures.append(f"{role}: single has errors={single['err']}")
    if int(batch8["err"]) > 0:
        failures.append(f"{role}: batch8 has errors={batch8['err']}")
    if int(concurrent["err"]) > 0:
        failures.append(f"{role}: concurrent_single has errors={concurrent['err']}")

    if max_single_p95_ms is not None and float(single["p95_ms"]) > max_single_p95_ms:
        failures.append(
            f"{role}: single p95 {single['p95_ms']}ms > threshold {max_single_p95_ms}ms"
        )
    if max_batch8_p95_ms is not None and float(batch8["p95_ms"]) > max_batch8_p95_ms:
        failures.append(
            f"{role}: batch8 p95 {batch8['p95_ms']}ms > threshold {max_batch8_p95_ms}ms"
        )
    if min_concurrent_rps is not None and float(concurrent["rps"]) < min_concurrent_rps:
        failures.append(
            f"{role}: concurrent rps {concurrent['rps']} < threshold {min_concurrent_rps}"
        )
    return failures


async def _main() -> int:
    parser = argparse.ArgumentParser(
        description="Role-oriented embedding performance smoke benchmark for omni-agent."
    )
    parser.add_argument("--base-port", type=int, default=18870)
    parser.add_argument("--single-runs", type=int, default=20)
    parser.add_argument("--batch-runs", type=int, default=10)
    parser.add_argument("--concurrent-total", type=int, default=64)
    parser.add_argument("--concurrent-width", type=int, default=8)
    parser.add_argument(
        "--upstream-base-url",
        type=str,
        default=os.environ.get("OMNI_EMBED_UPSTREAM_BASE_URL", "http://127.0.0.1:11434"),
        help="Embedding upstream base URL used by both roles.",
    )
    parser.add_argument(
        "--embedding-model",
        type=str,
        default="qwen3-embedding:0.6b",
        help="Base embedding model name (without provider prefix).",
    )
    parser.add_argument("--health-timeout-secs", type=float, default=120.0)
    parser.add_argument("--request-timeout-secs", type=float, default=120.0)
    parser.add_argument("--warmup-attempts", type=int, default=4)
    parser.add_argument("--warmup-retry-backoff-secs", type=float, default=1.5)
    parser.add_argument("--report-json", type=Path, default=_DEFAULT_REPORT)
    parser.add_argument(
        "--agent-bin",
        type=Path,
        default=_ROOT / "target" / "debug" / "omni-agent",
    )
    parser.add_argument("--max-single-p95-ms", type=float, default=None)
    parser.add_argument("--max-batch8-p95-ms", type=float, default=None)
    parser.add_argument("--min-concurrent-rps", type=float, default=None)
    parser.add_argument(
        "--keep-temp",
        action="store_true",
        help="Keep temporary config/log directory for debugging.",
    )
    args = parser.parse_args()

    _ensure_agent_binary(args.agent_bin)
    args.report_json.parent.mkdir(parents=True, exist_ok=True)
    upstream_base_url = args.upstream_base_url.strip().rstrip("/")
    if not upstream_base_url:
        raise ValueError("--upstream-base-url must be non-empty.")
    embedding_model = args.embedding_model.strip()
    if not embedding_model:
        raise ValueError("--embedding-model must be non-empty.")
    if args.request_timeout_secs <= 0:
        raise ValueError("--request-timeout-secs must be positive.")
    if args.warmup_attempts <= 0:
        raise ValueError("--warmup-attempts must be positive.")
    if args.warmup_retry_backoff_secs <= 0:
        raise ValueError("--warmup-retry-backoff-secs must be positive.")

    temp_root = Path(tempfile.mkdtemp(prefix="omni-agent-role-perf-"))
    started = time.time()
    try:
        results = []
        for role in _role_configs(args.base_port, upstream_base_url, embedding_model):
            result = await _run_role_benchmark(
                role=role,
                agent_bin=args.agent_bin,
                temp_root=temp_root,
                upstream_base_url=upstream_base_url,
                health_timeout_secs=args.health_timeout_secs,
                single_runs=args.single_runs,
                batch_runs=args.batch_runs,
                concurrent_total=args.concurrent_total,
                concurrent_width=args.concurrent_width,
                request_timeout_secs=args.request_timeout_secs,
                warmup_attempts=args.warmup_attempts,
                warmup_retry_backoff_secs=args.warmup_retry_backoff_secs,
            )
            results.append(result)

        payload = {
            "schema": "omni_agent.embedding.role_perf_smoke.v1",
            "generated_at_epoch": int(started),
            "duration_secs": round(time.time() - started, 2),
            "base_port": args.base_port,
            "upstream_base_url": upstream_base_url,
            "embedding_model": embedding_model,
            "single_runs": args.single_runs,
            "batch_runs": args.batch_runs,
            "concurrent_total": args.concurrent_total,
            "concurrent_width": args.concurrent_width,
            "roles": results,
        }

        failures: list[str] = []
        for role_result in results:
            failures.extend(
                _validate_role_result(
                    role_result,
                    max_single_p95_ms=args.max_single_p95_ms,
                    max_batch8_p95_ms=args.max_batch8_p95_ms,
                    min_concurrent_rps=args.min_concurrent_rps,
                )
            )
        payload["status"] = "pass" if not failures else "fail"
        payload["failures"] = failures

        args.report_json.write_text(
            json.dumps(payload, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )
        print(str(args.report_json))
        if failures:
            for failure in failures:
                print(f"FAIL: {failure}")
            return 1
        return 0
    finally:
        if args.keep_temp:
            print(f"kept temp dir: {temp_root}")
        else:
            shutil.rmtree(temp_root, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(asyncio.run(_main()))
