import angreal
import subprocess
import sys


@angreal.command(name="test", about="Run the test suite")
@angreal.argument(name="release", long="release", takes_value=False, help="Run tests in release mode (bincode wire format)")
def task_test(release=False):
    """Run cargo tests across the workspace."""
    cmd = ["cargo", "test", "--workspace"]
    if release:
        cmd.append("--release")

    result = subprocess.run(cmd)
    sys.exit(result.returncode)
