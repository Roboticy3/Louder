extends Control

const EventType := ControllerEventLayer.EventType

@export var scrolling_dialogue_feed:ScrollingDialogueFeed
var current_controller:PSController

func _ready() -> void:
	ControllerEventLayer.event.connect(_on_controller_event)

func _on_controller_event(type: EventType, controller: PSController) -> void:
	current_controller = ControllerEventLayer.get_connected_controller()
	if current_controller == null or current_controller == controller:
		return
	if current_controller.can_spectate(controller):
		visible = true
	else:
		visible = false
		return
	
	match type:
		ControllerEventLayer.EventType.CATCH_UP:
			scrolling_dialogue_feed.push_item(controller)
		ControllerEventLayer.EventType.CHOICE:
			scrolling_dialogue_feed.push_item(controller)
		ControllerEventLayer.EventType.DISCONNECT:
			scrolling_dialogue_feed.clear()
