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

func list_choices() -> Array:
	return conversation.get_connections_from(position)

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
