"""The languages the generator knows how to write bindings for.

Adding a language means adding a module here and listing it in `registry`.
Nothing else in the generator changes: a backend is handed the model and
returns files.
"""

from __future__ import annotations

from .base import Backend, Emitted, Options

__all__ = ["Backend", "Emitted", "Options"]
