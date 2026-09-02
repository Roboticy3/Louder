class_name PSController extends Node

const BodyProperties = PSObjects.BodyProperties
const ChoiceProperties = PSObjects.ChoiceProperties
const HubProperties = PSObjects.HubProperties

@export var group := &"ps_controller"
@export var simulator:PSSimulator
@export var config:PSConfig
@export var speak_as:PackedStringArray
@export var listen_to:PackedStringArray
@export var mute:bool

#state of the controller.
var data:Dictionary[BodyProperties, Variant]

func _ready() -> void:
	add_to_group(group)
	
func is_active():
	return !data.is_empty()

#Asynchronous Multiple Speakers
#1. Hide choices pointing to speakers not owned in `my_speakers`
func _filter_valid_choice(c:Dictionary) -> bool:
	if mute: return false
	
	var c_position := int(c.get(ChoiceProperties.to_id, -1))
	var c_speaker := str(simulator.get_property(c_position, HubProperties.speaker))
	if !(c_speaker in config.speakers_by_name):
		return true
	if c_speaker in speak_as:
		return true
	if !config.is_ownable(c_speaker):
		if !config.is_public(c_speaker) and !(c_speaker in listen_to):
			return false
		return true
	return false

func _filter_visible_node(position:int) -> bool:
	var n_speaker := str(simulator.get_property(position, HubProperties.speaker))
	if !(n_speaker in config.speakers_by_name):
		return true
	if n_speaker in listen_to:
		return true
	if config.is_public(n_speaker):
		return true
	return false

func traverse(choice:String) -> bool:
	data[BodyProperties.choice] = choice
	data = simulator.align(data, _filter_valid_choice)
	return data.get(BodyProperties.success, false)

func get_text(property:String) -> String:
	return str(data.get(property))

func get_choices() -> Array:
	return data.get(BodyProperties.choices, [])

func _filter_thinking() -> Array:
	var thinking := {}
	for controller in get_tree().get_nodes_in_group(group):
		if controller == self: continue
		if controller is PSController and !controller.get_choices().is_empty():
			for speaker in controller.speak_as:
				thinking[speaker] = true
	return thinking.keys()

func get_position() -> int:
	return data.get(BodyProperties.position, -1)

func get_branch() -> String:
	return data.get(BodyProperties.branch, "")

func _get_property(property:HubProperties, default=null) -> Variant:
	return simulator.get_property(get_position(), property, default)
