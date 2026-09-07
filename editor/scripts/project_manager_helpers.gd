extends Node

const OBJECT_TYPE := "type"
const TYPE_HUB := CustomGraphHub.TYPE
const TYPE_SPEAKER := SpeakerEntity.TYPE
const TYPE_CONNECTION := "connection"
const TYPE_CAMERA := "camera"

func export_network_entity(hub: NetworkEntity) -> Dictionary:
	var data := {OBJECT_TYPE: hub.get_entity_type()}
	var exported := hub.export()
	for key in exported:
		var value = exported[key]
		if value == null:
			continue
		match key:
			"position_offset":
				split_vector(data, "position_offset", value)
			"conversation_id":
				data[key] = str(value)
			_:
				data[key] = value if (value is bool or value is PackedStringArray) else str(value)
	return data

func import_network_entity(data: Dictionary, hub: NetworkEntity) -> Dictionary:
	var restored := {"position_offset": join_vector(data, "position_offset")}
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

func split_vector(data: Dictionary, prefix: String, value: Vector2) -> void:
	data[prefix + ":x"] = str(value.x)
	data[prefix + ":y"] = str(value.y)

func join_vector(data: Dictionary, prefix: String) -> Vector2:
	return Vector2(
		float(data.get(prefix + ":x", "0")),
		float(data.get(prefix + ":y", "0"))
	)
