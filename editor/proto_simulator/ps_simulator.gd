class_name PSSimulator extends Node

@export var conversation: PSConversation

@export var start_position: String = ""

var position: String = ""

func _ready() -> void:
	if not start_position.is_empty():
		position = start_position

func set_position(id) -> void:
	position = str(id)

func get_position() -> String:
	return position

func list_choices(evaluate_conditions:=true) -> Array:
	var choices := conversation.get_connections_from(position)
	if evaluate_conditions: choices = choices.filter(_evaluate_choice)
	return choices

func _evaluate_choice(c:Object) -> bool:
	var to_id = c.get("to_id")
	if !(to_id is int):
		return false
	var hub = conversation.hubs_by_id[to_id]
	var conditions = hub.get("conditions")
	if !(conditions is String):
		return true
	print(conditions)
	return true

func traverse(choice: String) -> bool:
	for connection in conversation.get_connections_from(position):
		if str(connection.get("from_choice", "")) == choice:
			position = str(connection.get("to_id", ""))
			return true
	return false

func traverse_index(i: int) -> bool:
	var choices := conversation.get_connections_from(position)
	if i < 0 or i >= choices.size():
		return false
	position = str(choices[i].get("to_id", ""))
	return true
