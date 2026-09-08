extends Node

const OBJECT_TYPE := "type"
const CLIPBOARD_REFERENCE := "__copy_reference"

@export var editor:GraphEdit
@export var pointer:PointerLogic

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("graph_copy"):
		DisplayServer.clipboard_set(serialize_selection())
		pass
	elif event.is_action_pressed("graph_paste"):
		deserialize_selection(DisplayServer.clipboard_get())

func serialize_selection() -> String:
	var selected: Array[NetworkEntity]
	for n in editor.get_children():
		if n is NetworkEntity:
			if n.selected:
				selected.append(n)
	print("copied ", selected.size(), " objects")
	if selected.is_empty():
		return "[]"
	
	var reference := pointer.get_graph_pointer_position()
	var data: Array = [{"__copy_reference": [reference.x, reference.y]}]
	for n in selected:
		data.append(ProjectManagerHelpers.export_network_entity(n))
	return JSON.stringify(data)

func deserialize_selection(clipboard:String) -> bool:
	var data = JSON.parse_string(clipboard)
	if !(data is Array):
		return false
	
	var copy_reference := Vector2.ZERO
	var payload: Array = []
	for entry in data:
		if entry is Dictionary and entry.has(CLIPBOARD_REFERENCE):
			var ref := entry[CLIPBOARD_REFERENCE] as Array
			copy_reference = Vector2(float(ref[0]), float(ref[1]))
		else:
			payload.append(entry)
	
	var paste_offset := pointer.get_graph_pointer_position() - copy_reference
	
	for n in editor.get_children():
		if n is NetworkEntity:
			if n.selected:
				n.selected = false
	
	var hubs_created: Array[NetworkEntity]
	for s in payload:
		if s is Dictionary:
			var type := str(s.get(OBJECT_TYPE, ""))
		
			if type in NetworkEntityDefaults.ENTITY_SOURCES:
				var entity:NetworkEntity = NetworkEntityDefaults.ENTITY_SOURCES[type].instantiate()
				var hub_data := ProjectManagerHelpers.import_network_entity(s, entity)
				hub_data["position_offset"] = ProjectManagerHelpers.join_vector(s, "position_offset") + paste_offset
				entity.hub_data = hub_data
				editor.add_child(entity)
				entity.selected = true
				hubs_created.append(entity) 
	print("pasted ", hubs_created.size(), " objects")
	return !hubs_created.is_empty()
