extends Node

@export var color_picker:ColorPickerButton
@export var hub:GraphNode

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	color_picker.color_changed.connect(_on_color_updated)
	_on_color_updated(color_picker.color)

func _on_color_updated(color:Color):
	var new_box := StyleBoxFlat.new()
	new_box.bg_color = color
	hub.add_theme_stylebox_override("titlebar", new_box)
