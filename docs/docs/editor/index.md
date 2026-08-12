# Editor

The Louder editor is an open source graph editor for creating branching
dialogue trees. You build conversations by placing dialogue nodes on a
graph and connecting them to define the flow of conversation.

Each node holds a line of dialogue (or a choice prompt) with properties
like speaker name, text content, difficulty check settings, and branching
choices. When you save, the entire graph is exported as a single file
that can be shared with collaborators or used with the [engine](/engine/)
to run dialogue in your game.

## Shortcuts
!TODO! Most keybinds do not work while editing text. This should be fixed somehow. [28](https://github.com/Roboticy3/Louder/issues/28)

### Selection and Focus
!TODO! Focus changes based on how nodes are connected to each other is still not done. This should improve flow. [27](https://github.com/Roboticy3/Louder/issues/27)

| Shortcut | Action | Description |
|---|---|---|
| **Alt+A** | Create Node | Spawns a new dialogue node at the current mouse position. If you keep pressing Alt+A without moving the mouse, each new node is offset slightly so they don't stack on top of each other. |
| **Tab/Shift+Tab** | Next/Previous | Move to the next editable field in a node. If no node is selected or the last editable field of the current node is selected, a different node is selected. |
| **Up/Down** | Increase/Decrease | Change the value of a number spinner up or down by one. |

### Actions
| Shortcut | Action | Description |
|---|---|---|
| **Delete** | Delete | Removes all currently selected nodes from the graph. After deletion, the editor automatically selects a nearby node (see [Node Selection](#node-selection)). |

### Other
| Shortcut | Action | Description |
|---|---|---|
| **F3** | Debug Toggle | Toggles the on-screen debug overlay, which displays runtime messages. Messages auto-expire after a short duration. |

### File Actions

| Shortcut | Action | Description |
|---|---|---|
| **Ctrl+S** | Save | Saves the current conversation to its existing file path. If the conversation has never been saved before, this opens the Save As dialog instead. |
| **Ctrl+Shift+S** | Save As | Always opens the Save As dialog, letting you choose a new file name and location. |
| **Ctrl+O** | Load | Opens the file browser to load a previously saved conversation. The current graph is cleared before the loaded conversation is restored. |

### Mouse Actions

| Shortcut | Action | Description |
|---|---|---|
| **Scroll Wheel** | Zoom | Zooms in and out on the graph. The zoom level is saved with the conversation. |
| **Middle Mouse Drag** | Pan | Pans the graph view. The scroll offset is saved with the conversation. |
| **Right Mouse Drag** | Pan (alt) | Alternate panning method. |
| **Shift+Click** | Multi-Select | Adds or removes individual nodes from the current selection. |
| **Drag on Empty Space** | Box Select | Draws a rectangle that selects all nodes inside it. |
| **Drag Port-to-Port** | Connect | Creates a connection between two node ports. |
| **Right-Click Connection** | Disconnect | Right-clicking an existing connection line removes it. |

### Arrow Keys in Number Spinners

When a number spinner has focus (the choices count or difficulty check level),
**Up Arrow** increments the value and **Down Arrow** decrements it. This
works even when the spinner's text field is focused.

## Saving and Loading

The editor saves and loads conversations as self-contained files. This
keeps the format compact while preserving all the data you need.

### File Formats
!TODO! Add support for a more readable format [23](https://github.com/Roboticy3/Louder/issues/23)

Projects save out to .json in an object-based format. Each node and connection is treated as a separate object, with the hope that this will allow for easier modification later down the line. The actual node object in the editor ([hub.tscn](https://github.com/Roboticy3/Louder/blob/main/editor/entities/hub.tscn)) is composed of subnodes tagged as "hub_component", whose names are used as the names of fields in the output json, which then match up with their values. A hub with no data changed is also treated as "default". Empty text fields, unchecked checkboxes, and numbers set to 0 are ignored by the output json.

### What Gets Saved

When you save, the editor stores:

- **All nodes** -- Speaker name, dialogue text, difficulty check settings
  (enabled, skill, level), expressions, start node toggle, choice texts, and
  graph position.
- **All connections** -- Which nodes are connected, and which choice each
  connection originates from.
- **Camera state** -- The graph's scroll position and zoom level, so the
  view is restored when you reopen the file.

### Exporting

!TODO! Set up the engine. Goal: Any saved json can be exported and fed as text into the [engine](/engine/) to run dialogue trees in your game. [29](https://github.com/Roboticy3/Louder/issues/29)

