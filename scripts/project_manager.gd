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

	for child in editor.get_children():
		if child is CustomGraphHub:
			conversation.hubs.append(child.export())
			

	conversation.connections = editor.connections

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

	for hub_resource in conversation.hubs:
		var hub := HUB_SCENE.instantiate()
		editor.add_child(hub)
		hub.hub_resource = hub_resource
		hub.owner = editor.owner

	for edge in conversation.connections:
		#{
			#from_node: StringName,
			#from_port: int,
			#to_node: StringName,
			#to_port: int,
			#keep_alive: bool
		#}
		editor.connect_node(edge["from_node"], edge["from_port"], edge["to_node"], edge["to_port"], edge["keep_alive"])

	print("loaded conversation from ", path, " with ", conversation.hubs.size(), " hubs and ", conversation.connections.size(), " edges!")

	return true

func save_current_project() -> bool:
	if has_current_project():
		return save_project_to_file(current_project_path)
	return false

func has_current_project() -> bool:
	return current_project_path.is_absolute_path()
