#!/usr/bin/env python3
"""Run distributed multi-process Qianji swarm workers against one shared session.

This script launches multiple independent `qianji` processes in parallel. Each worker gets:
- a unique `AGENT_ID`
- a role class via `AGENT_ROLE_CLASS`
- shared `VALKEY_URL`
- shared `session_id`

The goal is to validate distributed synchronization behavior through one shared checkpoint and
consensus backplane.
"""

from __future__ import annotations

import argparse
import json
import os
import selectors
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path

DEFAULT_VALKEY_URL = "redis://127.0.0.1:6379/0"
DEFAULT_CONTEXT = "{}"
DEFAULT_TIMEOUT_SECONDS = 300.0


@dataclass(frozen=True)
class AgentSpec:
    """One swarm worker identity and voting profile."""

    agent_id: str
    role_class: str
    weight: float = 1.0

    @property
    def label(self) -> str:
        """Stable display label for streaming process logs."""
        return f"{self.agent_id}/{self.role_class}"


def _project_root() -> Path:
    return Path(__file__).resolve().parents[1]


def _default_manifest_path(project_root: Path) -> Path:
    return (
        project_root
        / "packages/rust/crates/xiuxian-zhixing/resources/zhixing/skills/agenda-management"
        / "references/agenda_flow.toml"
    )


def default_agents() -> list[AgentSpec]:
    """Default worker profiles for agenda adversarial routing."""
    return [
        AgentSpec(agent_id="student_node_1", role_class="student", weight=1.0),
        AgentSpec(agent_id="steward_node_1", role_class="steward", weight=1.0),
        AgentSpec(agent_id="teacher_node_1", role_class="teacher", weight=1.0),
    ]


def parse_agent_spec(raw: str) -> AgentSpec:
    """Parse one `--agent` value.

    Accepted forms:
    - `agent_id:role_class`
    - `agent_id:role_class:weight`
    """

    parts = [part.strip() for part in raw.split(":")]
    if len(parts) not in {2, 3}:
        raise ValueError(f"invalid --agent '{raw}', expected 'agent_id:role_class[:weight]'")
    agent_id, role_class = parts[0], parts[1]
    if not agent_id:
        raise ValueError(f"invalid --agent '{raw}', agent_id is empty")
    if not role_class:
        raise ValueError(f"invalid --agent '{raw}', role_class is empty")
    if len(parts) == 2:
        return AgentSpec(agent_id=agent_id, role_class=role_class, weight=1.0)

    try:
        weight = float(parts[2])
    except ValueError as exc:
        raise ValueError(f"invalid --agent '{raw}', weight must be numeric") from exc
    if weight <= 0.0:
        raise ValueError(f"invalid --agent '{raw}', weight must be > 0")
    return AgentSpec(agent_id=agent_id, role_class=role_class, weight=weight)


def load_context_json(args: argparse.Namespace) -> str:
    """Resolve context JSON payload from CLI input."""
    if args.context_file:
        payload = Path(args.context_file).read_text(encoding="utf-8")
    elif args.context_json:
        payload = args.context_json
    else:
        payload = DEFAULT_CONTEXT
    # Validate now so child processes fail fast with clear error.
    json.loads(payload)
    return payload


def build_qianji_command(
    project_root: Path,
    manifest_path: Path,
    context_json: str,
    session_id: str,
    cargo_bin: str,
    features: str,
) -> list[str]:
    """Build one `cargo run` invocation for the qianji binary."""
    command = [
        cargo_bin,
        "run",
        "-p",
        "xiuxian-qianji",
        "--features",
        features,
        "--bin",
        "qianji",
        "--",
        str(project_root),
        str(manifest_path),
        context_json,
        session_id,
    ]
    return command


def resolve_agents(agent_flags: list[str]) -> list[AgentSpec]:
    """Resolve agent profiles from flags or defaults."""
    parsed = [parse_agent_spec(raw) for raw in agent_flags]
    return parsed if parsed else default_agents()


