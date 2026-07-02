extends Node

@export var editor:GraphEdit

const HUB_SCENE := preload("res://entities/hub.tscn")

func _ready() -> void:
	editor.connection_request.connect(_on_connection_request)
	editor.disconnection_request.connect(_on_disconnection_request)
	editor.connection_to_empty.connect(_on_connection_to_empty)

func _on_connection_request(from_node: String, from_port: int, to_node: String, to_port: int) -> void:
	editor.connect_node(from_node, from_port, to_node, to_port)

func _on_disconnection_request(from_node: String, from_port: int, to_node: String, to_port: int) -> void:
	editor.disconnect_node(from_node, from_port, to_node, to_port)

func _on_connection_to_empty(from_node: String, from_port: int, release_position: Vector2) -> void:
	var hub := HUB_SCENE.instantiate()
	hub.position_offset = editor.get_canvas_transform().affine_inverse() * release_position + Vector2.UP * hub.size.y * 0.5
	editor.add_child(hub)
	hub.owner = editor.owner
	editor.connect_node(from_node, from_port, hub.name, 0)
	editor.set_selected(hub)
