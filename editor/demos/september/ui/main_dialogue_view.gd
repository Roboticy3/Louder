extends Control

@export var scrolling_dialogue_feed:ScrollingDialogueFeed
var current_controller:PSController

func _ready() -> void:
	ControllerEventLayer.event.connect(_on_controller_event)

func _on_controller_event(type: ControllerEventLayer.EventType, controller: PSController) -> void:
	if controller != ControllerEventLayer.get_connected_controller():
		return
	match type:
		ControllerEventLayer.EventType.CONNECT:
			scrolling_dialogue_feed.push_item(controller.data)
		ControllerEventLayer.EventType.CHOICE:
			scrolling_dialogue_feed.push_item(controller.data)
		ControllerEventLayer.EventType.DISCONNECT:
			scrolling_dialogue_feed.clear()
