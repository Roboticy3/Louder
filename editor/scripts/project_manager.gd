extends MenuButton

@export var editor:GraphEdit

const SAVE := 0
const SAVE_AS := 1
const LOAD := 2

var current_project_path:String
signal current_project_changed()

const HUB_SCENE := preload("res://entities/hub.tscn")

const OBJECT_TYPE := "type"
const TYPE_HUB := "hub"
const TYPE_CONNECTION := "connection"
const TYPE_CAMERA := "camera"

var save_as_dialog:FileDialog
var load_dialog:FileDialog

func _ready():
	text = "File"
	var popup := get_popup()
	popup.add_item("Save", SAVE)
	popup.add_item("Save As...", SAVE_AS)
	popup.add_item("Load...", LOAD)
	popup.id_pressed.connect(_on_id_pressed)
	popup.set_item_shortcut(SAVE, _shortcut_for_action(&"graph_save"))
	popup.set_item_shortcut(SAVE_AS, _shortcut_for_action(&"graph_save_as"))
	popup.set_item_shortcut(LOAD, _shortcut_for_action(&"graph_load"))

	save_as_dialog = _make_dialog("Save As")
	save_as_dialog.file_selected.connect(save_project_to_file)

	load_dialog = _make_dialog("Load", true)
	load_dialog.file_selected.connect(load_project_from_file)

func _make_dialog(dialog_name:String, for_open:=false) -> FileDialog:
	var dialog := FileDialog.new()
	dialog.name = dialog_name
	dialog.use_native_dialog = true
	dialog.access = FileDialog.ACCESS_RESOURCES
	dialog.filters = PackedStringArray(["*.json"])
	if for_open:
		dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
		dialog.title = "Open a File"
	add_child(dialog)
	return dialog

func _shortcut_for_action(action:StringName) -> Shortcut:
	var shortcut := Shortcut.new()
	var event := InputEventAction.new()
	event.action = action
	shortcut.events = [event]
	return shortcut

func _on_id_pressed(id:int):
	match id:
		SAVE:
			_on_save_pressed()
		SAVE_AS:
			save_as_dialog.show()
		LOAD:
			load_dialog.show()

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
			objects.append(_export_hub(child))

	for c in editor.connections:
		var from_hub = editor.get_node(NodePath(c["from_node"]))
		var to_hub = editor.get_node(NodePath(c["to_node"]))
		if not (from_hub is NetworkEntity and to_hub is NetworkEntity):
			continue
		objects.append({
			OBJECT_TYPE: TYPE_CONNECTION,
			"from_id": str(from_hub.id),
			"from_choice": from_hub.get_choices_logic().port_out_to_choice(c["from_port"]),
			"to_id": str(to_hub.id),
		})

	var camera := {OBJECT_TYPE: TYPE_CAMERA, "zoom": str(editor.zoom)}
	_split_vector(camera, "position", editor.scroll_offset)
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

	for child in editor.get_children():
		if child is NetworkEntity:
			editor.remove_child(child)
			child.queue_free()
	editor.clear_connections()

	await get_tree().process_frame # wait for the editor to process the removal of nodes

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
		match object.get(OBJECT_TYPE):
			TYPE_HUB:
				var hub:NetworkEntity = HUB_SCENE.instantiate()
				hub.hub_data = _import_hub_data(object, hub)
				editor.add_child(hub)
				hubs_created.append(hub)
			TYPE_CONNECTION:
				connections.append(object)
			TYPE_CAMERA:
				camera_position = _join_vector(object, "position")
				camera_zoom = float(object.get("zoom", "1"))

	var new_connections:Array[Dictionary] = []
	for connection in connections:
		var from_hub := IdSystem.get_assigned(int(connection.get("from_id", "-1")))
		var to_hub := IdSystem.get_assigned(int(connection.get("to_id", "-1")))
		if (from_hub is NetworkEntity) && (to_hub is NetworkEntity):
			var from_port:int = from_hub.get_choices_logic().choice_to_port_out(connection.get("from_choice", ""))
			var to_port:int = to_hub.get_choices_logic().port_in()
			
			new_connections.append({
				"from_node": String(from_hub.name),
				"from_port": from_port,
				"to_node": String(to_hub.name),
				"to_port": to_port,
				"keep_alive": false
			})
	editor.connections = new_connections

	editor.scroll_offset = camera_position
	editor.zoom = camera_zoom

	print("loaded conversation from ", path, " with ", hubs_created.size(), " hubs and ", connections.size(), " edges!")

	return true

func _export_hub(hub: NetworkEntity) -> Dictionary:
	var data := {}
	var exported := hub.export()
	for key in exported:
		var value = exported[key]
		if value == null:
			continue
		match key:
			"position_offset":
				_split_vector(data, "position_offset", value)
			"conversation_id":
				data[key] = str(value)
			_:
				data[key] = value if (value is bool or value is PackedStringArray) else str(value)
	data[OBJECT_TYPE] = TYPE_HUB
	return data

func _import_hub_data(data: Dictionary, hub: NetworkEntity) -> Dictionary:
	var restored := {"position_offset": _join_vector(data, "position_offset")}
	for key in data:
		match key:
			"conversation_id":
				restored[key] = int(data[key])
			"position_offset:x", "position_offset:y":
				continue
			_:
				var node := hub.get_node_or_null(NodePath(key))
				var value = data[key]
				if node is ChoicesSectionLogic:
					restored[key] = PackedStringArray(value)
				elif node is SpinBox:
					restored[key] = float(value)
				else:
					restored[key] = value
	return restored

func _split_vector(data: Dictionary, prefix: String, value: Vector2) -> void:
	data[prefix + ":x"] = str(value.x)
	data[prefix + ":y"] = str(value.y)

func _join_vector(data: Dictionary, prefix: String) -> Vector2:
	return Vector2(
		float(data.get(prefix + ":x", "0")),
		float(data.get(prefix + ":y", "0"))
	)

func save_current_project() -> bool:
	if has_current_project():
		return save_project_to_file(current_project_path)
	return false

func has_current_project() -> bool:
	return current_project_path.is_absolute_path()
