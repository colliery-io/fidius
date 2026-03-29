import angreal
import subprocess
import sys


@angreal.command(name="build", about="Build the workspace")
@angreal.argument(name="release", long="release", takes_value=False, help="Build in release mode")
def task_build(release=False):
    """Build all crates in the workspace."""
    cmd = ["cargo", "build", "--workspace"]
    if release:
        cmd.append("--release")

    result = subprocess.run(cmd)
    sys.exit(result.returncode)
