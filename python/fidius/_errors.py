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
"""Typed plugin errors.

Plugin code raises ``PluginError(code, message, details=...)`` for failures
that should reach the host as a structured error. The fidius-python loader
recognises this exception class specifically and round-trips the fields into
the host's :class:`fidius_core::PluginError`.

Other exceptions (``ValueError``, ``KeyError``, etc.) still surface as
plugin errors, but their ``code`` is set to ``"PYTHON_ERROR"`` and the
exception class name is folded into the message — they're treated as
unstructured failures rather than typed contract violations.
"""

from __future__ import annotations

from typing import Optional


class PluginError(Exception):
    """Structured plugin error that round-trips to the host with its fields intact.

    Parameters
    ----------
    code:
        Short machine-readable error code (e.g. ``"BAD_INPUT"``).
    message:
        Human-readable explanation.
    details:
        Optional dict of structured context. Must be JSON-serialisable.
    """

    def __init__(
        self,
        code: str,
        message: str,
        details: Optional[dict] = None,
    ) -> None:
        super().__init__(message)
        self.code = code
        self.message = message
        self.details = details

    def __repr__(self) -> str:  # pragma: no cover - cosmetic
        return f"PluginError(code={self.code!r}, message={self.message!r}, details={self.details!r})"
