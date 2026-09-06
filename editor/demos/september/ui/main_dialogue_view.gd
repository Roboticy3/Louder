extends Control

const EventType := ControllerEventLayer.EventType

@export var scrolling_dialogue_feed:ScrollingDialogueFeed
@export var scrolling_dialogue_thinking:ScrollingDialogueThinking

@export var controller_group := &"ps_controller"

func _ready() -> void:
	ControllerEventLayer.event.connect(_on_controller_event)

func _on_controller_event(_type: EventType, controller: PSController) -> void:
	update_thinking()
	
	if controller != ControllerEventLayer.get_connected_controller():
		return
	
	update_feed()

func update_feed():
	var connected_controller := ControllerEventLayer.get_connected_controller()
	
	if connected_controller == null:
		scrolling_dialogue_feed.clear()
		return
	
	scrolling_dialogue_feed.push_item(connected_controller)

func update_thinking():
	var thinking_set: Dictionary[String, bool]
	var connected_controller := ControllerEventLayer.get_connected_controller()
	for c in get_tree().get_nodes_in_group(controller_group):
		if c == connected_controller: continue
		
		if c is PSController:
			if !c.is_active(): continue
			
			var choices = c.get_choices()
			if choices.is_empty(): continue
			
			for speaker in c.speak_as:
				thinking_set[speaker] = true
	
	var thinking_array := thinking_set.keys()
	scrolling_dialogue_thinking.update_thinkers(thinking_array)
	
