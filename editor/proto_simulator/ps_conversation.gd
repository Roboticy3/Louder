class_name PSConversation extends Node

const OBJECT_TYPE := "type"
const TYPE_HUB := "hub"
const TYPE_CONNECTION := "connection"
const TYPE_CAMERA := "camera"

@export var input_path: String = ""

var hubs_by_id := {}
var connections_by_from := {}
var camera := {}

func _ready() -> void:
	if not input_path.is_empty():
		load_from_json(input_path)

func load_from_json(path: String) -> bool:
	var file := FileAccess.open(path, FileAccess.READ)
	if not file:
		push_error("simulator: could not open '%s'" % path)
		return false

	var parsed = JSON.parse_string(file.get_as_text())
	file.close()
	if not (parsed is Array):
		push_error("simulator: '%s' does not contain an array of objects" % path)
		return false

	unroll(parsed)
	print(hubs_by_id)
	print(connections_by_from)
	return true

func unroll(objects: Array) -> void:
	hubs_by_id.clear()
	connections_by_from.clear()
	camera = {}

	for object in objects:
		if not (object is Dictionary):
			continue
		match object.get(OBJECT_TYPE):
			TYPE_HUB:
				hubs_by_id[str(object.get("conversation_id", ""))] = object
			TYPE_CONNECTION:
				var from_id := str(object.get("from_id", ""))
				if not connections_by_from.has(from_id):
					connections_by_from[from_id] = []
				connections_by_from[from_id].append(object)
			TYPE_CAMERA:
				camera = object

func get_hub(conversation_id) -> Dictionary:
	return hubs_by_id.get(str(conversation_id), {})

func get_connections_from(from_id) -> Array:
	return connections_by_from.get(str(from_id), [])
