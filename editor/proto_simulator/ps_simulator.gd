class_name PSSimulator extends Node

@export var conversation_path:="user://test.json"
@export var conversation: PSConversation

@export var start_position: int = 0

@onready var position := 0
@export var state:PSConversationState


func _ready():
	var c := PSConversation.new()
	c.load_from_json(conversation_path)
	
	if c.hubs_by_id.is_empty():
		return
	
	var start := int(c.hubs_by_id.keys()[0])
	for id in c.hubs_by_id:
		if c.hubs_by_id[id].get("StartCheck"):
			start = id
			break
	
	if !state:
		state = PSConversationState.new()
	conversation = c
	warp(start)

func set_position(id) -> void:
	position = int(id)

func get_position() -> int:
	return position

func list_choices(evaluate_conditions:=true) -> Array:
	var choices := conversation.get_connections_from(position)
	if evaluate_conditions: choices = choices.filter(_evaluate_choice)
	return choices

func _evaluate_choice(c:Dictionary) -> bool:
	var to_id = int(c.get("to_id"))
	var hub = conversation.hubs_by_id[to_id]
	var conditions = str(hub.get("Conditions"))
	
	var conditions_arr := conditions.split("\n")
	var e := Expression.new()
	for condition in conditions_arr:
		
		var error := e.parse(condition, state.variable_names)
		if error != OK:
			push_warning("Condition has bad syntax. Treating as false")
			return false
		var result = e.execute(state.variable_values)
		if !result:
			print("Condition ", condition, " evaluated to false.")
			return false
	return true

func get_current_property(property:String):
	return str(conversation.hubs_by_id[position].get(property))

func traverse(choice: String) -> bool:
	for connection in conversation.get_connections_from(position):
		if str(connection.get("from_choice", "")) == choice:
			position = int(connection.get("to_id", ""))
			state._on_traverse(position)
			return true
	push_error("Traversed to nonexistent choice: ", choice)
	return false

func traverse_index(i: int) -> bool:
	var choices := conversation.get_connections_from(position)
	if i < 0 or i >= choices.size():
		push_error("Traversed to choice index out of bounds: ", i)
		return false
	position = int(choices[i].get("to_id", ""))
	state._on_traverse(position)
	return true

func warp(new_position:=position) -> bool:
	if new_position in conversation.hubs_by_id:
		position = new_position
		state._on_traverse(position)
		return true
	push_error("Warped to nonexistent location: ", new_position)
	return false

func _evaluate_hub(h:Dictionary) -> void:
	var expressions = str(h.get("Expressions"))
	print(expressions)
	return
