"""MCP embedding client for CLI commands.

When the user runs `omni mcp --port <configured-port>` in the background, route test
and other CLI flows can use the already-warm embedding process instead of cold-starting
local embedding in each CLI invocation.

Supported endpoints:
- MCP SSE server: POST to `/messages/`
- MCP fast path: POST to `/embed` or `/embed/batch`
- Dedicated embedding HTTP server: POST to `/embed` or `/embed/batch` (default: 18501)
"""

from __future__ import annotations

import asyncio
import json
import logging
import socket
import threading
import time
from contextlib import suppress
from typing import TYPE_CHECKING
from urllib.parse import urlparse

if TYPE_CHECKING:
    from collections.abc import Awaitable, Callable

# Timeout for probe (detect whether MCP responds); keep short when no server is up.
PROBE_TIMEOUT_S = 3.0
# Timeout for actual embed requests.
REQUEST_TIMEOUT_S = 30.0
MCP_EMBED_CLIENT_TIMEOUT_S = 5.0

EMBEDDING_HTTP_PORT = 18501
_MCP_HTTP_EMBED_PATHS = ("/embed", "/embed/batch")
_MCP_EMBED_TERMINAL_CODES = {
    "embedding_timeout",
    "embedding_unavailable",
    "embedding_overloaded",
}
_MCP_EMBED_TERMINAL_MESSAGE_MARKERS = (
    "embedding timed out",
    "embedding timeout",
    "embedding unavailable",
    "embedding queue saturated",
    "tool_embedding_timeout",
)
_MCP_EMBED_UNAVAILABLE_COOLDOWN_S = 20.0
_MCP_EMBED_UNAVAILABLE_UNTIL: dict[int, float] = {}
_SHARED_HTTP_CLIENT = None
_SHARED_HTTP_CLIENT_LOCK = threading.Lock()
logger = logging.getLogger("omni.agent.cli.mcp_embed")


class McpEmbedUnavailable(RuntimeError):
    """Raised when MCP is reachable but upstream embedding is unavailable."""


def _mcp_embed_unavailable_cooldown_secs() -> float:
    """Cooldown window after terminal MCP embed failures."""
    try:
        from omni.foundation.config.settings import get_setting

        value = get_setting("mcp.embed_unavailable_cooldown_secs")
    except Exception:
        value = None
    try:
        parsed = float(value) if value is not None else _MCP_EMBED_UNAVAILABLE_COOLDOWN_S
    except Exception:
        return _MCP_EMBED_UNAVAILABLE_COOLDOWN_S
    return parsed if parsed > 0 else _MCP_EMBED_UNAVAILABLE_COOLDOWN_S


def _mcp_embed_client_timeout_secs() -> float:
    """Client timeout budget for MCP embed calls (separate from server-side timeout)."""
    try:
        from omni.foundation.config.settings import get_setting

        value = get_setting("mcp.embed_client_timeout_secs")
    except Exception:
        value = None
    try:
        parsed = float(value) if value is not None else MCP_EMBED_CLIENT_TIMEOUT_S
    except Exception:
        return MCP_EMBED_CLIENT_TIMEOUT_S
    return parsed if parsed > 0 else MCP_EMBED_CLIENT_TIMEOUT_S


def _mcp_embed_cooldown_remaining(port: int) -> float:
    """Seconds remaining before MCP embed should be retried for a port."""
    until = _MCP_EMBED_UNAVAILABLE_UNTIL.get(port, 0.0)
    remaining = until - time.monotonic()
    return remaining if remaining > 0 else 0.0


def _mark_mcp_embed_unavailable(port: int) -> None:
    """Mark MCP embed backend as temporarily unavailable for this port."""
    cooldown = _mcp_embed_unavailable_cooldown_secs()
    _MCP_EMBED_UNAVAILABLE_UNTIL[port] = time.monotonic() + cooldown


def _get_shared_http_client():
    """Get or create a process-level shared AsyncClient for local embedding calls."""
    global _SHARED_HTTP_CLIENT
    try:
        import httpx
    except Exception:
        return None

    with _SHARED_HTTP_CLIENT_LOCK:
        if _SHARED_HTTP_CLIENT is None or getattr(_SHARED_HTTP_CLIENT, "is_closed", False):
            _SHARED_HTTP_CLIENT = httpx.AsyncClient(timeout=REQUEST_TIMEOUT_S)
        return _SHARED_HTTP_CLIENT


