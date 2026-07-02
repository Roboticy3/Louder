extends Node

@export var editor:GraphEdit

const HUB_SCENE := preload("res://entities/hub.tscn")

var spawns_on_point := 0

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("ui_create"):
		var mouse_pos := editor.get_local_mouse_position() + Vector2.ONE * 5 * spawns_on_point
		var canvas_pos := editor.get_canvas_transform().affine_inverse() * mouse_pos
		var hub := HUB_SCENE.instantiate()
		hub.position_offset = canvas_pos - hub.size * 0.5
		editor.add_child(hub)
		hub.owner = editor.owner
		editor.set_selected(hub)
		get_viewport().set_input_as_handled()
		spawns_on_point += 1
	if event is InputEventMouseMotion:
		spawns_on_point = 0
