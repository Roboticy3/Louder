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
		var connection := ConnectionResource.new()
		#get choice text from the hub node using c["from_port"]
		var from_hub = editor.get_node(NodePath(c["from_node"]))
		if from_hub is CustomGraphHub:
			connection.from_choice = from_hub.choices_logic.port_out_to_choice(c["from_port"])
		else:
			continue

		connection.from_node = c["from_node"]
		connection.to_node = c["to_node"]

		conversation.connections.append(connection)

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

	var new_connections:Array[Dictionary] = []
	for connection in conversation.connections:
		var from_hub := editor.get_node_or_null(NodePath(connection.from_node))
		var to_hub := editor.get_node_or_null(NodePath(connection.to_node))
		if (from_hub is CustomGraphHub) && (to_hub is CustomGraphHub):
			var from_port:int = from_hub.choices_logic.choice_to_port_out(connection.from_choice)
			var to_port:int = to_hub.choices_logic.port_in()
			
			new_connections.append({
				"from_node": connection.from_node,
				"from_port": from_port,
				"to_node": connection.to_node,
				"to_port": to_port
			})
	editor.connections = new_connections

	print("loaded conversation from ", path, " with ", conversation.hubs.size(), " hubs and ", conversation.connections.size(), " edges!")

	return true

func save_current_project() -> bool:
	if has_current_project():
		return save_project_to_file(current_project_path)
	return false

func has_current_project() -> bool:
	return current_project_path.is_absolute_path()
