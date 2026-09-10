extends Control

@export var of:Control

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	of.resized.connect(_on_resized, CONNECT_DEFERRED)
	_on_resized.call_deferred()

func _on_resized():
	custom_minimum_size.y = of.size.y
