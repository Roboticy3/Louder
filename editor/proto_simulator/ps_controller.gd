class_name PSController extends Node

@export var group := &"ps_controller"
@export var simulator:PSSimulator
@export var config:PSConfig
@export var speak_as:PackedStringArray
@export var listen_to:PackedStringArray
@export var mute:bool

var data:Dictionary

func _ready() -> void:
	add_to_group(group)
	
	await get_tree().process_frame
	
	if simulator.is_node_ready():
		update()

signal updated()
func update():
	if !is_instance_valid(simulator):
		push_error("Updated PSController ", self, " with no simulator.")
		return
	
	if !_filter_visible_node(simulator.get_position()):
		return
	
	data[PSSimulator.HUB_SPKR] = simulator.get_current_property(PSSimulator.HUB_SPKR)
	data[PSSimulator.HUB_TXTC] = simulator.get_current_property(PSSimulator.HUB_TXTC)
	data[PSSimulator.HUB_CHCS] = simulator.list_choices_data(_filter_valid_choice)
	updated.emit()
	
func is_active():
	return !data.is_empty()

#Asynchronous Multiple Speakers
#1. Hide choices pointing to speakers not owned in `my_speakers`
func _filter_valid_choice(c:Dictionary) -> bool:
	if mute: return false
	var c_speaker := str(simulator.get_property(int(c["to_id"]), PSSimulator.HUB_SPKR))
	if !(c_speaker in config.speakers_by_name):
		return true
	if c_speaker in speak_as:
		return true
	if !config.is_ownable(c_speaker):
		return true
	return false

func _filter_visible_node(position:int) -> bool:
	var n_speaker := str(simulator.get_property(position, PSSimulator.HUB_SPKR))
	if !(n_speaker in config.speakers_by_name):
		return true
	if n_speaker in listen_to:
		return true
	if config.is_public(n_speaker):
		return true
	return false

func traverse(choice:String) -> bool:
	#TODO - maybe some validation here that the choice is accessible to *this*
	#controller and not just the simulator overall?
	return simulator.traverse(choice)

func get_text(property:String) -> String:
	return str(data.get(property))

func get_choices() -> Array:
	var choices = data.get(PSSimulator.HUB_CHCS)
	if choices is Array:
		return choices
	return []
