@skill_command(name="commit")
def commit(msg: str) -> str:
    """Create commit."""
    return "ok"


@skill_command(name="status")
def status() -> str:
    """Show status."""
    return "ok"
