extends Control

@export var scrolling_dialogue_feed:ScrollingDialogueFeed
var current_controller:PSController

func _ready() -> void:
	ControllerEventLayer.choice_event.connect(_on_controller_choice)
	ControllerEventLayer.connect_event.connect(_on_controller_connect)
	ControllerEventLayer.disconnect_event.connect(_on_controller_disconnect)
	ControllerEventLayer.catch_up_event.connect(_on_controller_catch_up)

func _on_controller_choice(controller: PSController) -> void:
	if !_update_visibility(controller): return
	scrolling_dialogue_feed.push_item(controller)

func _on_controller_catch_up(controller: PSController) -> void:
	if !_update_visibility(controller): return
	scrolling_dialogue_feed.push_item(controller)

func _on_controller_disconnect(controller: PSController) -> void:
	if !_update_visibility(controller): return
	scrolling_dialogue_feed.clear()

func _on_controller_connect(controller: PSController) -> void:
	_update_visibility(controller)

func _update_visibility(controller: PSController) -> bool:
	current_controller = ControllerEventLayer.get_connected_controller()
	if current_controller == null or current_controller == controller:
		return false
	if current_controller.can_spectate(controller):
		visible = true
	else:
		visible = false
		return false
	return true

func update_feed(controller:PSController):
	if controller == null:
		scrolling_dialogue_feed.clear()
		visible = false
		return
	
	scrolling_dialogue_feed.push_item(controller)
