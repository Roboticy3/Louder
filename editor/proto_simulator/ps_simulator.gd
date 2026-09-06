class_name PSSimulator extends Node

#interface to iterate through conversations

const HubProperties := PSObjects.HubProperties
const BodyProperties := PSObjects.BodyProperties
const ChoiceProperties := PSObjects.ChoiceProperties
const SpeakerProperties := PSObjects.SpeakerProperties

const DEFAULT_BRANCH := "main"
const CANNED_CHOICES: Array[Dictionary] = [
	{
		ChoiceProperties.from_choice: "Choice 1",
	},
	{
		ChoiceProperties.from_choice: "Choice 2",
	},
	{
		ChoiceProperties.from_choice: "Choice 3 (Branch: breakout)",
		"branch": "breakout"
	},
]
const CANNED_STATE_0: Dictionary[BodyProperties, Variant] = {
	BodyProperties.speaker: "Canned State 0",
	BodyProperties.text: "Welcome to Canned State 0!",
	BodyProperties.position: 0,
	BodyProperties.branch: DEFAULT_BRANCH,
	BodyProperties.success: true,
	BodyProperties.choices: CANNED_CHOICES,
}
const CANNED_STATE_1: Dictionary[BodyProperties, Variant] = {
	BodyProperties.speaker: "Canned State 1",
	BodyProperties.text: "You've moved to Canned State 1! If you choice (Branch: breakout). Any choice goes back to position 0 and choice 1 or 2 will go back to (Branch: main)",
	BodyProperties.position: 1,
	BodyProperties.success: true,
	BodyProperties.choices: CANNED_CHOICES,
}

@export var group := &"ps_simulator"
@export var conversation:PSConversation
@export var state:PSDialogueState

func _ready():
	add_to_group(group)

#TODO Get a property from a position in the conversation.
func get_property(position: int, property: HubProperties, default=null) -> Variant:
	match position:
		0:
			return CANNED_STATE_0.get(property, default)
		1:
			return CANNED_STATE_1.get(property, default)
		_:
			return default

#Filters
func no_filter(_data:Dictionary) -> bool: return true
func has_property_or_flag(data:Dictionary, property): return true if data.get(property, false) else false

#TODO List choices available at a certain position based on a given filter.
func list(position: int, filter:=no_filter) -> Array[Dictionary]:
	return CANNED_CHOICES

#TODO
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
#The `filter` argument is relevant when the choice leads to a hub with
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
	var from_position: int = body.get(BodyProperties.position, -1)
	var choices := list(from_position)
	
	var to_position := get_destination(from_position, choice)
	var canned_branch := "main"
	for c in choices:
		if c.get(ChoiceProperties.from_choice, "") == choice:
			canned_branch = c.get("branch", "")
			if !canned_branch.is_empty():
				print("PSSimulator branching!")
	
	body = body.duplicate()
	if to_position >= 0:
		match to_position:
			0:
				body = CANNED_STATE_0.duplicate()
			1:
				body = CANNED_STATE_1.duplicate()
		
		if !canned_branch.is_empty():
			body[BodyProperties.branch] = canned_branch
		pass
	else:
		body[BodyProperties.success] = false
	
	return body

#TODO Return the position that would result from making choice `choice` from
#position `position`.
func get_destination(position: int, choice: String) -> int:
	return (position + 1) % 2

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

#TODO Creates a body at a starting node in the conversation.
func align_start(from:=-1) -> Dictionary[BodyProperties, Variant]:
	var start := find_start(from)
	if start == -1:
		return {
			BodyProperties.success: false
		}
	else:
		var data := CANNED_STATE_0.duplicate()
		data[BodyProperties.success] = true
		return data
