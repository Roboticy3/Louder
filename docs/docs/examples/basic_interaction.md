# Bare Minimum

Simple example to make sure the engine is working, without delving into complex
flow features such as channels and priority, or features that require more
surrounding architecture to handle effectively like story state and callbacks.

Uses the editor demo conversation "res://demos/conversations/jail.json" with two perspectives- Freya and
Benjamin, configured by "res://demos/speakers/test.json" displayed side-by-side in one terminal.

Freya is on the left, and chooses with 1, 2, 3.. While Benjamin is on the right
and chooses with q, w, e...

This demonstrates the engine's ability to:
 - Load a conversation file and a config from absolute paths
 - Start a conversation
 - Handle choices and provide events to a front end
 - Do simple visibility logic for the separate views
Failing this logic constitutes a bug in the engine, which then needs to be updated.

And demonstrates the ability of an interface to:
 - Display speakers and text
 - Display choices
 - Relay choices to the engine
 - Show if the other view has choices currently by printing "Benjamin is thinking" in Freya's column or "Freya is thinking" in Benjamin's column respectively.
 - Aliasing. "Benjamin" appears as REDACTED to both parties despite being named in the config.
 - Continuing. When only 1 choice is available, it should be rendered as "CONTINUE", and bound to both 1 and ` for Freya and both q and Tab for Benjamin.

Even in this simple example, there are some subtleties to the engine that stand
out. The "Note" nodes in the conversation explain specific examples of these cases in more detail:
 - Private nodes. Nodes with recipients. are considered "private" and are only
 shown to a particular character.
 - Auto nodes. Auto nodes have multiple branches and may split the flow temporarily, even though there are no changes in channel in this exmaple.
 - Choice indexes. Conversation logic may hide some choices, offsetting the indices in the UI. The choice index passed by the UI is the index into the filtered choices for that particular character, not a general choice index.

This example is small enough to write exhaustive unit tests for, ensuring full functionality of the "core" dialogue system AND visual inspection of this system by a user before moving on to advanced features.