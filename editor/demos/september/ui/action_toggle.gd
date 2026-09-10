extends Node

@export var action:StringName = "debug_menu_1"

func _input(event: InputEvent) -> void:
	if event.is_action_pressed(action):
		var is_visible = get("visible") 
		set("visible", false if is_visible else true)
		pass
