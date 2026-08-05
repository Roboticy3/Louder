extends Node

@export var choices_container:Container
@export var simulator:PSSimulator

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	update_choices_container()

func update_choices_container():
	pass
