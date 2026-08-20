class_name PSConversation extends Node

const OBJECT_TYPE := "type"
const TYPE_HUB := "hub"
const TYPE_CONNECTION := "connection"
const TYPE_CAMERA := "camera"

@export var input_path: String = ""

var hubs_by_id:Dictionary[int, Dictionary] = {}
var connections_by_from:Dictionary[int, Array] = {}
var camera := {}

func _ready() -> void:
	if not input_path.is_empty():
		load_from_json(input_path)

func load_from_json(path: String) -> bool:
	var parsed := JSONSceneLoader.load_from_json(path)
	if parsed.is_empty():
		return false

	unroll(parsed)
	#print(hubs_by_id)
	#print(connections_by_from)
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
				hubs_by_id[int(object.get("conversation_id", ""))] = object
			TYPE_CONNECTION:
				var from_id := int(object.get("from_id", ""))
				if not connections_by_from.has(from_id):
					connections_by_from[from_id] = []
				connections_by_from[from_id].append(object)
			TYPE_CAMERA:
				camera = object

func get_hub(conversation_id) -> Dictionary:
	return hubs_by_id.get(int(conversation_id), {})

func get_connections_from(from_id) -> Array:
	var connections = connections_by_from.get(int(from_id), [])
	return connections
