# Copyright 2026 Colliery, Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import os
import subprocess
import sys

import angreal
from angreal.integrations.venv import venv_required


# Dedicated venv for the Python SDK test harness so we don't pollute the
# project-root .venv (which uv manages for other purposes). Lives next to
# the SDK source.
_PROJECT_ROOT = os.path.dirname(angreal.get_root())
_VENV_PATH = os.path.join(_PROJECT_ROOT, "python", ".venv")


@angreal.command(
    name="python-test",
    about="Run pytest against the Python SDK module",
)
@venv_required(_VENV_PATH, requirements=["pytest"])
def task_python_test():
    """Run the fidius Python SDK test suite inside a managed venv.

    @venv_required handles venv create + pip install pytest on first run
    and reuses it on subsequent runs. Tests live in python/tests/.
    """
    tests_dir = os.path.join(_PROJECT_ROOT, "python", "tests")

    result = subprocess.run(
        ["python", "-m", "pytest", tests_dir, "-v"],
    )
    sys.exit(result.returncode)
