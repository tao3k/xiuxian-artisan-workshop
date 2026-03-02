#!/usr/bin/env python3
"""Sweep mistral-sdk /embed/batch performance and persist CI-comparable reports.

This script starts an isolated omni-agent gateway for each `max_num_seqs` value,
benchmarks `/embed/batch` under multiple batch shapes, and writes:

- timestamped JSON report
- timestamped Markdown summary
- `latest` JSON/Markdown snapshots for CI diff/comparison
"""

from __future__ import annotations

import argparse
import asyncio
import json
import math
import os
import signal
import statistics
import subprocess
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import httpx


ROOT = Path(__file__).resolve().parents[2]
DEFAULT_AGENT_BIN = ROOT / "target" / "debug" / "omni-agent"
DEFAULT_REPORT_DIR = ROOT / ".run" / "reports"
DEFAULT_MODEL = "Qwen/Qwen3-Embedding-0.6B"
DEFAULT_HOST = "127.0.0.1"
DEFAULT_PORT_BASE = 18920
DEFAULT_TIMEOUT_SECS = 120.0
DEFAULT_MAX_NUM_SEQS = (64, 128, 256)


@dataclass(frozen=True)
class BatchShape:
    batch_size: int
    concurrency: int
    total_requests: int

    @property
    def label(self) -> str:
        return f"b{self.batch_size}-c{self.concurrency}-n{self.total_requests}"


@dataclass
class CaseResult:
    max_num_seqs: int
    shape: BatchShape
    ok: int
    err: int
    elapsed_secs: float
    rps: float
    text_rps: float
    avg_ms: float
    p95_ms: float
    p99_ms: float
    max_ms: float
    errors: list[str]

    def to_dict(self) -> dict[str, Any]:
        return {
            "max_num_seqs": self.max_num_seqs,
            "shape": {
                "batch_size": self.shape.batch_size,
                "concurrency": self.shape.concurrency,
                "total_requests": self.shape.total_requests,
                "label": self.shape.label,
            },
            "ok": self.ok,
            "err": self.err,
            "elapsed_secs": round(self.elapsed_secs, 3),
            "rps": round(self.rps, 2),
            "text_rps": round(self.text_rps, 2),
            "avg_ms": round(self.avg_ms, 2),
            "p95_ms": round(self.p95_ms, 2),
            "p99_ms": round(self.p99_ms, 2),
            "max_ms": round(self.max_ms, 2),
            "errors": self.errors[:5],
        }


def percentile(values: list[float], pct: float) -> float:
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


def parse_shape(raw: str) -> BatchShape:
    try:
        batch, conc, total = raw.lower().split("x")
        shape = BatchShape(int(batch), int(conc), int(total))
    except Exception as exc:  # noqa: BLE001
        raise argparse.ArgumentTypeError(
            f"invalid --shape '{raw}', expected <batch>x<concurrency>x<total>"
        ) from exc
    if shape.batch_size <= 0 or shape.concurrency <= 0 or shape.total_requests <= 0:
        raise argparse.ArgumentTypeError(f"invalid --shape '{raw}', values must be positive")
    return shape


def render_config(
    model: str, max_num_seqs: int, batch_max_concurrency: int, max_in_flight: int
) -> str:
    return (
        "[agent]\n"
        'llm_backend = "http"\n'
        "\n"
        "[llm.embedding]\n"
        'backend = "mistral_sdk"\n'
        "batch_max_size = 128\n"
        f"batch_max_concurrency = {batch_max_concurrency}\n"
        f"max_in_flight = {max_in_flight}\n"
        f'model = "{model}"\n'
        "\n"
        "[llm.mistral]\n"
        f"sdk_embedding_max_num_seqs = {max_num_seqs}\n"
        "\n"
        "[memory]\n"
        'embedding_backend = "mistral_sdk"\n'
        f'embedding_model = "{model}"\n'
        'persistence_backend = "local"\n'
        "\n"
        "[mcp]\n"
        "strict_startup = false\n"
    )