def stream_swarm_processes(
    workers: list[tuple[AgentSpec, subprocess.Popen[str]]],
    timeout_seconds: float,
) -> dict[str, int]:
    """Stream logs from workers until all exits or timeout."""
    selector = selectors.DefaultSelector()
    label_by_stdout: dict[object, str] = {}
    for spec, process in workers:
        if process.stdout is None:
            continue
        selector.register(process.stdout, selectors.EVENT_READ)
        label_by_stdout[process.stdout] = spec.label

    start = time.monotonic()
    while label_by_stdout:
        now = time.monotonic()
        if now - start > timeout_seconds:
            for _, process in workers:
                if process.poll() is None:
                    process.terminate()
            raise TimeoutError(f"swarm execution timed out after {timeout_seconds:.1f}s")

        events = selector.select(timeout=0.5)
        for key, _ in events:
            stream = key.fileobj
            label = label_by_stdout.get(stream, "unknown")
            line = stream.readline()
            if line:
                print(f"[{label}] {line.rstrip()}")
                continue
            selector.unregister(stream)
            label_by_stdout.pop(stream, None)

        for _spec, process in workers:
            if process.poll() is not None and process.stdout in label_by_stdout:
                stream = process.stdout
                if stream is not None:
                    selector.unregister(stream)
                    label_by_stdout.pop(stream, None)

    return {spec.label: int(process.wait()) for spec, process in workers}


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    """Parse CLI arguments."""
    parser = argparse.ArgumentParser(
        description="Launch multi-process Qianji swarm workers with shared Valkey session."
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        help="Workflow TOML path. Defaults to agenda flow in xiuxian-zhixing resources.",
    )
    parser.add_argument(
        "--context-json",
        type=str,
        default=None,
        help="Context JSON payload string for qianji input.",
    )
    parser.add_argument(
        "--context-file",
        type=Path,
        default=None,
        help="Path to a JSON file used as context input.",
    )
    parser.add_argument(
        "--session-id",
        type=str,
        default=None,
        help="Shared session id. Auto-generated when omitted.",
    )
    parser.add_argument(
        "--valkey-url",
        type=str,
        default=os.getenv("VALKEY_URL", DEFAULT_VALKEY_URL),
        help="Valkey endpoint. Default: env VALKEY_URL or redis://127.0.0.1:6379/0.",
    )
    parser.add_argument(
        "--agent",
        action="append",
        default=[],
        help="Worker spec: agent_id:role_class[:weight]. Repeat for multiple workers.",
    )
    parser.add_argument(
        "--cargo-bin",
        type=str,
        default="cargo",
        help="Cargo executable path.",
    )
    parser.add_argument(
        "--features",
        type=str,
        default="llm",
        help="Cargo features for xiuxian-qianji binary.",
    )
    parser.add_argument(
        "--timeout-seconds",
        type=float,
        default=DEFAULT_TIMEOUT_SECONDS,
        help=f"Hard timeout for full swarm run. Default: {DEFAULT_TIMEOUT_SECONDS:.0f}s.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print commands and environment only, do not launch processes.",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    root = _project_root()
    manifest_path = (args.manifest or _default_manifest_path(root)).resolve()
    if not manifest_path.exists():
        print(f"[ERROR] manifest not found: {manifest_path}", file=sys.stderr)
        return 1

    try:
        context_json = load_context_json(args)
    except Exception as error:
        print(f"[ERROR] invalid context JSON: {error}", file=sys.stderr)
        return 1

    session_id = args.session_id or f"swarm_{int(time.time() * 1000)}"
    agents = resolve_agents(args.agent)

    command = build_qianji_command(
        project_root=root,
        manifest_path=manifest_path,
        context_json=context_json,
        session_id=session_id,
        cargo_bin=args.cargo_bin,
        features=args.features,
    )

    print("=" * 80)
    print("Distributed Qianji Swarm")
    print(f"project_root : {root}")
    print(f"manifest     : {manifest_path}")
    print(f"session_id   : {session_id}")
    print(f"valkey_url   : {args.valkey_url}")
    print("agents       :")
    for spec in agents:
        print(f"  - {spec.agent_id} role={spec.role_class} weight={spec.weight}")
    print("command      :")
    print(f"  {' '.join(command)}")
    print("=" * 80)

    if args.dry_run:
        return 0

    workers: list[tuple[AgentSpec, subprocess.Popen[str]]] = []
    for spec in agents:
        env = os.environ.copy()
        env["VALKEY_URL"] = args.valkey_url
        env["AGENT_ID"] = spec.agent_id
        env["AGENT_ROLE_CLASS"] = spec.role_class
        env["AGENT_WEIGHT"] = str(spec.weight)

        process = subprocess.Popen(
            command,
            cwd=root,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        workers.append((spec, process))

    try:
        rc_by_label = stream_swarm_processes(workers, timeout_seconds=args.timeout_seconds)
    except TimeoutError as error:
        print(f"[ERROR] {error}", file=sys.stderr)
        return 1

    print("-" * 80)
    print("Swarm exit summary:")
    failed = False
    for label, rc in rc_by_label.items():
        mark = "OK" if rc == 0 else "FAIL"
        print(f"  {mark:<4} {label:<30} rc={rc}")
        failed = failed or rc != 0
    print("-" * 80)
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
