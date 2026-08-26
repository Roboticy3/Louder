class_name PSConfig extends Node

const OBJECT_TYPE := "type"
const OBJECT_SPEAKER := "speaker"

const SPK_NAME := "speaker"
const SPK_PBLC := "public"
const SPK_OWND := "ownable"

var speakers_by_name:Dictionary[String, Dictionary]

@export var config_path := "res://demos/speakers/test.json"

func _ready():
	load_from_json(config_path)

func load_from_json(path:String):
	var parsed := JSONSceneLoader.load_from_json(path)
	if parsed.is_empty():
		return false

	unroll(parsed)
	#print(hubs_by_id)
	#print(connections_by_from)
	return true

func unroll(objects:Array):
	speakers_by_name.clear()
	for o in objects:
		if not (o is Dictionary):
			continue
		
		match (o.get(OBJECT_TYPE)):
			OBJECT_SPEAKER:
				if o.get(SPK_NAME): 
					print("registering speaker ", o.get(SPK_NAME))
					speakers_by_name[o.get(SPK_NAME)] = o
	
	print("registered speakers: ", speakers_by_name.keys())

func is_ownable(speaker:String):
	var s = speakers_by_name.get(speaker)
	if s is Dictionary:
		return true if s.get(SPK_OWND, false) else false
	return true

func is_public(speaker:String):
	var s = speakers_by_name.get(speaker)
	if s is Dictionary:
		return true if s.get(SPK_PBLC, false) else false
	return true

func _on_conversation_changed(_path:String):
	_ready()
