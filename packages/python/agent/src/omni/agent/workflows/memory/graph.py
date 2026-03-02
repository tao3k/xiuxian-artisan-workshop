from .nodes import recall_node, store_node, synthesize_node


class MemoryWorkflowApp:
    """Minimal async memory workflow runtime without external graph runtime dependency."""

    async def ainvoke(self, state: dict):
        mode = state.get("mode", "recall")
        merged = dict(state)

        if mode == "store":
            merged.update(await store_node(merged))
            return merged

        merged.update(await recall_node(merged))
        merged.update(await synthesize_node(merged))
        return merged


def build_memory_graph() -> MemoryWorkflowApp:
    """Build memory workflow runtime."""
    return MemoryWorkflowApp()
