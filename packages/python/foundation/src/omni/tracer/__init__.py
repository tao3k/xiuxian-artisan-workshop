"""
omni.tracer - UltraRAG-style execution tracing system

Fine-grained execution tracing for native workflows + MCP.

Provides:
- Step-by-step execution tracking
- Thinking content capture (LLM streaming)
- Memory pool for variable history ($var, memory_var conventions)
- Callback system for custom processing
- Workflow integration
- Pipeline configuration (YAML to native workflow app)
- Rich console UI with colored output

UltraRAG Memory Conventions:
- $variable: Read-only parameters (from parameter.yaml)
- variable: Global variables
- memory_variable: History-tracked variables

Usage:
    from omni.tracer import ExecutionTracer, TracingCallbackHandler
    from omni.tracer.ui import traced, TracedExecution

    tracer = ExecutionTracer(trace_id="session-123")
    handler = TracingCallbackHandler(tracer)

    # Use with workflow callbacks
    app = workflow

    # Or use context manager
    async with traced("my_task", trace_id="run_001") as t:
        t.set_param("$query", "...")
        step = t.start_step("planner", "NODE_START", {...})
        ...
"""

from .async_utils import DispatchMode, dispatch_coroutine
from .callbacks import CallbackManager, LoggingCallback, TracingCallback
from .composite_invoker import CompositeToolInvoker
from .engine import ExecutionTracer, traced_session
from .graphflow import run_graphflow_pipeline
from .interfaces import ExecutionStep, ExecutionTrace, MemoryPool, StepType
from .invoker_stack import create_default_invoker_stack
from .mcp_invoker import MCPToolClient, MCPToolInvoker
from .node_factory import (
    MappingToolInvoker,
    NoOpToolInvoker,
    ToolInvoker,
    create_pipeline_node,
)
from .pipeline_builder import PipelineWorkflowBuilder
from .pipeline_checkpoint import compile_workflow, create_in_memory_checkpointer
from .pipeline_runtime import (
    PipelineExecutor,
    create_pipeline_executor,
    create_workflow_from_pipeline,
    create_workflow_from_pipeline_with_defaults,
    create_workflow_from_yaml,
    load_pipeline,
)
from .pipeline_schema import (
    CheckpointerRuntimeConfig,
    InvokerRuntimeConfig,
    PipelineConfig,
    PipelineRuntimeConfig,
    PipelineState,
    RetrievalRuntimeConfig,
    StateRuntimeConfig,
    TracerRuntimeConfig,
)
from .retrieval_invoker import RetrievalToolInvoker
from .storage import InMemoryTraceStorage, TraceStorage
from .ui import (
    TracedExecution,
    console,
    print_error,
    print_execution_path,
    print_header,
    print_info,
    print_memory,
    print_param,
    print_step_end,
    print_step_start,
    print_success,
    print_thinking,
    print_trace_summary,
    traced,
)
from .workflow_events import TracingCallbackHandler, create_traced_app, stream_with_trace
from .xml import escape_xml, extract_attr, extract_tag

__version__ = "0.2.0"

__all__ = [
    "CallbackManager",
    "CheckpointerRuntimeConfig",
    "CompositeToolInvoker",
    "DispatchMode",
    "ExecutionStep",
    "ExecutionTrace",
    "ExecutionTracer",
    "InMemoryTraceStorage",
    "InvokerRuntimeConfig",
    "LoggingCallback",
    "MCPToolClient",
    "MCPToolInvoker",
    "MappingToolInvoker",
    "MemoryPool",
    "NoOpToolInvoker",
    "PipelineConfig",
    "PipelineExecutor",
    "PipelineRuntimeConfig",
    "PipelineState",
    "PipelineWorkflowBuilder",
    "RetrievalRuntimeConfig",
    "RetrievalToolInvoker",
    "StateRuntimeConfig",
    "StepType",
    "ToolInvoker",
    "TraceStorage",
    "TracedExecution",
    "TracerRuntimeConfig",
    "TracingCallback",
    "TracingCallbackHandler",
    "compile_workflow",
    "console",
    "create_default_invoker_stack",
    "create_in_memory_checkpointer",
    "create_pipeline_executor",
    "create_pipeline_node",
    "create_traced_app",
    "create_workflow_from_pipeline",
    "create_workflow_from_pipeline_with_defaults",
    "create_workflow_from_yaml",
    "dispatch_coroutine",
    "escape_xml",
    "extract_attr",
    "extract_tag",
    "load_pipeline",
    "print_error",
    "print_execution_path",
    "print_header",
    "print_info",
    "print_memory",
    "print_param",
    "print_step_end",
    "print_step_start",
    "print_success",
    "print_thinking",
    "print_trace_summary",
    "run_graphflow_pipeline",
    "stream_with_trace",
    "traced",
    "traced_session",
]
