extends Node

@export var group := &"ps_conversation_view"

@export var simulator: PSSimulator

#Speaker visibility data
@export var config: PSConfig
@export var my_speakers:PackedStringArray
@export var mute:bool

var _components: Array[Node] = []

func _ready() -> void:
	add_to_group(group)
	
	_collect_ui_components(self, _components)
	for component in _components:
		if component is PSChoicesContainer:
			component.choice_pressed.connect(_on_choice_pressed)
	
	await get_tree().process_frame
	
	#is simulator ready to feed choices?
	if simulator.is_node_ready():
		update()
	simulator.ready.connect(update, CONNECT_DEFERRED)
	
	simulator.choice_made.connect(update, CONNECT_DEFERRED)

func _collect_ui_components(node: Node, components: Array[Node]) -> void:
	for child in node.get_children():
		if child.is_in_group("ui_component"):
			components.append(child)
		_collect_ui_components(child, components)

func update() -> void:
	if !is_instance_valid(simulator):
		push_error("Updated PSConversationView ", self, " with no simulator.")
		return

	for component in _components:
		if component is RichTextLabel:
			component.text = simulator.get_current_property(component.name)
		elif component is PSChoicesContainer:
			component.set_choices(simulator.list_choices(_filter_valid_speakers))


#Asynchronous Multiple Speakers
#1. Hide choices pointing to speakers not owned in `my_speakers`
func _filter_valid_speakers(c:Dictionary) -> bool:
	if mute: return false
	var c_speaker := str(simulator.get_property(int(c["to_id"]), "speaker"))
	if !(c_speaker in config.speakers_by_name):
		return true
	if c_speaker in my_speakers:
		return true
	if config.is_npc(c_speaker):
		return true
	return false

func _on_choice_pressed(choice: String) -> void:
	simulator.traverse(choice)
