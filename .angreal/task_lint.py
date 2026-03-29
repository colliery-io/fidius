import angreal
import subprocess
import sys


@angreal.command(name="lint", about="Run formatting and lint checks")
def task_lint():
    """Run cargo fmt check and clippy across the workspace."""
    fmt = subprocess.run(["cargo", "fmt", "--all", "--check"])
    if fmt.returncode != 0:
        print("Formatting issues found. Run 'cargo fmt --all' to fix.")
        sys.exit(fmt.returncode)

    clippy = subprocess.run(["cargo", "clippy", "--workspace", "--", "-D", "warnings"])
    sys.exit(clippy.returncode)
