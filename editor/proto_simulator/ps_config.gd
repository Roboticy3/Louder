class_name PSConfig extends Node

const OBJECT_TYPE := "type"
const OBJECT_SPEAKER := "speaker"

var speakers_by_name:Dictionary[String, Dictionary]

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
				speakers_by_name[o.get("Name")] = o
