class_name PSConfig extends Node

const OBJECT_TYPE := "type"
const OBJECT_SPEAKER := "speaker"
const SPEAKER_NAME := "speaker"

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
				if o.get(SPEAKER_NAME): 
					print("registering speaker ", o.get(SPEAKER_NAME))
					speakers_by_name[o.get(SPEAKER_NAME)] = o
	
	print("registered speakers: ", speakers_by_name.keys())
