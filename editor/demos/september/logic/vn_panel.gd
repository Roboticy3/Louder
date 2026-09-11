@tool
extends TextureRect

@export var sprite_directory := "res://demos/september/sprite/"
@export var texture_extension := ".png"

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	renamed.connect(_on_renamed)

func _on_renamed():
	texture = load(sprite_directory + name + texture_extension)
