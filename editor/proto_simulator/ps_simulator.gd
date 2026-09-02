class_name PSSimulator extends Node

#interface to iterate through conversations

const HubProperties := PSObjects.HubProperties
const BodyProperties := PSObjects.BodyProperties
const ChoiceProperties := PSObjects.ChoiceProperties
const SpeakerProperties := PSObjects.SpeakerProperties

@export var conversation:PSConversation
@export var state:PSDialogueState

func get_property(position: int, property: HubProperties, default=null):
	return default

#default filter for choice objects
func no_filter(_choice_data:Dictionary) -> bool: return true

#list choice objects from a position
func list(position: int, filter:=no_filter) -> Array[Dictionary]:
	return []

#align a "moving" body with the BodyProperties.choice field filled to the conversation.
#
#the list of choices is computed from `list` and the given choice must be contained within it.
#
#if the destination of the body has the HubProperties.auto field set to true, 
#the first valid choice is found, set to the body's choice, and taken as its motion
#recursively. Getting stuck in an auto hub with no path forward is treated the same
#as the original choice being invalid.
#
#returns a copy of the body with BodyProperties.success filled, and the position,
#branch, and choices updated to the body's current position in the conversation.
func align(body: Dictionary[BodyProperties, Variant], filter:=no_filter) -> Dictionary[BodyProperties, Variant]:
	return {
		BodyProperties.success: false
	}