def write_conf_root(
    root: Path, model: str, max_num_seqs: int, batch_max_concurrency: int, max_in_flight: int
) -> Path:
    conf_dir = root / "xiuxian-artisan-workshop"
    conf_dir.mkdir(parents=True, exist_ok=True)
    toml_path = conf_dir / "xiuxian.toml"
    toml_path.write_text(
        render_config(model, max_num_seqs, batch_max_concurrency, max_in_flight),
        encoding="utf-8",
    )
    return conf_dir.parent


def start_gateway(
    agent_bin: Path, conf_root: Path, host: str, port: int, log_path: Path
) -> subprocess.Popen[str]:
    if not agent_bin.exists():
        raise FileNotFoundError(f"omni-agent binary not found: {agent_bin}")
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_file = log_path.open("w", encoding="utf-8")
    env = os.environ.copy()
    env["OMNI_AGENT_MCP_STRICT_STARTUP"] = "false"
    env["RUST_LOG"] = env.get("RUST_LOG", "omni_agent=warn")
    return subprocess.Popen(
        [
            str(agent_bin),
            "--conf",
            str(conf_root),
            "gateway",
            "--bind",
            f"{host}:{port}",
            "--mcp-config",
            ".mcp.json",
        ],
        cwd=ROOT,
        env=env,
        stdout=log_file,
        stderr=subprocess.STDOUT,
        text=True,
    )


def stop_gateway(process: subprocess.Popen[str]) -> None:
    if process.poll() is not None:
        return
    process.terminate()
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(timeout=5)


async def wait_health(base_url: str, timeout_secs: float) -> None:
    deadline = time.monotonic() + timeout_secs
    async with httpx.AsyncClient(timeout=httpx.Timeout(3.0)) as client:
        while time.monotonic() < deadline:
            try:
                response = await client.get(f"{base_url}/health")
                if response.status_code == 200:
                    return
            except Exception:  # noqa: BLE001
                pass
            await asyncio.sleep(0.2)
    raise TimeoutError(f"gateway health not ready within {timeout_secs}s: {base_url}")


def build_payload(
    case_label: str, request_index: int, batch_size: int, model: str
) -> dict[str, Any]:
    texts = [
        f"embed-batch sweep {case_label} request={request_index} item={item_index}"
        for item_index in range(batch_size)
    ]
    return {"texts": texts, "model": model}


async def benchmark_shape(
    client: httpx.AsyncClient, endpoint: str, model: str, max_num_seqs: int, shape: BatchShape
) -> CaseResult:
    semaphore = asyncio.Semaphore(shape.concurrency)
    started = time.monotonic()
    latencies_ms: list[float] = []
    errors: list[str] = []
    ok = 0

    async def one_request(request_index: int) -> tuple[bool, float, str | None]:
        async with semaphore:
            payload = build_payload(shape.label, request_index, shape.batch_size, model)
            req_started = time.monotonic()
            try:
                response = await client.post(endpoint, json=payload)
                elapsed_ms = (time.monotonic() - req_started) * 1000.0
                if response.status_code != 200:
                    body = response.text[:200]
                    return False, elapsed_ms, f"status={response.status_code} body={body}"
                return True, elapsed_ms, None
            except Exception as exc:  # noqa: BLE001
                elapsed_ms = (time.monotonic() - req_started) * 1000.0
                return False, elapsed_ms, str(exc)

    tasks = [asyncio.create_task(one_request(i)) for i in range(shape.total_requests)]
    for task in asyncio.as_completed(tasks):
        success, elapsed_ms, error = await task
        latencies_ms.append(elapsed_ms)
        if success:
            ok += 1
        elif error is not None:
            errors.append(error)

    elapsed_secs = max(time.monotonic() - started, 1e-9)
    err = shape.total_requests - ok
    rps = shape.total_requests / elapsed_secs
    text_rps = (ok * shape.batch_size) / elapsed_secs
    avg_ms = statistics.fmean(latencies_ms) if latencies_ms else 0.0
    p95_ms = percentile(latencies_ms, 0.95)
    p99_ms = percentile(latencies_ms, 0.99)
    max_ms = max(latencies_ms) if latencies_ms else 0.0
    return CaseResult(
        max_num_seqs=max_num_seqs,
        shape=shape,
        ok=ok,
        err=err,
        elapsed_secs=elapsed_secs,
        rps=rps,
        text_rps=text_rps,
        avg_ms=avg_ms,
        p95_ms=p95_ms,
        p99_ms=p99_ms,
        max_ms=max_ms,
        errors=errors[:10],
    )


