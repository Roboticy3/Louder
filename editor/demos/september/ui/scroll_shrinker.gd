extends Node

@export var scroll_container:ScrollContainer
@export var inner_container:Container
@export var max_size_reference:Control

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	inner_container.resized.connect(shrink)
	shrink()

func shrink():
	var new_size := minf(inner_container.size.y, max_size_reference.size.y)
	scroll_container.size = Vector2(scroll_container.size.x, new_size)
	if new_size < max_size_reference.size.y:
		scroll_container.position = Vector2(scroll_container.position.x,(max_size_reference.size.y - new_size) * 0.5)
