class_name CharacterAvatar extends Node3D

@export var character_data:CharacterData
@export var body:CharacterBody3D
@export var group := &"character_avatar"

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	add_to_group(group)


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
