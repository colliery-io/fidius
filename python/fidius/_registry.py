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
"""Registry of fidius-decorated methods.

The host's PyO3 dispatcher looks up callables by name via :func:`get_method`.
Plugin authors use :func:`method` (re-exported as ``fidius.method``) to
register them.
"""

from __future__ import annotations

from typing import Callable

# Registry keyed on (module_name, method_name). Module scoping matters
# because the host runs many plugins in one embedded interpreter and they
# share `sys.modules`; a global registry would force all loaded plugins
# to use distinct method names. Module scoping makes "two plugins both
# expose @method def process()" work correctly.
_REGISTRY: dict[tuple[str, str], Callable] = {}


def method(func: Callable) -> Callable:
    """Register *func* under its ``__name__`` as a fidius plugin method.

    The registry is scoped per-module: two plugins can both expose
    ``@method def process()`` without clashing. Raises :class:`ValueError`
    if the *same* module registers two functions with the same name.
    """
    module = getattr(func, "__module__", "<unknown>") or "<unknown>"
    name = func.__name__
    key = (module, name)
    if key in _REGISTRY:
        raise ValueError(
            f"fidius: method '{name}' is already registered in module "
            f"'{module}'. Method names must be unique within a plugin module."
        )
    _REGISTRY[key] = func
    return func


def get_method(name: str, module: str | None = None) -> Callable:
    """Look up a previously-registered method.

    If *module* is given, looks for the entry registered against that
    module specifically. If *module* is ``None``, searches across all
    modules and raises if the name is ambiguous (registered in more than
    one module). Raises :class:`KeyError` if unknown.
    """
    if module is not None:
        return _REGISTRY[(module, name)]
    matches = [v for (m, n), v in _REGISTRY.items() if n == name]
    if not matches:
        raise KeyError(name)
    if len(matches) > 1:
        raise KeyError(
            f"fidius: method '{name}' is registered in multiple modules; "
            "pass `module=` to disambiguate."
        )
    return matches[0]


def list_methods(module: str | None = None) -> list[str]:
    """Return the sorted list of registered method names.

    If *module* is given, restricts to that module's methods. Otherwise
    returns the union across all modules.
    """
    if module is not None:
        return sorted(n for (m, n) in _REGISTRY if m == module)
    return sorted({n for (_m, n) in _REGISTRY})


def reset_registry() -> None:
    """Clear the registry. Intended for use in tests; not part of the public API."""
    _REGISTRY.clear()
