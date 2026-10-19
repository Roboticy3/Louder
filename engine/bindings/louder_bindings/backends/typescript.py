"""TypeScript bindings.

There is nothing to generate here beyond the declarations: the JavaScript
backend already writes a `.d.ts` beside every module it emits, and those files
*are* the TypeScript binding. This backend points at that output, adds the
compiler settings that keep it honest, and writes the file a TypeScript project
imports.
"""

from __future__ import annotations

from .base import Emitted
from .javascript import JavaScriptBackend


class TypeScriptBackend(JavaScriptBackend):
    name = "typescript"
    label = "TypeScript"
    tier = "deep"
    convenience = "full"
    extension = "d.ts"

    def build(self, out: Emitted) -> None:
        # The declarations only. The JavaScript these describe are written by
        # the JavaScript backend, into the directory beside this one, because
        # they are the same engine and there is no reason to emit them twice.
        out.add_lines("louder/index.d.ts", self.render_types())
        out.add_lines("louder/raw.d.ts", self.render_raw_types())
        out.add_lines("louder.d.ts", self.entry_declaration())
        out.add_lines("tsconfig.json", self.tsconfig())
        out.add_lines("README.md", self.render_readme())

    def entry_declaration(self) -> list[str]:
        return [
            "// Generated from the C header of the Louder engine.",
            "//",
            "// This re-exports the declarations that sit beside the JavaScript layer, so a",
            "// TypeScript project has one import to write rather than two.",
            "",
            'export * from "./louder/index.js";',
            'export { default } from "./louder/index.js";',
        ]

    def tsconfig(self) -> list[str]:
        return [
            "{",
            '  "compilerOptions": {',
            '    "target": "ES2022",',
            '    "module": "NodeNext",',
            '    "moduleResolution": "NodeNext",',
            '    "lib": ["ES2022", "DOM"],',
            '    "strict": true,',
            '    "noUncheckedIndexedAccess": true,',
            '    "exactOptionalPropertyTypes": true,',
            '    "declaration": true,',
            '    "emitDeclarationOnly": false,',
            '    "allowJs": true,',
            '    "checkJs": false,',
            "  },",
            '  "include": ["louder/**/*.d.ts", "louder.d.ts"]',
            "}",
        ]

    def render_readme(self) -> list[str]:
        return (
            [
                "# Louder, for TypeScript",
                "",
                "Generated from the C header of the `louder_rust` engine. Do not edit; run",
                "`python engine/bindings/generate.py` and this directory comes back.",
                "",
                "The declarations describe the JavaScript layer in the sibling `javascript/`",
                "directory, which is the same engine called through its WebAssembly build.",
                "There is no separate TypeScript implementation: the types here are the",
                "types, and the code behind them is the JavaScript.",
                "",
                "```ts",
                "import { Louder, type Event, LoadError } from \"./louder.js\";",
                "",
                "const engine = await Louder.load(wasmBytes);",
                "",
                "const status: LoadError = engine.loadConversation(\"chat\", document);",
                "if (status !== \"Ok\") {",
                "  throw new Error(status);",
                "}",
                "",
                "const actor: number = engine.addActor(\"chat\", 0, { speakAs: [\"A\"] });",
                "engine.choose(\"chat\", actor, 0);",
                "",
                "for (const event of engine.drainEvents()) {",
                "  console.log(event.kind, event.value);",
                "}",
                "```",
                "",
                "## What a caller gets",
                "",
                "- every entry point in `louder_rust/src/lib.rs`, with the types it takes",
                "  and returns",
                "- tagged unions as an interface with the tag and one optional member per",
                "  arm, rather than as a byte blob",
                "- bit flags as numbers, named by the constants the header declares",
                "- the `next_*` cursors as `Generator<number>`, so a walk reads as a loop",
                "",
                "## What was generated",
                "",
                "- " + self.counts(),
                "",
            ]
        )
