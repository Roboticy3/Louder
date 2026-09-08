extends Node

@export var basic_choice: AudioStreamPlayer
@export var special_choice: AudioStreamPlayer
# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	ControllerEventLayer.choice_event.connect(_on_choice)

func _on_choice(controller:PSController):
	if controller.get_success():
		if controller.is_listen_to():
			basic_choice.play()
		else:
			basic_choice.play()
