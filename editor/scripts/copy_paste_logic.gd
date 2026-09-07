extends Node

const OBJECT_TYPE := "type"

@export var editor:GraphEdit

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("graph_copy"):
		DisplayServer.clipboard_set(serialize_selection())
		pass
	elif event.is_action_pressed("graph_paste"):
		pass

func serialize_selection() -> String:
	var selected: Array[Dictionary]
	for n in editor.get_children():
		if n is NetworkEntity:
			if n.selected:
				selected.append(ProjectManagerHelpers.export_network_entity(n))
	return JSON.stringify(selected)

func deserialize_selection(clipboard:String) -> bool:
	var selected = JSON.parse_string(clipboard)
	if !(selected is Array):
		return false
	for n in editor.get_children():
		if n is NetworkEntity:
			if n.selected:
				n.selected = false
	
	var hubs_created: Array[NetworkEntity]
	for s in selected:
		if s is Dictionary:
			var type := str(s.get(OBJECT_TYPE, ""))
		
			if type in NetworkEntityDefaults.ENTITY_SOURCES:
				var entity:NetworkEntity = NetworkEntityDefaults.ENTITY_SOURCES[type].instantiate()
				entity.hub_data = ProjectManagerHelpers.import_network_entity(s, entity)
				editor.add_child(entity)
				hubs_created.append(entity) 
	return !hubs_created.is_empty()
