extends Node

@export var editor:GraphEdit

const HUB_SCENE := preload("res://entities/hub.tscn")

func _ready() -> void:
	editor.connection_request.connect(_on_connection_request)
	editor.disconnection_request.connect(_on_disconnection_request)
	editor.connection_to_empty.connect(_on_connection_to_empty)
	editor.connection_drag_started.connect(_on_connection_drag_started)

func _on_connection_request(from_node: String, from_port: int, to_node: String, to_port: int) -> void:
	editor.connect_node(from_node, from_port, to_node, to_port, false)

func _on_disconnection_request(from_node: String, from_port: int, to_node: String, to_port: int) -> void:
	editor.disconnect_node(from_node, from_port, to_node, to_port)

func _on_connection_to_empty(from_node: String, from_port: int, release_position: Vector2) -> void:
	var hub := HUB_SCENE.instantiate()
	hub.position_offset = editor.get_canvas_transform().affine_inverse() * release_position + Vector2.UP * hub.size.y * 0.5
	editor.add_child(hub)
	hub.owner = editor.owner
	editor.connect_node(from_node, from_port, hub.name, 0, false)
	editor.set_selected(hub)

var _drag_start_node := ""
var _drag_start_port := 0
var _drag_start_output := false
func _on_connection_drag_started(from_node: String, from_port: int, is_output: bool):

	var hub := editor.get_node(from_node)
	if not (hub is CustomGraphHub):
		return

	for connection in editor.get_connection_list_from_node(from_node):
		
		#{
			#from_node: StringName,
			#from_port: int,
			#to_node: StringName,
			#to_port: int,
			#keep_alive: bool
		#}
		
		var matches := false
		if is_output:
			matches = connection["from_node"] == from_node and connection["from_port"] == from_port
		else:
			matches = connection["to_node"] == from_node and connection["to_port"] == from_port

		if not matches:
			continue

		editor.disconnect_node(connection.from_node, connection.from_port, connection.to_node, connection.to_port)
		editor.force_connection_drag_end()

		_drag_start_node = connection["to_node"] if is_output else connection["from_node"]
		_drag_start_port = connection["to_port"] if is_output else connection["from_port"]
		_drag_start_output = !is_output
		
		print(_drag_start_node, ":", _drag_start_port, " (output: ", _drag_start_output, ")")
		return
