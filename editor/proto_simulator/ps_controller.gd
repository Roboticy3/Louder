class_name PSController extends Node

const CTRL_THNK := "thinking"

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
signal show_end()
func update():
	if !is_instance_valid(simulator):
		push_error("Updated PSController ", self, " with no simulator.")
		return
	
	var text_visible := _filter_visible_node(simulator.get_position())
	
	if text_visible:
		simulator.merge_current_into(data)
		
	data[PSSimulator.HUB_CHCS] = simulator.list_choices_data(_filter_valid_choice) if text_visible else []
	
	#sync with the rest of the controllers getting their choices
	await get_tree().process_frame
	
	#now we have enough info to see who's thinking
	data[CTRL_THNK] = _filter_thinking()
	
	updated.emit()
	if get_thinking().is_empty() and get_choices().is_empty():
		show_end.emit()
	
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

func _filter_thinking() -> Array:
	var thinking := {}
	for controller in get_tree().get_nodes_in_group(group):
		if controller == self: continue
		if controller is PSController and !controller.get_choices().is_empty():
			for speaker in controller.speak_as:
				thinking[speaker] = true
	return thinking.keys()

func get_thinking() -> Array:
	var thinking = data.get(CTRL_THNK)
	if thinking is Array:
		return thinking
	return []

func get_branch() -> String:
	return data.get(PSSimulator.HUB_BRCH, "")

func end():
	#TODO Disconnect from the simulator
	pass
