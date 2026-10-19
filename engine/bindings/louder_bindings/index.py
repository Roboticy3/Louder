"""The table of contents for a run.

Written last, so it can say what was actually produced. A reader who opens the
output directory and finds twenty-eight languages in it should be able to tell,
in one screen, which of them are finished and which are declarations.
"""

from __future__ import annotations

from . import registry
from .ir import Abi

TIER_TEXT = {
    "deep": "a raw layer and a fully idiomatic one on top of it",
    "solid": "complete bindings that are compiled rather than interpreted",
    "broad": "complete bindings with a thin convenience layer",
    "long": "declarations only, for the tail of FFI-capable languages",
}


def render_index(abi: Abi, written: list[str]) -> str:
    grouped: dict[str, list[str]] = {}
    for name in written:
        entry = registry.REGISTRY.get(name)
        if entry is None:
            continue
        grouped.setdefault(entry.tier, []).append(entry.label)

    lines: list[str] = []
    lines.append("# Louder bindings")
    lines.append("")
    lines.append(
        f"Generated from the C header of the `louder_rust` engine: "
        f"{len(abi.funcs)} entry points, {len(abi.structs)} structs, "
        f"{len(abi.unions)} tagged unions, {len(abi.enums)} enums and "
        f"{len(abi.consts)} constants."
    )
    lines.append("")
    lines.append("Nothing in here is written by hand. Run `python generate.py` from")
    lines.append("`engine/bindings/` and every directory below comes back. The engine's")
    lines.append("own documentation is carried across, so the description of an entry")
    lines.append("point in any language is the description written next to it in Rust.")
    lines.append("")
    lines.append("## By tier")
    lines.append("")
    for tier in ("deep", "solid", "broad", "long"):
        labels = grouped.get(tier)
        if not labels:
            continue
        lines.append(f"### {tier.capitalize()} — {TIER_TEXT[tier]}")
        lines.append("")
        for label in labels:
            lines.append(f"- [{label}]({registry.by_label(label)}/)")
        lines.append("")

    lines.append("## What every binding gets")
    lines.append("")
    lines.append("Every one of these exposes all")
    lines.append(f"{len(abi.funcs)} functions in `louder_rust/src/lib.rs`, and with them the types")
    lines.append("those functions take and return:")
    lines.append("")
    lines.append("- tagged unions, as a tag plus a set of named bodies, never as a blob")
    lines.append("- bit flags, as the bits the header defines and not as a bare integer")
    lines.append("- the constants, with the names the engine gives them")
    lines.append("")
    lines.append("What differs is how much is wrapped on top. A deep binding takes `str`")
    lines.append("where the header takes a buffer and a length, hands a variant back as")
    lines.append("`int | float | str`, and turns the `next_*` cursor functions into")
    lines.append("iterators. A long one is the declarations and nothing else, which is")
    lines.append("the honest amount of work a language whose FFI is a foreign-function")
    lines.append("declaration can be handed.")
    lines.append("")
    lines.append("## Shared inputs")
    lines.append("")
    lines.append("- `abi/louder_rust.c.h` — the header every binding is derived from")
    lines.append("- `abi/louder_rust.wasm` — the engine built for WebAssembly, whose own")
    lines.append("  type section is what the JavaScript binding is written against")
    lines.append("")
    return "\n".join(lines) + "\n"
