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
	if position == -1:
		return false
	var n_speaker := str(simulator.get_property(position, HubProperties.speaker))
	if !(n_speaker in config.speakers_by_name):
		return true
	if n_speaker in listen_to:
		return true
	if config.is_public(n_speaker):
		return true
	return false

func start() -> bool:
	data = simulator.align_start()
	return data.get(BodyProperties.success, false)

func traverse(choice:String) -> bool:
	data = simulator.align(data, choice, _filter_valid_choice)
	return data.get(BodyProperties.success, false)

func follow(other:PSController) -> void:
	data = other.data.duplicate()

func should_follow(other:PSController) -> bool:
	if !other or !other.simulator or other == self: return false
	var other_position := other.get_position()
	return other.get_branch() == get_branch() and _filter_visible_node(other_position)

func can_spectate(other:PSController) -> bool:
	if !other or !other.simulator or other == self: return false
	var other_position := other.get_position()
	return other.get_branch() != get_branch() and _filter_visible_node(other_position)

func get_text(property:String) -> String:
	return str(data.get(property))

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
	return data.get(BodyProperties.branch, PSSimulator.DEFAULT_BRANCH)

func get_choices():
	return simulator.list(get_position(), _filter_valid_choice)

func branch_changed() -> bool:
	return data.get(BodyProperties.branch_changed, false)

func _get_property(property:HubProperties, default=null) -> Variant:
	return simulator.get_property(get_position(), property, default)

func find_simulator() -> PSSimulator:
	var node := get_parent()
	while node:
		if node is PSSimulator:
			simulator = node
			return node
		node = node.get_parent()
	return null
