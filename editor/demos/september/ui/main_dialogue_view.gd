extends Control

const EventType := ControllerEventLayer.EventType

@export var scrolling_dialogue_feed:ScrollingDialogueFeed
var current_controller:PSController

func _ready() -> void:
	ControllerEventLayer.event.connect(_on_controller_event)

func _on_controller_event(type: EventType, controller: PSController) -> void:
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
	pass
