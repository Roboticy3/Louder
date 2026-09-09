class_name PSSimulator extends Node

#interface to iterate through conversations

const HubProperties := PSObjects.HubProperties
const BodyProperties := PSObjects.BodyProperties
const ChoiceProperties := PSObjects.ChoiceProperties
const SpeakerProperties := PSObjects.SpeakerProperties

const DEFAULT_BRANCH := "main"

@export var group := &"ps_simulator"
@export var conversation:PSConversation
@export var state:PSDialogueState

var expression:Expression

func _ready():
	expression = Expression.new()
	add_to_group(group)
	if !state:
		state = PSDialogueState.new()

#Get a property from a position in the conversation.
func get_property(position: int, property: HubProperties, default=null) -> Variant:
	return _hub(position).get(property, default)

func _hub(position: int) -> Dictionary:
	if conversation == null:
		return {}
	return conversation.get_hub(position)

#Filters
func no_filter(_data:Dictionary) -> bool: return true
func has_property_or_flag(data:Dictionary, property): return true if data.get(property, false) else false

#List choices available at a certain position based on a given filter.
#Choices linking to hubs flagged with HubProperties.once will not be accessible
#if they have been visited in `align`. This history can be controlled through the
#`state.covered` variable.
func list_choices(position: int, filter:=no_filter) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	if conversation == null:
		return result
	for connection in conversation.get_connections_from(position):
		var to_id: int = int(connection.get(ChoiceProperties.to_id, -1))
		
		if to_id == 38:
			pass
		
		if get_property(to_id, HubProperties.once) and to_id in state.covered:
			continue
		
		if !check_conditions(to_id):
			continue
		
		var choice := {
			ChoiceProperties.from_choice: str(connection.get(ChoiceProperties.from_choice, "")),
			ChoiceProperties.to_id: to_id,
		}
		
		if filter.call(choice):
			result.append(choice)
	return result

#align a body with the conversation according to a choice. The body represents
#a position in the conversation. The choice + the body's position will result in
#the same position as using `get_destination`.
#
#returns a copy of the body with BodyProperties.success filled, and the position,
#branch, and choices updated to the body's current position in the conversation.
#
#`choice` is validated against `list_choices` on the body's position, but the user's
#`filter` is *not* used. The user's filter represents visibility, so the user
#should simply not see choices that are not available to them specifically. 
#
#`filter` is also relevant when the choice leads to a hub with
#HubProperties.auto set to true. At this point, the first choice that passes
#the filter will be taken. align will repeat this until either it has found a
#non-auto hub, or no choices remain, in which case the body is tagged with
#BodyProperties.success: false.
#
#If HubProperties.branch is not an empty string, BodyProperties.branch will
#change. Otherwise, BodyProperties.branch is preserved (and, generally,
#properties of the input body are copied into the duplicate body unless other-
#wise stated).
func align(body: Dictionary[BodyProperties, Variant], choice: String, filter:=no_filter) -> Dictionary[BodyProperties, Variant]:
	
	var from_branch: String = body.get(BodyProperties.branch, DEFAULT_BRANCH)
	var from_position: int = body.get(BodyProperties.position, -1)
	var result := body.duplicate()
	
	var valid := false
	for c in list_choices(from_position):
		if str(c.get(ChoiceProperties.from_choice, "")) == choice:
			valid = true
			break
	if not valid:
		result[BodyProperties.success] = false
		return result
	
	result = _traverse(result, get_destination(from_position, choice), choice, filter)
	if result.get(BodyProperties.branch, DEFAULT_BRANCH) != from_branch:
		result[BodyProperties.branch_changed] = true
	
	return result

#Warp `body` directly to `position`, bypassing choice validation. Like align,
#auto hubs are traversed after landing. `taken_choice` is left empty.
func warp(body: Dictionary[BodyProperties, Variant], position: int, filter:=no_filter) -> Dictionary[BodyProperties, Variant]:
	var from_branch: String = body.get(BodyProperties.branch, DEFAULT_BRANCH)
	var result := body.duplicate()
	result = _traverse(result, position, "", filter)
	if result.get(BodyProperties.branch, DEFAULT_BRANCH) != from_branch:
		result[BodyProperties.branch_changed] = true
	return result

#Move `body` to `position`, then auto-advance through any auto hubs until a
#non-auto hub is reached, or the conversation runs dry. `taken_choice` is the
#choice that moved the body here ("" when starting a conversation).
func _traverse(body: Dictionary[BodyProperties, Variant], position: int, choice: String, filter: Callable) -> Dictionary[BodyProperties, Variant]:
	#set breakpoints here to debug specific conversation points
	if position == 79:
		pass
	
	var visited := {}
	var current := position
	var dominating: Dictionary
	while true:
		if current < 0 or current in visited:
			body[BodyProperties.success] = false
			break
		visited[current] = true
		
		var hub := _hub(current)
		body[BodyProperties.position] = current
		
		var auto := bool(hub.get(HubProperties.auto, false))
		
		_project_hub_to_body(hub, body)
		
		var text_changed := !str(hub.get(HubProperties.text, "")).is_empty()
		if !auto and !dominating.is_empty():
			
			_project_hub_to_body(dominating, body, true)
			
			if !text_changed:
				text_changed = !str(hub.get(HubProperties.text, "")).is_empty()
		
		if !text_changed and !choice.is_empty():
			body[BodyProperties.text] = choice
		
		var branch := str(hub.get(HubProperties.branch, ""))
		if not branch.is_empty():
			body[BodyProperties.branch] = branch
		
		if !state.covered.get(current):
			_evaluate_statements(hub.get(HubProperties.statements, ""))
			_evaluate_callbacks(hub.get(HubProperties.callbacks, ""))
		state.covered[current] = true
		
		if auto:
			dominating = hub
		else:
			dominating = {}
			body[BodyProperties.success] = true
			break
		
		var available := list_choices(current, filter)
		if available.is_empty():
			body[BodyProperties.success] = false
			break
		current = int(available[0].get(ChoiceProperties.to_id, -1))
	
	return body