async def close_shared_http_client() -> None:
    """Close process-level shared HTTP client if initialized."""
    global _SHARED_HTTP_CLIENT
    with _SHARED_HTTP_CLIENT_LOCK:
        client = _SHARED_HTTP_CLIENT
        _SHARED_HTTP_CLIENT = None
    if client is None:
        return
    with suppress(Exception):
        await client.aclose()


def _coerce_port(value: object) -> int | None:
    """Parse a candidate port into an integer in valid TCP range."""
    try:
        port = int(value) if isinstance(value, (int, float)) else int(str(value).strip())
    except Exception:
        return None
    if 0 < port < 65536:
        return port
    return None


def _port_from_url(url: object) -> int | None:
    """Extract TCP port from URL, handling implicit defaults."""
    if not isinstance(url, str) or not url.strip():
        return None
    try:
        parsed = urlparse(url)
    except Exception:
        return None
    if parsed.port is not None:
        return _coerce_port(parsed.port)
    scheme = (parsed.scheme or "").lower()
    if scheme == "http":
        return 80
    if scheme == "https":
        return 443
    return None


def _get_candidate_ports() -> list[int]:
    """Ports to try for MCP embedding, in order, derived strictly from config."""
    try:
        from omni.foundation.config.settings import get_setting

        candidates: list[int] = []
        preferred = _coerce_port(get_setting("mcp.preferred_embed_port"))
        if preferred is not None:
            candidates.append(preferred)

        client_url_port = _port_from_url(get_setting("embedding.client_url"))
        if client_url_port is not None:
            candidates.append(client_url_port)

        # Preserve order and deduplicate.
        deduped: list[int] = []
        for candidate in candidates:
            if candidate not in deduped:
                deduped.append(candidate)
        return deduped
    except Exception:
        return []


def _mcp_paths_for_port(port: int) -> tuple[str, ...]:
    """Return MCP JSON-RPC HTTP paths in preferred order for modern MCP servers."""
    return ("/messages/", "/mcp", "/")


def _extract_embed_error_code(payload: object) -> str:
    """Extract lower-cased embedding error code from HTTP/MCP payload."""
    if not isinstance(payload, dict):
        return ""
    direct_code = payload.get("code")
    if isinstance(direct_code, str):
        return direct_code.strip().lower()
    nested = payload.get("error")
    if isinstance(nested, dict):
        nested_code = nested.get("code")
        if isinstance(nested_code, str):
            return nested_code.strip().lower()
    return ""


def _extract_embed_error_message(payload: object) -> str:
    """Extract lower-cased embedding error message from HTTP/MCP payload."""
    if not isinstance(payload, dict):
        return ""
    direct_error = payload.get("error")
    if isinstance(direct_error, str):
        return direct_error.strip().lower()
    nested = payload.get("error")
    if isinstance(nested, dict):
        nested_message = nested.get("message")
        if isinstance(nested_message, str):
            return nested_message.strip().lower()
    return ""


def _is_terminal_embedding_failure(status_code: int, payload: object) -> bool:
    """Return True when response means MCP embed is reachable but backend unavailable."""
    code = _extract_embed_error_code(payload)
    if code in _MCP_EMBED_TERMINAL_CODES:
        return True
    message = _extract_embed_error_message(payload)
    if any(marker in message for marker in _MCP_EMBED_TERMINAL_MESSAGE_MARKERS):
        return True
    return status_code in {503, 504} and bool(message)


def _is_timeout_exception(exc: Exception) -> bool:
    """Best-effort timeout classification without hard dependency on httpx types."""
    class_name = exc.__class__.__name__.lower()
    if "timeout" in class_name:
        return True
    message = str(exc).lower()
    return "timed out" in message or "timeout" in message


def _format_exception(exc: Exception) -> str:
    """Readable exception text for logs/error messages."""
    message = str(exc).strip()
    if message:
        return message
    return exc.__class__.__name__


