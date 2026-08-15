class_name JSONSceneLoader extends Node

static func load_from_json(path: String) -> Array:
	var file := FileAccess.open(path, FileAccess.READ)
	if not file:
		push_error("simulator: could not open '%s'" % path)
		return []

	var parsed = JSON.parse_string(file.get_as_text())
	file.close()
	if not (parsed is Array):
		push_error("JSONSceneLoader: '%s' does not contain an array of objects" % path)
		return []

	#print(hubs_by_id)
	#print(connections_by_from)
	return parsed
