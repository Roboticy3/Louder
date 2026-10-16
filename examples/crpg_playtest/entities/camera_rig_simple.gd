class_name CameraRigSimple extends Node3D

@export var target:Node3D

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta: float) -> void:
	if target:
		global_position = target.global_position
