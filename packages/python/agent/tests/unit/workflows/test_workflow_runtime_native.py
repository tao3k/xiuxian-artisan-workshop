from __future__ import annotations

import pytest


def _single_step_plan() -> dict:
    return {
        "steps": [
            {
                "id": "1",
                "description": "run",
                "status": "pending",
                "result": "",
                "tool_calls": [],
            }
        ],
        "current_step_index": 0,
    }


@pytest.mark.asyncio
async def test_robust_task_runtime_interrupt_and_resume(monkeypatch: pytest.MonkeyPatch) -> None:
    from omni.agent.workflows.robust_task import graph as graph_module

    async def _discovery(_state: dict) -> dict:
        return {"last_thought": "d"}

    async def _clarify(_state: dict) -> dict:
        return {"status": "planning", "clarified_goal": "goal"}

    async def _plan(_state: dict) -> dict:
        return {"plan": _single_step_plan()}

    async def _execute(_state: dict) -> dict:
        return {"execution_history": ["step ok"], "status": "executing_check_next"}

    async def _review(_state: dict) -> dict:
        return {}

    async def _validate(_state: dict) -> dict:
        return {"status": "completed", "validation_result": {"is_valid": True}}

    async def _summary(_state: dict) -> dict:
        return {"final_summary": "done"}

    async def _reflect(_state: dict) -> dict:
        return {"retry_count": 1}

    monkeypatch.setattr(graph_module, "discovery_node", _discovery)
    monkeypatch.setattr(graph_module, "clarify_node", _clarify)
    monkeypatch.setattr(graph_module, "plan_node", _plan)
    monkeypatch.setattr(graph_module, "execute_node", _execute)
    monkeypatch.setattr(graph_module, "review_node", _review)
    monkeypatch.setattr(graph_module, "validate_node", _validate)
    monkeypatch.setattr(graph_module, "summary_node", _summary)
    monkeypatch.setattr(graph_module, "reflection_node", _reflect)
    monkeypatch.setattr(graph_module, "check_execution_progress", lambda _state: "validate")
    monkeypatch.setattr(graph_module, "advance_step", lambda state: {"plan": state["plan"]})

    app = graph_module.build_graph()
    thread = {"configurable": {"thread_id": "t-1"}}

    first_pass_nodes: list[str] = []
    async for event in app.astream({"user_request": "task"}, thread):
        first_pass_nodes.extend(event.keys())

    assert first_pass_nodes == ["discovery", "clarify", "plan", "execute"]
    snapshot = await app.aget_state(thread)
    assert snapshot.next == ("review",)

    app.update_state(thread, {"approval_status": "approved"})
    resume_nodes: list[str] = []
    async for event in app.astream(None, thread):
        resume_nodes.extend(event.keys())

    assert resume_nodes == ["review", "validate", "summary"]
    final_state = await app.aget_state(thread)
    assert final_state.next == ()
    assert final_state.values["final_summary"] == "done"


@pytest.mark.asyncio
async def test_robust_task_runtime_ainvoke_auto_approves(monkeypatch: pytest.MonkeyPatch) -> None:
    from omni.agent.workflows.robust_task import graph as graph_module

    async def _discovery(_state: dict) -> dict:
        return {}

    async def _clarify(_state: dict) -> dict:
        return {"status": "planning"}

    async def _plan(_state: dict) -> dict:
        return {"plan": _single_step_plan()}

    async def _execute(_state: dict) -> dict:
        return {"execution_history": ["ok"], "status": "executing_check_next"}

    async def _review(_state: dict) -> dict:
        return {}

    async def _validate(_state: dict) -> dict:
        return {"status": "completed", "validation_result": {"is_valid": True}}

    async def _summary(_state: dict) -> dict:
        return {"final_summary": "done"}

    async def _reflect(_state: dict) -> dict:
        return {}

    monkeypatch.setattr(graph_module, "discovery_node", _discovery)
    monkeypatch.setattr(graph_module, "clarify_node", _clarify)
    monkeypatch.setattr(graph_module, "plan_node", _plan)
    monkeypatch.setattr(graph_module, "execute_node", _execute)
    monkeypatch.setattr(graph_module, "review_node", _review)
    monkeypatch.setattr(graph_module, "validate_node", _validate)
    monkeypatch.setattr(graph_module, "summary_node", _summary)
    monkeypatch.setattr(graph_module, "reflection_node", _reflect)
    monkeypatch.setattr(graph_module, "check_execution_progress", lambda _state: "validate")

    app = graph_module.build_graph()
    out = await app.ainvoke({"user_request": "task"}, {"configurable": {"thread_id": "t-2"}})

    assert out["final_summary"] == "done"
    assert out["approval_status"] == "approved"


@pytest.mark.asyncio
async def test_memory_runtime_routes_by_mode(monkeypatch: pytest.MonkeyPatch) -> None:
    from omni.agent.workflows.memory import graph as memory_graph_module

    async def _recall(_state: dict) -> dict:
        return {"retrieved_docs": [{"content": "doc"}]}

    async def _synthesize(_state: dict) -> dict:
        return {"final_context": "ctx"}

    async def _store(_state: dict) -> dict:
        return {"storage_result": "stored"}

    monkeypatch.setattr(memory_graph_module, "recall_node", _recall)
    monkeypatch.setattr(memory_graph_module, "synthesize_node", _synthesize)
    monkeypatch.setattr(memory_graph_module, "store_node", _store)

    app = memory_graph_module.build_memory_graph()

    recall_result = await app.ainvoke({"query": "q", "mode": "recall"})
    assert recall_result["final_context"] == "ctx"
    assert recall_result["retrieved_docs"][0]["content"] == "doc"

    store_result = await app.ainvoke({"content": "x", "mode": "store"})
    assert store_result["storage_result"] == "stored"


@pytest.mark.asyncio
async def test_react_runtime_runs_without_external_graph_runtime(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    from omni.agent.core.omni.graph import workflow as workflow_module

    async def _think(state: dict) -> dict:
        step = state.get("step_count", 0) + 1
        if step == 1:
            return {"step_count": step, "tool_calls": [{"name": "x", "input": {}}]}
        return {"step_count": step, "tool_calls": [], "exit_reason": "completed"}

    async def _act(state: dict) -> dict:
        return {"tool_calls_count": state.get("tool_calls_count", 0) + 1}

    async def _reflect(_state: dict) -> dict:
        return {}

    monkeypatch.setattr(workflow_module, "think_node", _think)
    monkeypatch.setattr(workflow_module, "act_node", _act)
    monkeypatch.setattr(workflow_module, "reflect_node", _reflect)

    app = workflow_module.build_react_graph()
    result = await app.ainvoke({"step_count": 0, "tool_calls_count": 0, "tool_calls": []})

    assert result["step_count"] == 2
    assert result["tool_calls_count"] == 1
    assert result["exit_reason"] == "completed"
