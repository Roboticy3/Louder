extends Control

@export var container:Control
@export var ratio := 7.0

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	_on_container_resized()
	if container:
		container.resized.connect(_on_container_resized)

func _on_container_resized():
	if container:
		custom_minimum_size.y = container.size.y / ratio
	else:
		custom_minimum_size.y = get_viewport_rect().size.y / ratio
	pass
