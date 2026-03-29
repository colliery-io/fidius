import angreal
import subprocess
import sys


@angreal.command(name="check", about="Run cargo check and clippy")
def task_check():
    """Run cargo check followed by clippy across the workspace."""
    check = subprocess.run(["cargo", "check", "--workspace"])
    if check.returncode != 0:
        sys.exit(check.returncode)

    clippy = subprocess.run(["cargo", "clippy", "--workspace", "--", "-D", "warnings"])
    sys.exit(clippy.returncode)
