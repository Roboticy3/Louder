# Editor

The Louder editor is an open source graph editor for creating branching
dialogue trees. You build conversations by placing dialogue nodes on a
graph and connecting them to define the flow of conversation.

Each node holds a line of dialogue (or a choice prompt) with properties
like speaker name, text content, difficulty check settings, and branching
choices. When you save, the entire graph is exported as a single file
that can be shared with collaborators or used with the [engine](/engine/)
to run dialogue in your game.

## Quick Actions

Keyboard shortcuts are the fastest way to work in the editor. Most actions
are single keystroke or two-key combos that work regardless of what field
has focus.

### Shortcuts
!TODO! Most keybinds do not work while editing text. This should be fixed somehow.

## Selection and Focus
!TODO! Focus changes based on how nodes are connected to each other is still not done. This should improve flow.

| Shortcut | Action | Description |
|---|---|---|
| **Alt+A** | Create Node | Spawns a new dialogue node at the current mouse position. If you keep pressing Alt+A without moving the mouse, each new node is offset slightly so they don't stack on top of each other. |
| **Tab/Shift+Tab** | Next/Previous | Move to the next editable field in a node. If no node is selected or the last editable field of the current node is selected, a different node is selected. |

## Actions
| Shortcut | Action | Description |
|---|---|---|
| **Delete** | Delete | Removes all currently selected nodes from the graph. After deletion, the editor automatically selects a nearby node (see [Node Selection](#node-selection)). |

## Other
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

### !TODO! Moving focus

Pressing Alt+

## Node Selection

The editor keeps the graph in a usable state by ensuring something useful is
always selected.

### Basics

- **Click** a node to select it.
- **Shift+Click** to add or remove nodes from the selection.
- **Drag on empty space** to box-select multiple nodes.
- Clicking any field inside a node (speaker, text, checkbox, etc.) also
  selects that node.

When a node is selected, the editor focuses the first editable field so you
can start typing right away.

### Auto-Selection on Startup

When the editor opens, it selects the first node in the graph so there is
always something selected on launch.

### Auto-Selection After Delete

Pressing **Delete** removes all selected nodes. After removal, the editor
picks a new node to select using a three-tier fallback:

1. **Connected node** -- Follows connections from or to the last deleted node
   and selects the first connected node that is still on the graph.
2. **Start node** -- If no connected node is found, selects whichever node
   has the "Start" toggle enabled.
3. **Any node** -- As a last resort, selects the first node remaining on the
   graph.

If there are no nodes left at all, nothing is selected.

### Auto-Selection on Create

When you press **Alt+A** to create a new node, it is immediately selected so
you can start editing its fields right away.

### Drag-to-Empty Creates a Node

If you drag a connection from a port into empty space, a new node is created
at the drop point and selected automatically. This is a quick way to extend
the conversation without reaching for Alt+A.

## Saving and Loading

The editor saves and loads conversations as self-contained files. This
keeps the format compact while preserving all the data you need.

### File Formats

The editor supports two file formats:

- **`.tres`** -- A human-readable, text-based format. Great for version
  control and manual inspection.
- **`.res`** -- A compact binary format. Smaller file size but not
  diff-friendly.

Both formats store the same data. The choice between them is a matter of
preference.

### Quick Save (Ctrl+S)

If the conversation has been saved before, **Ctrl+S** overwrites the existing
file immediately with no dialog.

If the conversation has never been saved, **Ctrl+S** opens the Save As dialog
so you can choose a file name and location.

### Save As (Ctrl+Shift+S)

Always opens the Save As dialog, even if the conversation already has a file
path. Use this to save a copy under a different name.

### What Gets Saved

When you save, the editor stores:

- **All nodes** -- Speaker name, dialogue text, difficulty check settings
  (enabled, skill, level), expressions, start node toggle, choice texts, and
  graph position.
- **All connections** -- Which nodes are connected, and which choice each
  connection originates from.
- **Camera state** -- The graph's scroll position and zoom level, so the
  view is restored when you reopen the file.

### Loading (Ctrl+O)

Opening a file via **Ctrl+O** clears the entire editor and recreates the
conversation from the file. All nodes and connections are rebuilt, and the
camera position and zoom are restored.

### Sharing Conversations

Because conversations are stored as standard files, you can share them with
other people who have the editor. Simply send the `.tres` or `.res` file and
they can open it with **Ctrl+O**.

You can also use saved conversation files directly with the
[engine](/engine/) to run dialogue trees in your game.