async def embed_via_mcp(
    texts: list[str],
    port: int,
    path: str = "/messages/",
    *,
    try_http_fast_path: bool = True,
    request_timeout_s: float = REQUEST_TIMEOUT_S,
) -> list[list[float]] | None:
    """Get embeddings via MCP server JSON-RPC path.

    Returns None if the server is unavailable or the response is invalid.
    """
    # Fast path for MCP transports: direct embedding endpoint avoids JSON-RPC tool envelope.
    if path == "/messages/" and try_http_fast_path:
        vectors = await embed_via_mcp_http(
            texts,
            port=port,
            request_timeout_s=request_timeout_s,
        )
        if vectors is not None:
            logger.debug(
                "MCP direct embed fast path selected",
                extra={"port": port, "path": path, "request_timeout_s": request_timeout_s},
            )
            return vectors

    url = f"http://127.0.0.1:{port}{path}"
    try:
        client = _get_shared_http_client()
        if client is None:
            return None

        request = {
            "jsonrpc": "2.0",
            "id": "mcp-embed",
            "method": "tools/call",
            "params": {
                "name": "embedding.embed_texts",
                "arguments": {"texts": texts},
            },
        }
        response = await client.post(
            url,
            json=request,
            headers={"Content-Type": "application/json"},
            timeout=request_timeout_s,
        )
        payload: object = {}
        with suppress(Exception):
            payload = response.json()
        if _is_terminal_embedding_failure(response.status_code, payload):
            raise McpEmbedUnavailable(
                f"status={response.status_code}, code={_extract_embed_error_code(payload) or 'unknown'}"
            )
        response.raise_for_status()
        result = payload if isinstance(payload, dict) else response.json()

        if result.get("result"):
            content = result["result"].get("content", [])
            if content and isinstance(content, list):
                text_content = content[0].get("text", "")
                if text_content:
                    data = json.loads(text_content)
                    if data.get("success"):
                        return data.get("vectors")
        if _is_terminal_embedding_failure(response.status_code, result):
            raise McpEmbedUnavailable(
                f"status={response.status_code}, code={_extract_embed_error_code(result) or 'unknown'}"
            )
        logger.debug(
            "MCP embed tool call returned no vectors",
            extra={"port": port, "path": path, "status_code": response.status_code},
        )
        return None
    except McpEmbedUnavailable:
        raise
    except Exception as exc:
        if _is_timeout_exception(exc):
            raise McpEmbedUnavailable(f"request_timeout: {_format_exception(exc)}") from exc
        logger.debug(
            "MCP embed tool call failed",
            extra={"port": port, "path": path, "error": str(exc), "url": url},
        )
        return None


async def embed_via_mcp_http(
    texts: list[str],
    port: int,
    request_timeout_s: float = REQUEST_TIMEOUT_S,
) -> list[list[float]] | None:
    """Get embeddings via MCP SSE direct HTTP endpoint (/embed or /embed/batch)."""
    for path in _MCP_HTTP_EMBED_PATHS:
        url = f"http://127.0.0.1:{port}{path}"
        try:
            client = _get_shared_http_client()
            if client is None:
                return None
            response = await client.post(
                url,
                json={"texts": texts},
                timeout=request_timeout_s,
            )
            if response.status_code == 200:
                data = response.json()
                vectors = data.get("vectors")
                if isinstance(vectors, list):
                    logger.debug(
                        "MCP direct embed endpoint selected",
                        extra={"port": port, "path": path, "status_code": response.status_code},
                    )
                    return vectors
            else:
                payload: object = {}
                with suppress(Exception):
                    payload = response.json()
                if _is_terminal_embedding_failure(response.status_code, payload):
                    raise McpEmbedUnavailable(
                        f"status={response.status_code}, code={_extract_embed_error_code(payload) or 'unknown'}"
                    )
                logger.debug(
                    "MCP direct embed endpoint not ready",
                    extra={"port": port, "path": path, "status_code": response.status_code},
                )
        except McpEmbedUnavailable:
            raise
        except Exception as exc:
            if _is_timeout_exception(exc):
                raise McpEmbedUnavailable(f"request_timeout: {_format_exception(exc)}") from exc
            logger.debug(
                "MCP direct embed endpoint failed",
                extra={"port": port, "path": path, "error": str(exc), "url": url},
            )
    return None


async def embed_via_http(
    texts: list[str],
    port: int = EMBEDDING_HTTP_PORT,
    request_timeout_s: float = REQUEST_TIMEOUT_S,
) -> list[list[float]] | None:
    """Get embeddings via dedicated embedding HTTP server (/embed or /embed/batch)."""
    for path in _MCP_HTTP_EMBED_PATHS:
        url = f"http://127.0.0.1:{port}{path}"
        try:
            client = _get_shared_http_client()
            if client is None:
                return None
            response = await client.post(
                url,
                json={"texts": texts},
                timeout=request_timeout_s,
            )
            if response.status_code == 200:
                data = response.json()
                vectors = data.get("vectors")
                if isinstance(vectors, list):
                    logger.debug(
                        "Embedding HTTP endpoint selected",
                        extra={"port": port, "path": path, "status_code": response.status_code},
                    )
                    return vectors
            else:
                logger.debug(
                    "Embedding HTTP endpoint unavailable",
                    extra={"port": port, "path": path, "status_code": response.status_code},
                )
        except Exception as exc:
            logger.debug(
                "Embedding HTTP endpoint failed",
                extra={"port": port, "path": path, "error": str(exc), "url": url},
            )
    return None


