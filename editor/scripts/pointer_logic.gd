class_name PointerLogic extends Node

@export var editor:GraphEdit

func get_graph_pointer_position() -> Vector2:
	var mouse_pos := editor.get_local_mouse_position()
	#https://github.com/godotengine/godot-proposals/issues/14306
	return (mouse_pos + editor.scroll_offset) / editor.zoom
