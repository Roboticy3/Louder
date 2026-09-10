class_name MainDialogueView extends Control

@export var scrolling_dialogue_feed:ScrollingDialogueFeed
@export var scrolling_dialogue_thinking:ScrollingDialogueThinking
@export var background_rect:TextureRect

@export var controller_group := &"ps_controller"
@export var filter_speak_as:PackedStringArray

func _ready() -> void:
	ControllerEventLayer.choice_event.connect(_on_controller_event)
	ControllerEventLayer.connect_event.connect(_on_controller_event)
	ControllerEventLayer.disconnect_event.connect(_on_controller_event)
	ControllerEventLayer.catch_up_event.connect(_on_controller_event)

func _on_controller_event(controller: PSController) -> void:
	update_thinking()
	
	if filter_speak_as.is_empty():
		if controller != ControllerEventLayer.get_connected_controller():
			return
	for s in controller.speak_as:
		var good := false
		if s in filter_speak_as:
			good = true
		if !good:
			return
	
	update_feed(controller)

func update_feed(c:PSController):	
	if c == null:
		scrolling_dialogue_feed.clear()
		return
	
	scrolling_dialogue_feed.push_item(c)

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
	