async def detect_embedding_http_port() -> int:
    """Return embedding HTTP server port (18501) if it is up and healthy, else 0."""
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.settimeout(1)
    try:
        if sock.connect_ex(("127.0.0.1", EMBEDDING_HTTP_PORT)) != 0:
            return 0
    except Exception:
        return 0
    finally:
        sock.close()

    try:
        client = _get_shared_http_client()
        if client is None:
            return 0
        response = await client.get(f"http://127.0.0.1:{EMBEDDING_HTTP_PORT}/health", timeout=2.0)
        if response.status_code == 200:
            return EMBEDDING_HTTP_PORT
    except Exception:
        pass
    return 0


async def _mcp_health_ok(port: int) -> bool:
    """Return True when MCP SSE server responds healthy on /health."""
    try:
        client = _get_shared_http_client()
        if client is None:
            return False
        response = await client.get(
            f"http://127.0.0.1:{port}/health",
            timeout=PROBE_TIMEOUT_S,
        )
        if response.status_code != 200:
            return False
        data = response.json()
        return str(data.get("status", "")).lower() in {"healthy", "ok"}
    except Exception:
        return False


async def probe_mcp_embed_port(port: int) -> bool:
    """Return True if MCP is reachable on this port.

    Fast path uses GET /health so probe does not consume extra embed POST calls.
    Falls back to active embed probes for partial deployments.
    """
    if await _mcp_health_ok(port):
        return True

    vectors = await embed_via_mcp_http(
        ["[DETECT]"],
        port=port,
        request_timeout_s=PROBE_TIMEOUT_S,
    )
    if vectors is not None:
        return True

    for path in _mcp_paths_for_port(port):
        vectors = await embed_via_mcp(
            ["[DETECT]"],
            port=port,
            path=path,
            request_timeout_s=PROBE_TIMEOUT_S,
        )
        if vectors is not None:
            return True
    return False


async def detect_mcp_port(candidate_ports: list[int] | None = None) -> int:
    """Detect a working MCP/embedding port.

    Tries dedicated embedding HTTP server (18501) first, then each configured candidate port.
    Returns the first port that responds, or 0.
    """
    port = await detect_embedding_http_port()
    if port > 0:
        return port

    candidates = candidate_ports if candidate_ports is not None else _get_candidate_ports()
    for p in candidates:
        if await probe_mcp_embed_port(p):
            return p
    return 0


def make_mcp_embed_func(port: int) -> Callable[[list[str]], Awaitable[list[list[float]]]]:
    """Return an async embed function that uses MCP or embedding HTTP on the given port, with local fallback."""

    async def _embed(texts: list[str]) -> list[list[float]]:
        from omni.foundation.services.embedding import EmbeddingUnavailableError

        cooldown_remaining = _mcp_embed_cooldown_remaining(port)
        if cooldown_remaining > 0 and port != EMBEDDING_HTTP_PORT:
            raise EmbeddingUnavailableError(
                f"MCP embedding unavailable on port {port} "
                f"(cooldown_remaining_secs={cooldown_remaining:.1f})"
            )

        try:
            request_timeout_s = _mcp_embed_client_timeout_secs()
            if port == EMBEDDING_HTTP_PORT:
                vectors = await embed_via_http(texts, port)
                if vectors is not None:
                    return vectors
            else:
                vectors = await embed_via_mcp_http(
                    texts,
                    port=port,
                    request_timeout_s=request_timeout_s,
                )
                if vectors is not None:
                    return vectors

                for path in _mcp_paths_for_port(port):
                    vectors = await embed_via_mcp(
                        texts,
                        port,
                        path=path,
                        try_http_fast_path=False,
                        request_timeout_s=request_timeout_s,
                    )
                    if vectors is not None:
                        return vectors
        except McpEmbedUnavailable as exc:
            _mark_mcp_embed_unavailable(port)
            raise EmbeddingUnavailableError(
                f"MCP embedding unavailable on port {port}: {exc}"
            ) from exc
        from omni.foundation.services.embedding import get_embedding_service

        loop = asyncio.get_running_loop()
        return await loop.run_in_executor(None, lambda: get_embedding_service().embed_batch(texts))

    return _embed
