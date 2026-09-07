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
func list(position: int, filter:=no_filter) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	if conversation == null:
		return result
	for connection in conversation.get_connections_from(position):
		var to_id: int = int(connection.get(ChoiceProperties.to_id, -1))
		
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
#`choice` is validated against `list` on the body's position, but the user's
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
	for c in list(from_position):
		if str(c.get(ChoiceProperties.from_choice, "")) == choice:
			valid = true
			break
	if not valid:
		result[BodyProperties.success] = false
		return result
	
	result = _traverse(result, get_destination(from_position, choice), choice, filter)
	if result.get(BodyProperties.branch, DEFAULT_BRANCH) != from_branch:
		result[BodyProperties.branch_changed] = true
	
	state.covered[result.get(BodyProperties.position, -1)] = true
	
	return result

#Move `body` to `position`, then auto-advance through any auto hubs until a
#non-auto hub is reached, or the conversation runs dry. `taken_choice` is the
#choice that moved the body here ("" when starting a conversation).
func _traverse(body: Dictionary[BodyProperties, Variant], position: int, choice: String, filter: Callable) -> Dictionary[BodyProperties, Variant]:
	if position == 21:
		pass
	
	var visited := {}
	var current := position
	while true:
		if current < 0 or current in visited:
			body[BodyProperties.success] = false
			break
		visited[current] = true
		
		var hub := _hub(current)
		body[BodyProperties.position] = current
		if hub.has(HubProperties.speaker):
			body[BodyProperties.speaker] = hub[HubProperties.speaker]
		
		if hub.has(HubProperties.text):
			body[BodyProperties.text] = hub[HubProperties.text]
		elif !choice.is_empty():
			body[BodyProperties.text] = choice
		
		var branch := str(hub.get(HubProperties.branch, ""))
		if not branch.is_empty():
			body[BodyProperties.branch] = branch
		
		_evaluate_statements(hub.get(HubProperties.statements, ""))
		_evaluate_callbacks(hub.get(HubProperties.callbacks, ""))
		
		if not (hub.get(HubProperties.auto, false) as bool):
			body[BodyProperties.success] = true
			break
		
		var available := list(current, filter)
		if available.is_empty():
			body[BodyProperties.success] = false
			break
		current = int(available[0].get(ChoiceProperties.to_id, -1))
	
	return body

const ASSIGNMENTS:PackedStringArray = [
	"=",
	"+=",
	"-=",
	"++",
	"--"
]
func _evaluate_statements(statements:String) -> void:
	var statements_arr := statements.split("\n", true)
	
	for s in statements_arr:
		var partial_left:String
		var partial_assignment:String
		var partial_right:String
		for a in ASSIGNMENTS:
			var index := s.find(a)
			if index == -1: continue
			partial_left = s.substr(0, index)
			partial_assignment = s.substr(index, a.length())
			partial_right = s.substr(index + a.length())
		
		if partial_assignment.is_empty():
			push_error("Could not locate assignment in statement: ", s)
			continue
		
		var value_right = 0
		if !partial_right.is_empty():
			expression.parse(partial_right, state.scope.get_names())
			value_right = expression.execute([], state.scope, true, false)
		
		var value_left = state.scope.get(partial_left)
		match partial_assignment:
			"=":
				state.scope.set(partial_left, value_right)
			"+=":
				state.scope.set(partial_left, value_left + value_right)
			"-=":
				state.scope.set(partial_left, value_left - value_right)
			"++":
				state.scope.set(partial_left, value_left + 1)
			"--":
				state.scope.set(partial_left, value_left - 1)

signal callback(args:PackedStringArray)
func _evaluate_callbacks(callbacks:String) -> void:
	var callbacks_arr := callbacks.split("\n", false)
	for c in callbacks_arr:
		callback.emit(PackedStringArray(c.split(" ", false)))

func check_conditions(position:int) -> bool:
	var hub := _hub(position)
	var passing := true
	var conditions := str(hub.get(HubProperties.conditions, "")).split("\n", false)
	
	for c in conditions:
		expression.parse(c, state.scope.get_names())
		passing = passing and expression.execute([], state.scope, true, true)
	
	return passing

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
