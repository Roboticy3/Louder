extends Node

const ChoiceProperties := PSObjects.ChoiceProperties

@export var controller:PSController

func _ready() -> void:
	ControllerEventLayer.request_controller(controller)

func plonk():
	var choices := controller.get_choices()
	if choices.is_empty():
		return
	var i := randi_range(0, choices.size() - 1)
	ControllerEventLayer.request_choice(controller, choices[i].get(ChoiceProperties.from_choice, ""))
