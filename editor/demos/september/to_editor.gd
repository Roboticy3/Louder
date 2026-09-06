extends Button

@export var level:PackedScene

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pressed.connect(_on_pressed)

func _on_pressed():
	get_tree().change_scene_to_packed(level)
