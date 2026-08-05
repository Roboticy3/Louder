class_name PSSimulator extends Node

@export var conversation_path:="user://test.json"
@export var conversation: PSConversation

@export var start_position: int = 0

@onready var position := 0
@export var state:PSConversationState
var e := Expression.new()

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
	if !hub.get("Conditions"):
		return true
	
	var conditions := str(hub.get("Conditions"))
	
	var conditions_arr := conditions.split("\n")
	for condition in conditions_arr:
		var error := e.parse(condition)
		if error != OK:
			push_warning("Condition ", condition, " \"", e.get_error_text(), "\". Treating as false")
			return false
		var result = e.execute([], state.scope)
		print("Condition ", condition, " evaluated to ", result, " \"", e.get_error_text(), "\"")
		print("Condition ", condition, " evaluated. Resulting scope: ", state.scope)
		if !result:
			return false
	return true

func get_current_property(property:String):
	return str(conversation.hubs_by_id[position].get(property))

func traverse(choice: String) -> bool:
	for connection in conversation.get_connections_from(position):
		if str(connection.get("from_choice", "")) == choice:
			position = int(connection.get("to_id", ""))
			_evaluate_hub(conversation.hubs_by_id[position])
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
	_evaluate_hub(conversation.hubs_by_id[position])
	state._on_traverse(position)
	return true

func warp(new_position:=position) -> bool:
	if new_position in conversation.hubs_by_id:
		position = new_position
		_evaluate_hub(conversation.hubs_by_id[position])
		state._on_traverse(position)
		return true
	push_error("Warped to nonexistent location: ", new_position)
	return false

func _evaluate_hub(h:Dictionary) -> void:
	if !h.get("Expressions"):
		return
	var expressions = str(h.get("Expressions"))
	var expressions_arr = expressions.split("\n")
	
	#three group regex to manually parse assignments:
	#(gap) (identifier) (gap) (equals) (expression)
	var re := RegEx.create_from_string(r"\s*([a-zA-Z][a-zA-Z0-9_]*)\s*(=)\s*(.*)")
	for expression in expressions_arr:
		var matches := re.search(expression)
		if matches.strings.size() < 4:
			push_warning("Assignment ", expression, " is incomplete. Expected the form \"<identifier> = <expression>\"")
			continue
		
		var error := e.parse(matches.strings[3])
		if error != OK:
			push_warning("Expression ", matches.strings[3], " \"", e.get_error_text(), "\". Skipping..")
		state.scope[matches.strings[1]] = e.execute([], state.scope)
		print("Expression ", matches.strings[3], " evaluated \"", e.get_error_text(), "\". Resulting scope: ", state.scope)
	return
