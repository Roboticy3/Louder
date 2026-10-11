# Bare Minimum

Two perspectives on one conversation, side by side in a single terminal.

Freya is on the left and chooses with `1`..`9`. Benjamin is on the right and
chooses with `q`..`p`. `ESC` quits. Each side only ever sees the hubs that are
visible to it, so the columns differ, and a side with nothing to decide says
who does.

## Running

```
build_and_test.ps1
```

Or by hand:

```
g++ -std=c++17 -O2 -I . main.cpp dialogue.cpp terminal.cpp -o basic_interaction.exe louder_rust.dll
.\basic_interaction.exe
```

The conversation and speaker config default to `jail.json` and `speakers.json`
beside the executable, and both paths can be given as arguments.

## What it demonstrates

The point of the example is the engine, so `louder_rust.h` is called directly.
There is deliberately no wrapper object around the C ABI: the engine's own names
and ids are used as they are, and what this example adds is only what the engine
does not do.

Engine side, from the spec in `docs/docs/examples/basic_interaction.md`:
  - Loading a conversation and a speaker config from absolute paths.
  - Starting a conversation from the hub tagged `start`, rather than assuming the
    document numbers its hubs from zero.
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
maps each key through to the engine index in `View::press`, in `dialogue.cpp`.

Hub 26 is where this bites in the demo conversation: it carries five raw choices
and offers Benjamin four of them, and it is private to Benjamin, so Freya's
column is offered none of it at all.

## Layout

| File | Purpose |
| --- | --- |
| `main.cpp` | The demo: load, add the two actors, loop. Every engine call it makes is in this file. |
| `dialogue.h/.cpp` | The two columns: which key belongs to which, what the engine says about each actor, and how it is laid out. |
| `terminal.h/.cpp` | The console, and nothing else. |
| `interface_test.cpp` | The interface's own rules. Engine rules are tested in `engine/tests/engine_test.cpp`. |

## Data files, and why they are not the editor's

`jail.json` and `speakers.json` here are copies of `editor/demos/conversations/jail.json`
and `editor/demos/speakers/test.json`, corrected to the schema in
`docs/docs/editor/schema.md`, which the engine implements. The editor's own
output does not currently conform, so it cannot be loaded. Run
`python make_demo_data.py` to regenerate them from the editor's files; the
corrections it makes are listed there and were:

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
