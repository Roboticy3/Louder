# Editor

The Louder editor is an open source graph editor made using [Godot Game Engine](https://godotengine.org/),
which provides a [powerful base](https://docs.godotengine.org/en/stable/classes/class_graphedit.html)
for graph editing which this editor is built on top of. To match this, the
editor exports conversations in Godot's [resource format](https://docs.godotengine.org/en/stable/tutorials/scripting/resources.html)
, a YAML-style format (.res and .tres) which provides rich options for the 
editor without much overhead. You can share conversations for other people to 
work with in their editor, or use conversations with the [engine](../engine/)
to run dialogue trees.