import sys
from pathlib import Path

root_dir = Path(__file__).parent.parent
sys.path.insert(0, str(root_dir / "packages/python/agent/src"))
sys.path.insert(0, str(root_dir / "packages/python/core/src"))
sys.path.insert(0, str(root_dir / "packages/python/foundation/src"))
sys.path.insert(0, str(root_dir / "assets/skills"))

from git.scripts.rendering import render_template


def main():
    print("Testing template render...")
    try:
        res = render_template(
            "prepare_result.j2",
            has_staged=True,
            staged_files=["a", "b"],
            staged_file_count=2,
            scope_warning="",
            valid_scopes=["feat", "fix"],
            lefthook_summary="passed",
            lefthook_report="",
            diff_content="diff",
            diff_stat="2 files changed",
            wf_id="123",
            submodule_info="subs",
        )
        print("Length of result:", len(res))
        print("Result preview:", res[:100])
    except Exception as e:
        print("Error:", e)


if __name__ == "__main__":
    main()
