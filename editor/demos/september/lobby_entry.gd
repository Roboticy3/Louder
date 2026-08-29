extends HBoxContainer

@export var client_data_listener_group := &"client_data_listener"
@export var server_listener_group := &"server_listener"
@export var component_group := &"lobby_entry_component"

@export var mode:Label
@export var username:Label
@export var character:Label

func _ready() -> void:
	add_to_group(client_data_listener_group)
	add_to_group(server_listener_group)

func export() -> Dictionary:
	var data := {}
	var seen_names := {}
	for child in find_children("*", "", true, false):
		if child.is_in_group(component_group):
			if seen_names.has(child.name):
				push_error("Lobby entry \"%s\" has duplicate component name \"%s\"; aborting export" % [name, child.name])
				return {}
			seen_names[child.name] = true
			data[child.name as String] = _export_component(child)
	return data

func import(data: Dictionary) -> void:
	for key in data:
		if key is String:
			var node := find_child(key, true, false)
			if node:
				_import_component(node, data[key])

func _export_component(node: Node) -> Variant:
	if node is Label:
		return node.text
	return null

func _import_component(node: Node, value: Variant) -> void:
	if node is Label:
		node.text = value

func _on_client_data_changed(to:Dictionary[int, Dictionary]):
	import(to.get(get_multiplayer_authority(), {}))

func _on_server_created(_port:int):
	_on_client_data_changed(NetworkLayer.client_data)
