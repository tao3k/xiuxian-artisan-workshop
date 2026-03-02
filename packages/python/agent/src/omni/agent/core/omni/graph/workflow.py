from .nodes import act_node, reflect_node, think_node


class ReactWorkflowApp:
    """Simple ReAct loop runtime without external graph runtime dependency."""

    async def ainvoke(self, state: dict, max_iterations: int = 24) -> dict:
        merged = dict(state)

        for _ in range(max_iterations):
            merged.update(await think_node(merged))
            if merged.get("exit_reason") or not merged.get("tool_calls"):
                break

            merged.update(await act_node(merged))
            merged.update(await reflect_node(merged))
            if merged.get("exit_reason"):
                break

        return merged


def build_react_graph() -> ReactWorkflowApp:
    """Build ReAct runtime app."""
    return ReactWorkflowApp()
