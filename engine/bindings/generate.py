#!/usr/bin/env python3
"""Generate bindings for the Louder engine.

    python engine/bindings/generate.py                 # every language
    python engine/bindings/generate.py --lang python    # just one
    python engine/bindings/generate.py --list          # what there is
    python engine/bindings/generate.py --check         # what would change

The engine is read through `cbindgen`, which already knows how to describe its
own C boundary; nothing about the ABI is written down twice.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from louder_bindings.cli import main  # noqa: E402

if __name__ == "__main__":
    raise SystemExit(main())