def render_markdown(report: dict[str, Any]) -> str:
    lines = []
    lines.append("# Omni-Agent /embed/batch Sweep")
    lines.append("")
    lines.append(f"- Generated At: `{report['generated_at_iso']}`")
    lines.append(f"- Model: `{report['model']}`")
    lines.append(f"- Endpoint: `{report['endpoint_path']}`")
    lines.append(f"- max_num_seqs sweep: `{report['max_num_seqs']}`")
    lines.append("")
    lines.append(
        "| max_num_seqs | shape | ok | err | rps | text_rps | avg_ms | p95_ms | p99_ms | max_ms |"
    )
    lines.append("|---:|:---|---:|---:|---:|---:|---:|---:|---:|---:|")
    for row in report["results"]:
        lines.append(
            f"| {row['max_num_seqs']} | {row['shape']['label']} | {row['ok']} | {row['err']} | "
            f"{row['rps']:.2f} | {row['text_rps']:.2f} | {row['avg_ms']:.2f} | "
            f"{row['p95_ms']:.2f} | {row['p99_ms']:.2f} | {row['max_ms']:.2f} |"
        )
    lines.append("")
    best_p95 = report.get("best_p95")
    if best_p95:
        lines.append(
            f"- Best p95: `max_num_seqs={best_p95['max_num_seqs']}`, "
            f"`shape={best_p95['shape']['label']}`, p95=`{best_p95['p95_ms']:.2f}ms`"
        )
    best_text_rps = report.get("best_text_rps")
    if best_text_rps:
        lines.append(
            f"- Best text_rps: `max_num_seqs={best_text_rps['max_num_seqs']}`, "
            f"`shape={best_text_rps['shape']['label']}`, text_rps=`{best_text_rps['text_rps']:.2f}`"
        )
    return "\n".join(lines) + "\n"


