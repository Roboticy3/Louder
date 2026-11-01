extends MenuButton

@export var editor:GraphEdit

const SAVE := 0
const SAVE_AS := 1
const LOAD := 2
const NEW := 3

var current_project_path:String
signal current_project_changed()

var save_as_dialog:FileDialog
var load_dialog:FileDialog

@export var disable_save := false
@export var disable_load := false
@export var disable_new := false

func _ready():
	text = "File"
	var popup := get_popup()
	popup.add_item("Save", SAVE)
	popup.add_item("Save As...", SAVE_AS)
	popup.add_item("Load...", LOAD)
	popup.add_item("New...", NEW)
	popup.id_pressed.connect(_on_id_pressed)
	
	popup.set_item_shortcut(SAVE, _shortcut_for_action(&"graph_save"))
	popup.set_item_shortcut(SAVE_AS, _shortcut_for_action(&"graph_save_as"))
	popup.set_item_shortcut(LOAD, _shortcut_for_action(&"graph_load"))
	popup.set_item_shortcut(NEW, _shortcut_for_action(&"graph_new"))

	popup.set_item_disabled(SAVE, disable_save)
	popup.set_item_disabled(SAVE_AS, disable_save)
	popup.set_item_disabled(LOAD, disable_load)
	popup.set_item_disabled(NEW, disable_new)

	save_as_dialog = _make_dialog("Save As")
	save_as_dialog.file_selected.connect(save_project_to_file)

	load_dialog = _make_dialog("Load", true)
	load_dialog.file_selected.connect(load_project_from_file)
	
	var cc :=  CustomConversationLayer.conversation_path
		load_project_from_file(cc)
	if FileAccess.file_exists(cc):

func _make_dialog(dialog_name:String, for_open:=false) -> FileDialog:
	var dialog := FileDialog.new()
	dialog.name = dialog_name
	dialog.use_native_dialog = true
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.filters = PackedStringArray(["*.json"])
	if for_open:
		dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
		dialog.title = "Open a File"
	add_child(dialog)
	return dialog

func _shortcut_for_action(action:StringName) -> Shortcut:
	var s := Shortcut.new()
	var event := InputEventAction.new()
	event.action = action
	s.events = [event]
	return s

func _on_id_pressed(id:int):
	match id:
		SAVE:
			_on_save_pressed()
		SAVE_AS:
			save_as_dialog.show()
		LOAD:
			load_dialog.show()
		NEW:
			new_project()

func _on_save_pressed():
	if has_current_project():
		save_current_project()
	else:
		save_as_dialog.show()

func save_project_to_file(path:String) -> bool:
	if !path.is_absolute_path():
		return false

	var objects:Array[Dictionary] = []

	for child in editor.get_children():
		if child is NetworkEntity:
			objects.append(ProjectManagerHelpers.export_network_entity(child))

	var connections: Array[Dictionary] = []
	for c in editor.connections:
		var from_hub = editor.get_node(NodePath(c["from_node"]))
		var to_hub = editor.get_node(NodePath(c["to_node"]))
		if not (from_hub is NetworkEntity and to_hub is NetworkEntity):
			continue
		connections.append({
			ProjectManagerHelpers.OBJECT_TYPE: ProjectManagerHelpers.TYPE_CONNECTION,
			"from_id": str(from_hub.id),
			"from_port": c["from_port"],
			"to_id": str(to_hub.id),
		})
	
	connections.sort_custom(func(a, b):
		if a["from_id"] != b["from_id"]:
			return a["from_id"] < b["from_id"]
		return a["from_port"] < b["from_port"]
	)
	
	for c in connections:
		objects.append(c)

	var camera := {
		ProjectManagerHelpers.OBJECT_TYPE: ProjectManagerHelpers.TYPE_CAMERA, 
		"zoom": str(editor.zoom)
	}
	ProjectManagerHelpers.split_vector(camera, "position", editor.scroll_offset)
	objects.append(camera)

	current_project_path = path
	current_project_changed.emit()

	var file := FileAccess.open(path, FileAccess.WRITE)
	if not file:
		return false
	file.store_string(JSON.stringify(objects, "\t"))
	file.close()

	print("saved conversation to ", path, " with ", objects.size(), " objects")
	return true

func load_project_from_file(path:String, make_current:=true) -> bool:
	if !path.is_absolute_path():
		return false

	await clear_project()

	var file := FileAccess.open(path, FileAccess.READ)
	if not file:
		return false
	var parsed = JSON.parse_string(file.get_as_text())
	file.close()
	if not (parsed is Array):
		return false

	if make_current:
		current_project_path = path
		current_project_changed.emit()

	var hubs_created: Array[NetworkEntity] = []
	var connections: Array[Dictionary] = []
	var camera_position := Vector2.ZERO
	var camera_zoom := 1.0

	for object in parsed:
		if not (object is Dictionary):
			continue
		
		var ot := str(object.get(ProjectManagerHelpers.OBJECT_TYPE))
		
		if ot in NetworkEntityDefaults.ENTITY_SOURCES:
			var entity:NetworkEntity = NetworkEntityDefaults.ENTITY_SOURCES[ot].instantiate()
			entity.hub_data = ProjectManagerHelpers.import_network_entity(object, entity)
			editor.add_child(entity)
			hubs_created.append(entity)
			
		match ot:
			ProjectManagerHelpers.TYPE_CONNECTION:
				connections.append(object)
			ProjectManagerHelpers.TYPE_CAMERA:
				camera_position = ProjectManagerHelpers.join_vector(object, "position")
				camera_zoom = float(object.get("zoom", "1"))

	var new_connections:Array[Dictionary] = []
	for connection in connections:
		var from_hub := IdSystem.get_assigned(int(connection.get("from_id", "-1")))
		var to_hub := IdSystem.get_assigned(int(connection.get("to_id", "-1")))
		if (from_hub is NetworkEntity) && (to_hub is NetworkEntity):
			
			new_connections.append({
				"from_node": String(from_hub.name),
				"from_port": connection.get("from_port"),
				"to_node": String(to_hub.name),
				"to_port": connection.get("to_port"),
				"keep_alive": false
			})
	editor.connections = new_connections

	editor.scroll_offset = camera_position
	editor.zoom = camera_zoom

	print("loaded conversation from ", path, " with ", hubs_created.size(), " hubs and ", connections.size(), " edges!")

	return true

func new_project():
	if disable_new: return
	clear_project()

func clear_project():
	for child in editor.get_children():
		if child is NetworkEntity:
			editor.remove_child(child)
			child.queue_free()
	editor.clear_connections()
	IdSystem.clear_ids()
	current_project_path = ""

	await get_tree().process_frame # wait for the editor to process the removal of nodes

func save_current_project() -> bool:
	if has_current_project():
		return save_project_to_file(current_project_path)
	return false

func has_current_project() -> bool:
	return current_project_path.is_absolute_path()

func _on_conversation_changed(new_path:String):
	load_project_from_file(new_path)
