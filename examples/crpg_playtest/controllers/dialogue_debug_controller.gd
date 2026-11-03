extends Node

@export var quick_possess:CharacterData

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	if quick_possess:
		Network.request_select(quick_possess.name)
		var a := preload("res://entities/character_avatar.tscn").instantiate()
		var b := preload("res://controllers/character_avatar_controller_single.tscn").instantiate()
		a.character_data = quick_possess
		b.character_data = quick_possess
		add_child(a)
		add_child(b)
