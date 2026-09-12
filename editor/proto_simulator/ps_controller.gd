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
	return (simulator is PSSimulator) and !data.is_empty()

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
	
	var n_private := bool(simulator.get_property(c_position, HubProperties.private, false))
	var n_recipient := str(simulator.get_property(c_position, HubProperties.recipient, ""))
	if n_private:
		return n_recipient in speak_as
	
	if !config.is_ownable(c_speaker):
		if !config.is_public(c_speaker) and !(c_speaker in listen_to):
			return false
		return true
	return false

func _filter_visible_node(position:int) -> bool:
	if position == -1:
		return false
	var n_private := bool(simulator.get_property(position, HubProperties.private, false))
	var n_recipient := str(simulator.get_property(position, HubProperties.recipient, ""))
	if n_private:
		return n_recipient in speak_as
	
	var n_speaker := str(simulator.get_property(position, HubProperties.speaker))
	if !(n_speaker in config.speakers_by_name):
		return true
	if n_speaker in listen_to:
		return true
	if config.is_public(n_speaker):
		return true
	return false

func get_start() -> int:
	var q := simulator.align_start(-1, _filter_valid_choice)
	return int(q.get(BodyProperties.position))

func start() -> bool:
	data = simulator.align_start(-1, _filter_valid_choice)
	return data.get(BodyProperties.success, false)

func traverse(choice:String) -> bool:
	data = simulator.align(data, choice, _filter_valid_choice)
	return data.get(BodyProperties.success, false)

func warp(position:int) -> bool:
	data = simulator.warp(data, position, _filter_valid_choice)
	return data.get(BodyProperties.success, false)

func follow(other:PSController) -> void:
	data = other.data.duplicate()

func should_follow(other:PSController) -> bool:
	if !other or !other.simulator or other == self: return false
	var other_position := other.get_position()
	return other.get_branch() == get_branch() and _filter_visible_node(other_position)

func choice_follow(other:PSController, pre_body:Dictionary[BodyProperties, Variant], from_choice:String) -> bool:
	if get_branch() == other.get_branch():
		var post_body := simulator.align(pre_body, from_choice, _filter_valid_choice)
		data = post_body
		return post_body.get(BodyProperties.success, true)
	return false

func can_spectate(other:PSController) -> bool:
	if !other or !other.simulator or other == self: return false
	var other_position := other.get_position()
	return other.get_branch() != get_branch() and _filter_visible_node(other_position)

func get_text() -> String:
	return str(data.get(BodyProperties.text, ""))

func get_position() -> int:
	return data.get(BodyProperties.position, -1)

func get_branch() -> String:
	return data.get(BodyProperties.branch, PSSimulator.DEFAULT_BRANCH)


func get_speaker() -> String:
	return data.get(BodyProperties.speaker, "")

func is_listen_to():
	return get_speaker() in listen_to

func is_successful() -> bool:
	return data.get(BodyProperties.success, true)

func is_dominated() -> bool:
	return data.get(BodyProperties.dominated, false)

func get_choices() -> Array[Dictionary]:
	return simulator.list_choices(get_position(), _filter_valid_choice)

func branch_changed() -> bool:
	return data.get(BodyProperties.branch_changed, false)

func _filter_thinking() -> Array:
	var thinking := {}
	for controller in get_tree().get_nodes_in_group(group):
		if controller == self: continue
		if controller is PSController and !controller.get_choices().is_empty():
			for speaker in controller.speak_as:
				thinking[speaker] = true
	return thinking.keys()

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
