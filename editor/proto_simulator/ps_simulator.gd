class_name PSSimulator extends Node

@export var conversation: PSConversation

@export var start_position: int = 0

@onready var position := 0
@export var state:PSConversationState

func set_position(id) -> void:
	position = int(id)

func get_position() -> int:
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
	var conditions = hub.get("Conditions")
	if !(conditions is String):
		print("no conditions found")
		return true
	print(conditions)
	return true

func traverse(choice: String) -> bool:
	for connection in conversation.get_connections_from(position):
		if str(connection.get("from_choice", "")) == choice:
			position = int(connection.get("to_id", ""))
			state._on_traverse(position)
			return true
	return false

func traverse_index(i: int) -> bool:
	var choices := conversation.get_connections_from(position)
	if i < 0 or i >= choices.size():
		return false
	position = int(choices[i].get("to_id", ""))
	state._on_traverse(position)
	return true

func _evaluate_hub(h:Object) -> void:
	var expressions = h.get("Expressions")
	if !(expressions is String):
		print("no expressions found")
		return
	print(expressions)
	return
