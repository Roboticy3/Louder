extends Node

@export var editor:GraphEdit
@export var pointer:PointerLogic
@export var debugger:OnScreenDebugger

@export var port_change_listener_group := &"port_change_listener"

func _ready() -> void:
	add_to_group(port_change_listener_group)
	editor.connection_request.connect(_on_connection_request)
	editor.disconnection_request.connect(_on_disconnection_request)
	editor.connection_to_empty.connect(_on_connection_to_empty)
	editor.connection_drag_started.connect(_on_connection_drag_started)

func _on_port_changed(node_name:String, port:int) -> void:
	var node:GraphNode = editor.get_node_or_null(node_name)
	if node == null:
		return
	
	var connections := editor.get_connection_list_from_node(node_name)
	for c in connections:
		if c["to_node"] == node_name:
			var slot := node.get_input_port_slot(c["to_port"])
			if !node.is_slot_enabled_left(slot):
				editor.disconnect_node(c["from_node"], c["from_port"], c["to_node"], c["to_port"])
		elif c["from_node"] == node_name:
			var slot := node.get_output_port_slot(c["from_port"])
			if !node.is_slot_enabled_right(slot):
				editor.disconnect_node(c["from_node"], c["from_port"], c["to_node"], c["to_port"])

func _on_connection_request(from_node: String, from_port: int, to_node: String, to_port: int) -> void:
	editor.connect_node(from_node, from_port, to_node, to_port, false)

func _on_disconnection_request(from_node: String, from_port: int, to_node: String, to_port: int) -> void:
	editor.disconnect_node(from_node, from_port, to_node, to_port)

func _on_connection_to_empty(from_node: String, from_port: int, release_position: Vector2) -> void:
	var source = editor.get_node_or_null(from_node)
	var make_next_choice := false
	if source is CustomGraphHub:
		var choices:Array[Label] = source.choices_section_logic.choices
		var audible:bool = choices.size() > 0 and choices[from_port].text.find("\"") != -1
		make_next_choice = audible and !source.get_data("is_choice", false) and !source.get_data("auto", false)
	
	var hub:CustomGraphHub = NetworkEntityDefaults.ENTITY_SOURCES["hub"].instantiate()
	hub.import({
		"is_choice": make_next_choice
	})
	editor.add_child(hub)
	hub.owner = editor.owner

	hub.position_offset = pointer.get_graph_pointer_position() + Vector2.UP * hub.size.y * 0.5

	editor.connect_node(from_node, from_port, hub.name, 0, false)
	editor.set_selected(hub)

func _on_connection_drag_started(from_node: String, from_port: int, is_output: bool) -> void:
	if !is_output:
		return

	var node:GraphNode = editor.get_node_or_null(from_node)
	if node == null:
		return
	
	var slot := node.get_output_port_slot(from_port)
	for c in editor.get_connection_list_from_node(from_node):
		if c["from_node"] == from_node and c["from_port"] == from_port:
			var to_node:GraphNode = editor.get_node_or_null(NodePath(c["to_node"]))
			if to_node == null:
				continue
			var to_slot := to_node.get_input_port_slot(c["to_port"])
			editor.disconnect_node(c["from_node"], c["from_port"], c["to_node"], c["to_port"])
