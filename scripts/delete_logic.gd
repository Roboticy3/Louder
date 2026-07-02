extends Node

@export var editor:GraphEdit

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("ui_graph_delete"):
		for c in editor.get_children():
			if c is GraphElement and c.selected:
				c.queue_free()
