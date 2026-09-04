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
	BodyProperties.success: true,
	BodyProperties.branch: DEFAULT_BRANCH,
	BodyProperties.choices: CANNED_CHOICES,
}
const CANNED_STATE_1: Dictionary[BodyProperties, Variant] = {
	BodyProperties.speaker: "Canned State 1",
	BodyProperties.text: "You've moved to Canned State 1! If you choice (Branch: breakout), any choice now should return to the main branch.",
	BodyProperties.position: 0,
	BodyProperties.success: true,
	BodyProperties.choices: CANNED_CHOICES,
}

@export var group := &"ps_simulator"
@export var conversation:PSConversation
@export var state:PSDialogueState

func _ready():
	add_to_group(group)

func get_property(position: int, property: HubProperties, default=null) -> Variant:
	match position:
		0:
			return CANNED_STATE_0.get(property, default)
		1:
			return CANNED_STATE_1.get(property, default)
		_:
			return default

#default filter for objects
func no_filter(_data:Dictionary) -> bool: return true

func has_property_or_flag(data:Dictionary, property): return true if data.get(property, false) else false

#list choice objects from a position
func list(position: int, filter:=no_filter) -> Array[Dictionary]:
	return CANNED_CHOICES

#align a "moving" body with the BodyProperties.choice field filled to the conversation.
#
#the list of choices is computed from `list` and the given choice must be contained within it.
#user provided filter is *not* passed here.
#
#if the destination of the body has the HubProperties.auto field set to true, 
#the first valid choice is found, set to the body's choice, and taken as its motion
#recursively. Getting stuck in an auto hub with no path forward is treated the same
#as the original choice being invalid.
#
#returns a copy of the body with BodyProperties.success filled, and the position,
#branch, and choices updated to the body's current position in the conversation.
func align(body: Dictionary[BodyProperties, Variant], choice: String, filter:=no_filter) -> Dictionary[BodyProperties, Variant]:
	var from_position: int = body.get(BodyProperties.position, -1)
	var choices := list(from_position)
	
	var to_position: int = -1
	var canned_branch := ""
	for c in choices:
		if c.get(ChoiceProperties.from_choice, "") == choice:
			to_position = c.get(ChoiceProperties.to_id, -1)
			canned_branch = c.get("branch", "")
	
	body = body.duplicate()
	if to_position >= 0:
		match to_position:
			0:
				body = CANNED_STATE_0.duplicate()
			1:
				body = CANNED_STATE_1.duplicate()
		if !canned_branch.is_empty():
			body[BodyProperties.branch] = canned_branch
	else:
		body[BodyProperties.success] = false
	
	return body

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

func find_start(from:=-1) -> int:
	return search(
		has_property_or_flag.bind(
			HubProperties.start
		), 
		from
	)

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