func _project_hub_to_body(hub:Dictionary, body:Dictionary[BodyProperties, Variant], is_domination:=false) -> void:
	var dominated = false
	if hub.has(HubProperties.speaker):
		body[BodyProperties.speaker] = hub[HubProperties.speaker]
		dominated = is_domination
	
	if hub.has(HubProperties.text):
		body[BodyProperties.text] = hub[HubProperties.text]
		dominated = is_domination
	
	if dominated:
		body[BodyProperties.dominated] = true

signal state_variable_changed(simulator:PSSimulator, variable:String, value)
const ASSIGNMENTS:PackedStringArray = [
	"=",
	"+=",
	"-=",
	"++",
	"--"
]
func _evaluate_statements(statements:String) -> void:
	var statements_arr := statements.split("\n", false)
	
	for s in statements_arr:
		var partial_left:String
		var partial_assignment:String
		var partial_right:String
		for a in ASSIGNMENTS:
			var index := s.find(a)
			if index == -1: continue
			partial_left = s.substr(0, index).strip_edges()
			partial_assignment = s.substr(index, a.length())
			partial_right = s.substr(index + a.length())
		
		if partial_assignment.is_empty():
			push_error("Could not locate assignment in statement: ", s)
			continue
		
		var value_right = 0
		if !partial_right.is_empty():
			value_right = _evaluate_string(partial_right)
		
		var value_left = state.scope.get(partial_left)
		
		match partial_assignment:
			"=":
				pass
			"+=":
				value_right = value_left + value_right
			"-=":
				value_right = value_left - value_right
			"++":
				value_right = value_left + 1
			"--":
				value_right = value_left - 1
		
		if value_right != value_left:
			state.scope.set(partial_left, value_right)
			state_variable_changed.emit(self, partial_left, value_right)

signal callback(simulator:PSSimulator, args:PackedStringArray)
func _evaluate_callbacks(callbacks:String) -> void:
	var callbacks_arr := callbacks.split("\n", false)
	for c in callbacks_arr:
		callback.emit(self, PackedStringArray(c.split(" ", false)))

func check_conditions(position:int) -> bool:
	var hub := _hub(position)
	
	var conditions := str(hub.get(HubProperties.conditions, "")).split("\n", false)
	for c in conditions:
		if !_evaluate_string(c):
			return false
		
	return true

func _evaluate_string(e:String) -> Variant:
	var result = null
	
	var names := names_to_trick_parser(e)
	for n in names:
		e.replace(n, str(state.scope.get(n)))
	expression.parse(e)
	result = expression.execute([], state.scope, true)
	if expression.has_execute_failed():
		pass
	
	return result

func names_to_trick_parser(e:String) -> PackedStringArray:
	#trick the expression parser into allowing names that haven't been declared yet
	#state.scope is a special type that fills in missing variables, so all valid variable names syntactically are valid names
	var invalid_name := RegEx.new()
	invalid_name.compile("[^a-zA-Z_]")
	var names:PackedStringArray
	var words := e.split(" ", false)
	for w in words:
		if invalid_name.search(w.substr(0, 1)):
			continue
		names.append(w)
	return names

#Return the position that would result from making choice `choice` from
#position `position`.
func get_destination(position: int, choice: String) -> int:
	if conversation == null:
		return -1
	for connection in conversation.get_connections_from(position):
		if str(connection.get(ChoiceProperties.from_choice, "")) == choice:
			return int(connection.get(ChoiceProperties.to_id, -1))
	return -1

#search iterator.
#
#given a filter, finds the first node that passes that filter.
#if `from` is passed as nonnegative, the filter won't start looking for
#matches until from is passed.
#basic usage is a loop that updates the start point and calls search repeatedly. 
func search(filter=no_filter, from:=-1) -> int:
	var from_passed := from < 0
	for id in conversation.hubs_by_id:
		if from >= 0 and id == from:
			from_passed = true
			continue
		
		if from_passed:
			if filter.call(conversation.hubs_by_id[id]):
				return id
	
	return -1

#Finds a start node in the conversation
func find_start(from:=-1) -> int:
	return search(
		has_property_or_flag.bind(
			HubProperties.start
		), 
		from
	)

#Creates a body at a starting node in the conversation.
#
#This comes with all the expected behavior of align as well. If start is an
#auto node, align_start will align out of it
func align_start(from:=-1, filter:=no_filter) -> Dictionary[BodyProperties, Variant]:
	var start := find_start(from)
	if start == -1:
		return {
			BodyProperties.success: false
		}
	var body: Dictionary[BodyProperties, Variant] = {}
	return _traverse(body, start, "", filter)
