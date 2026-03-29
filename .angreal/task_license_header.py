import angreal
import subprocess
import sys
import os

HEADER = """\
// Copyright 2026 Colliery, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

"""

HEADER_MARKER = "Copyright 2026 Colliery, Inc."


def find_rs_files():
    """Find all .rs files not in target directories."""
    result = []
    for root, dirs, files in os.walk("."):
        dirs[:] = [d for d in dirs if d != "target" and d != ".metis"]
        for f in files:
            if f.endswith(".rs"):
                result.append(os.path.join(root, f))
    return result


@angreal.command(name="license-header", about="Add or check license headers on .rs files")
@angreal.argument(name="check", long="check", takes_value=False, help="Check only, don't modify files")
def task_license_header(check=False):
    """Add Colliery Apache 2.0 license header to all .rs files."""
    files = find_rs_files()
    missing = []

    for path in files:
        with open(path, "r") as f:
            content = f.read()

        if HEADER_MARKER in content:
            continue

        missing.append(path)

        if not check:
            with open(path, "w") as f:
                f.write(HEADER + content)
            print(f"  Added header: {path}")

    if check and missing:
        print(f"Files missing license header ({len(missing)}):")
        for path in missing:
            print(f"  {path}")
        sys.exit(1)
    elif not check:
        print(f"Added header to {len(missing)} files.")
    else:
        print("All files have license headers.")
