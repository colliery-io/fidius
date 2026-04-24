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
"""Unit tests for the fidius Python SDK module.

Tests run with the parent of the ``fidius/`` package on ``sys.path`` —
the same way a vendored install would be loaded by the host.
"""

import sys
from pathlib import Path

# Put the parent of fidius/ on sys.path so `import fidius` works without
# installing the package — mirrors the vendored-load pattern.
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import pytest  # noqa: E402

from fidius import PluginError, get_method, list_methods, method, reset_registry  # noqa: E402


@pytest.fixture(autouse=True)
def _isolate_registry():
    """Each test gets a clean registry so order doesn't matter."""
    reset_registry()
    yield
    reset_registry()


def test_method_registers_under_function_name():
    @method
    def greet(name):
        return f"hi {name}"

    assert "greet" in list_methods()
    assert get_method("greet")("alice") == "hi alice"


def test_decorator_returns_function_unchanged():
    def fn():
        return 42

    decorated = method(fn)
    assert decorated is fn
    assert get_method("fn")() == 42


def test_multiple_methods_in_one_module():
    @method
    def a():
        return "a"

    @method
    def b():
        return "b"

    @method
    def c():
        return "c"

    assert list_methods() == ["a", "b", "c"]


def test_duplicate_registration_raises():
    @method
    def dup():
        return 1

    with pytest.raises(ValueError, match="already registered"):

        @method
        def dup():  # noqa: F811 — intentional shadow
            return 2


def test_get_method_unknown_raises_keyerror():
    with pytest.raises(KeyError):
        get_method("does_not_exist")


def test_plugin_error_carries_code_message_details():
    err = PluginError("BAD_INPUT", "missing field", details={"field": "name"})
    assert err.code == "BAD_INPUT"
    assert err.message == "missing field"
    assert err.details == {"field": "name"}
    # str(exc) returns the message — useful for downstream `str(...)` callers.
    assert str(err) == "missing field"


def test_plugin_error_details_optional():
    err = PluginError("NOPE", "no")
    assert err.details is None


def test_module_importable_from_vendor_layout(tmp_path):
    """Simulate the vendored-load pattern: copy fidius/ into a temp dir,
    add it to sys.path, and verify import + decoration still work.
    """
    import shutil

    src = Path(__file__).resolve().parent.parent / "fidius"
    vendor = tmp_path / "vendor"
    vendor.mkdir()
    shutil.copytree(src, vendor / "fidius")

    # Drop our existing import so we can reload from the vendored copy.
    for mod in list(sys.modules):
        if mod == "fidius" or mod.startswith("fidius."):
            del sys.modules[mod]

    sys.path.insert(0, str(vendor))
    try:
        import fidius as vendored_fidius

        @vendored_fidius.method
        def from_vendor():
            return "ok"

        assert vendored_fidius.get_method("from_vendor")() == "ok"
    finally:
        sys.path.remove(str(vendor))
        for mod in list(sys.modules):
            if mod == "fidius" or mod.startswith("fidius."):
                del sys.modules[mod]
