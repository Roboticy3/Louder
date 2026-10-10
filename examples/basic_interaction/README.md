# Bare Minimum

Two perspectives on one conversation, side by side in a single terminal.

Freya is on the left and chooses with `1`..`9`. Benjamin is on the right and
chooses with `q`..`p`. `ESC` quits. Each side only ever sees the hubs that are
visible to it, so the columns differ, and a side with nothing to decide says
who does.

## Running

```
build_and_run.ps1
```

Or by hand:

```
g++ -std=c++17 -O2 -I . main.cpp engine_bridge.cpp dialogue_view.cpp screen.cpp -o basic_interaction.exe louder_rust.dll
.\basic_interaction.exe
```

The conversation and speaker config default to `jail.json` and `speakers.json`
beside the executable, and both paths can be given as arguments.

## What it demonstrates

Engine side, from the spec in `docs/docs/examples/basic_interaction.md`:
  - Loading a conversation and a speaker config from absolute paths.
  - Starting a conversation from the hub tagged `start`.
  - Two actors on one conversation, each speaking as one character.
  - Relaying choices to the engine and reading back what the engine decided.
  - Per-character visibility, which is what makes the two columns differ.

Interface side:
  - Showing speaker and text.
  - Showing choices, keyed per column.
  - Relaying the player's key to the engine.
  - Showing "X is thinking..." in the column that has nothing to decide.
  - Aliasing: Benjamin is displayed as `REDACTED` to both parties, while the
    engine still reports `Benjamin` as the speaker. The substitution belongs to
    the interface, not the engine.

## The subtlety worth noticing

A choice index in the UI is **not** the index the engine wants. The engine
indexes choices by their position in the hub's full choice list and hides the
ones this actor may not take, so the visible list is usually shorter and its
indices do not start at zero. The interface numbers its visible list from 1 and
maps each key through to the engine index in
`DialogueView::handle_key`, in `dialogue_view.cpp`.

## Layout

| File | Purpose |
| --- | --- |
| `engine_bridge.h/.cpp` | Thin wrapper over the C ABI. Strings become ids here and only turn back into text at the edges. |
| `dialogue_view.h/.cpp` | The interface logic: events in, a description of both columns out. No terminal, no engine calls it does not need. |
| `screen.h/.cpp` | Console only: sizing, wrapping, colour, key reading. |
| `main.cpp` | Wiring: load, add actors, loop. |
| `interface_test.cpp` | Interface tests. Drive the real engine and the real view, never a terminal. |
| `dialogue_test.cpp` | Dialogue logic tests: the engine's flow rules, exhaustively. |

## Data files, and why they are not the editor's

`jail.json` and `speakers.json` here are copies of `editor/demos/conversations/jail.json`
and `editor/demos/speakers/test.json`, corrected to the schema in
`docs/docs/editor/schema.md`, which the engine implements. The editor's own
output does not currently conform, so it cannot be loaded. The corrections:

| Editor writes | Schema and engine expect | Fix applied |
| --- | --- | --- |
| hub `"auto": true` | `flags` bit 0 (`HubFlags_AUTO`) | `"flags": 1` on hubs 27 and 35 |
| hub `"_private": true` | `recipients`, the only privacy mechanism | dropped; hub 26 already carries `"recipients": "Benjamin"` |
| speaker `"speaker": "X"` | `"name": "X"` | renamed |
| speaker `"public": true` | `"flags"` bit 0 (`SpeakerFlags_PUBLIC`) | `"flags": 1` or `"flags": 0` |
| speaker `"relationships": [...]` | `"listening"`, newline separated | joined with newlines |
| `"conversation_id"`, `"position_offset:*"` on speakers | not in the schema | dropped; they were editor-only layout |
| `"camera"` records | accepted and ignored | dropped |

Editor-only concepts with no engine meaning, therefore also dropped:
`ownable`, `color`, `voice`, `description`.

Everything else is unchanged from the editor's files, so once the editor is
fixed these copies can be deleted and the demo pointed at the real output.