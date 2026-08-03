extends Node

@export var simulator:PSConversation
@export var path:="user://test.json"

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	simulator.load_from_json(path)
