#!/usr/bin/env python3
"""Quality and budget argument sections for memory CI gate parser."""

from __future__ import annotations

from typing import Any


def add_quality_args(parser: Any) -> None:
    """Register quality threshold and warning budget arguments."""
    parser.add_argument("--min-planned-hits", type=int, default=10)
    parser.add_argument("--min-successful-corrections", type=int, default=3)
    parser.add_argument("--min-recall-credit-events", type=int, default=1)
    parser.add_argument("--min-quality-score", type=float, default=90.0)
    parser.add_argument(
        "--slow-response-min-duration-ms",
        type=int,
        default=20000,
        help="Minimum accepted total duration for evolution scenario (milliseconds).",
    )
    parser.add_argument(
        "--slow-response-long-step-ms",
        type=int,
        default=1200,
        help="Threshold used to classify a step as slow-response (milliseconds).",
    )
    parser.add_argument(
        "--slow-response-min-long-steps",
        type=int,
        default=1,
        help="Minimum number of slow-response steps expected in evolution scenario.",
    )
    parser.add_argument("--trace-min-quality-score", type=float, default=90.0)
    parser.add_argument("--trace-max-events", type=int, default=2000)
    parser.add_argument("--min-session-steps", type=int, default=20)
    parser.set_defaults(require_cross_group_step=True, require_mixed_batch_steps=True)
    parser.add_argument(
        "--no-require-cross-group-step",
        action="store_false",
        dest="require_cross_group_step",
    )
    parser.add_argument(
        "--no-require-mixed-batch-steps",
        action="store_false",
        dest="require_mixed_batch_steps",
    )
    parser.add_argument("--discover-cache-hit-p95-ms", type=float, default=15.0)
    parser.add_argument("--discover-cache-miss-p95-ms", type=float, default=80.0)
    parser.add_argument("--discover-cache-bench-iterations", type=int, default=12)
    parser.add_argument("--max-mcp-call-waiting-events", type=int, default=0)
    parser.add_argument("--max-mcp-connect-waiting-events", type=int, default=0)
    parser.add_argument("--max-mcp-waiting-events-total", type=int, default=0)
    parser.add_argument("--max-memory-stream-read-failed-events", type=int, default=0)
    parser.add_argument("--max-embedding-timeout-fallback-turns", type=int, default=0)
    parser.add_argument("--max-embedding-cooldown-fallback-turns", type=int, default=0)
    parser.add_argument("--max-embedding-unavailable-fallback-turns", type=int, default=0)
    parser.add_argument("--max-embedding-fallback-turns-total", type=int, default=0)