async def run(args: argparse.Namespace) -> int:
    report_dir: Path = args.report_dir
    report_dir.mkdir(parents=True, exist_ok=True)
    now = time.strftime("%Y%m%d-%H%M%S")
    report_json = report_dir / f"omni-agent-embed-batch-sweep-{now}.json"
    report_md = report_dir / f"omni-agent-embed-batch-sweep-{now}.md"
    latest_json = report_dir / "omni-agent-embed-batch-sweep-latest.json"
    latest_md = report_dir / "omni-agent-embed-batch-sweep-latest.md"

    all_results: list[CaseResult] = []
    boot_failures: list[dict[str, Any]] = []
    host = args.host

    for index, max_num_seqs in enumerate(args.max_num_seqs):
        port = args.port_base + index
        base_url = f"http://{host}:{port}"
        with tempfile.TemporaryDirectory(prefix=f"embed-sweep-{max_num_seqs}-") as tmp:
            tmp_root = Path(tmp)
            conf_root = write_conf_root(
                tmp_root,
                args.model,
                max_num_seqs,
                args.batch_max_concurrency,
                args.max_in_flight,
            )
            log_path = ROOT / ".run" / "logs" / f"omni-agent-embed-batch-sweep-{max_num_seqs}.log"
            gateway = start_gateway(args.agent_bin, conf_root, host, port, log_path)
            try:
                await wait_health(base_url, args.health_timeout_secs)
                timeout = httpx.Timeout(
                    args.request_timeout_secs, connect=min(10.0, args.request_timeout_secs)
                )
                async with httpx.AsyncClient(timeout=timeout) as client:
                    for shape in args.shapes:
                        # Warmup
                        for warmup_idx in range(args.warmup_requests):
                            payload = build_payload(
                                shape.label, warmup_idx, shape.batch_size, args.model
                            )
                            warmup_response = await client.post(
                                f"{base_url}/embed/batch", json=payload
                            )
                            if warmup_response.status_code != 200:
                                preview = warmup_response.text[:200]
                                raise RuntimeError(
                                    f"warmup failed max_num_seqs={max_num_seqs} shape={shape.label} "
                                    f"status={warmup_response.status_code} body={preview}"
                                )
                        result = await benchmark_shape(
                            client,
                            f"{base_url}/embed/batch",
                            args.model,
                            max_num_seqs,
                            shape,
                        )
                        all_results.append(result)
            except Exception as exc:  # noqa: BLE001
                boot_failures.append(
                    {
                        "max_num_seqs": max_num_seqs,
                        "port": port,
                        "error": str(exc),
                        "log_path": str(log_path),
                    }
                )
            finally:
                stop_gateway(gateway)

    rows = [row.to_dict() for row in all_results]
    candidates_no_error = [row for row in rows if row["err"] == 0]
    best_p95 = None
    if candidates_no_error:
        best_p95 = min(candidates_no_error, key=lambda row: row["p95_ms"])
    best_text_rps = None
    if candidates_no_error:
        best_text_rps = max(candidates_no_error, key=lambda row: row["text_rps"])

    report = {
        "schema": "omni_agent.embed_batch_sweep.v1",
        "generated_at_epoch": int(time.time()),
        "generated_at_iso": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "model": args.model,
        "endpoint_path": "/embed/batch",
        "agent_bin": str(args.agent_bin),
        "max_num_seqs": list(args.max_num_seqs),
        "batch_max_concurrency": args.batch_max_concurrency,
        "max_in_flight": args.max_in_flight,
        "shapes": [
            {
                "batch_size": shape.batch_size,
                "concurrency": shape.concurrency,
                "total_requests": shape.total_requests,
                "label": shape.label,
            }
            for shape in args.shapes
        ],
        "results": rows,
        "best_p95": best_p95,
        "best_text_rps": best_text_rps,
        "boot_failures": boot_failures,
        "status": "pass" if rows and not boot_failures else "partial" if rows else "fail",
    }

    report_json.write_text(
        json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )
    report_md.write_text(render_markdown(report), encoding="utf-8")
    latest_json.write_text(report_json.read_text(encoding="utf-8"), encoding="utf-8")
    latest_md.write_text(report_md.read_text(encoding="utf-8"), encoding="utf-8")

    print(report_json)
    print(report_md)
    print(latest_json)
    print(latest_md)
    if report["status"] == "fail":
        return 1
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Sweep omni-agent mistral-sdk /embed/batch performance"
    )
    parser.add_argument("--agent-bin", type=Path, default=DEFAULT_AGENT_BIN)
    parser.add_argument("--report-dir", type=Path, default=DEFAULT_REPORT_DIR)
    parser.add_argument("--host", default=DEFAULT_HOST)
    parser.add_argument("--port-base", type=int, default=DEFAULT_PORT_BASE)
    parser.add_argument("--model", default=DEFAULT_MODEL)
    parser.add_argument("--health-timeout-secs", type=float, default=60.0)
    parser.add_argument("--request-timeout-secs", type=float, default=DEFAULT_TIMEOUT_SECS)
    parser.add_argument("--warmup-requests", type=int, default=2)
    parser.add_argument("--batch-max-concurrency", type=int, default=8)
    parser.add_argument("--max-in-flight", type=int, default=64)
    parser.add_argument(
        "--max-num-seqs",
        type=int,
        nargs="+",
        default=list(DEFAULT_MAX_NUM_SEQS),
        help="List of mistral sdk_embedding_max_num_seqs values.",
    )
    parser.add_argument(
        "--shape",
        dest="shapes",
        type=parse_shape,
        action="append",
        default=[],
        help="Batch shape as <batch>x<concurrency>x<total_requests>. Repeatable.",
    )
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    if not args.shapes:
        args.shapes = [
            BatchShape(8, 32, 320),
            BatchShape(16, 32, 320),
            BatchShape(32, 16, 256),
            BatchShape(64, 8, 128),
        ]
    return asyncio.run(run(args))


if __name__ == "__main__":
    raise SystemExit(main())
