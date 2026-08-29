extends HBoxContainer

@export var group := &"lobby_entry"
@export var component_group := &"lobby_entry_component"

@export var mode:Label
@export var username:Label
@export var character:Label

func _ready() -> void:
	add_to_group(group)

func set_mode(to:String) -> void:
	mode.text = to + ":"

func set_username(to:String) -> void:
	username.text = to

func set_character(to:String) -> void:
	character.text = " (playing as " + to + ")"

func get_username() -> String:
	return username.text

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
