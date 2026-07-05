extends Node

@export var editor:GraphEdit

@export var save_button:Button
@export var save_as_button:Button
@export var load_button:Button

@export var save_as_dialog:FileDialog
@export var load_dialog:FileDialog

var current_project_path:String
signal current_project_changed()

const HUB_SCENE := preload("res://entities/hub.tscn")

func _ready():
	save_button.pressed.connect(_on_save_pressed)
	save_as_button.pressed.connect(save_as_dialog.show)
	load_button.pressed.connect(load_dialog.show)
	
	save_as_dialog.file_selected.connect(save_project_to_file)
	load_dialog.file_selected.connect(load_project_from_file)
	
func _on_save_pressed():
	if has_current_project():
		save_current_project()
	else:
		save_as_dialog.show()

func save_project_to_file(path:String) -> bool:
	if !path.is_absolute_path():
		return false
	
	var conversation := ConversationResource.new()

	var hub_list: Array[Node] = []
	for child in editor.get_children():
		if child is CustomGraphHub:
			conversation.hubs.append(child.export())
			hub_list.append(child)

	conversation.connections = []
	for c in editor.connections:
		var from_idx := -1
		var to_idx := -1
		for i in range(hub_list.size()):
			if hub_list[i].name == c.from_node:
				from_idx = i
			if hub_list[i].name == c.to_node:
				to_idx = i
		if from_idx >= 0 and to_idx >= 0:
			conversation.connections.append({
				"from_hub": from_idx,
				"from_port": c.from_port,
				"to_hub": to_idx,
				"to_port": c.to_port,
				"keep_alive": c.get("keep_alive", false)
			})

	current_project_path = path
	current_project_changed.emit()

	var code := ResourceSaver.save(conversation, path) 

	print("saved conversation to ", path, " with code ", error_string(code))
	return code == OK

func load_project_from_file(path:String, make_current:=true) -> bool:
	if !path.is_absolute_path():
		return false

	for child in editor.get_children():
		if child is CustomGraphHub:
			editor.remove_child(child)
			child.queue_free()
	editor.clear_connections()

	var conversation := ResourceLoader.load(path) as ConversationResource
	if not conversation:
		return false

	if make_current:
		current_project_path = path
		current_project_changed.emit()

	var hubs_created: Array[CustomGraphHub] = []
	for hub_resource in conversation.hubs:
		var hub := HUB_SCENE.instantiate()
		hub.position_offset = hub_resource.position_offset
		editor.add_child(hub)
		hub.hub_resource = hub_resource
		hub.owner = editor.owner
		hubs_created.append(hub)

	for edge in conversation.connections:
		if edge.has("from_hub"):
			var from_idx := edge["from_hub"] as int
			var to_idx := edge["to_hub"] as int
			if from_idx >= 0 and from_idx < hubs_created.size() and to_idx >= 0 and to_idx < hubs_created.size():
				editor.connect_node(
					hubs_created[from_idx].name,
					edge["from_port"],
					hubs_created[to_idx].name,
					edge["to_port"],
					edge.get("keep_alive", false)
				)
		else:
			editor.connect_node(
				edge["from_node"],
				edge["from_port"],
				edge["to_node"],
				edge["to_port"],
				edge.get("keep_alive", false)
			)

	print("loaded conversation from ", path, " with ", conversation.hubs.size(), " hubs and ", conversation.connections.size(), " edges!")

	return true

func save_current_project() -> bool:
	if has_current_project():
		return save_project_to_file(current_project_path)
	return false

func has_current_project() -> bool:
	return current_project_path.is_absolute_path()
