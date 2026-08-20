extends Node

@export var menu_button:MenuButton
@export var button_name:String
@export var toggle_group:StringName
@export var accel:String

var bound_id := -1
func _ready():
	var popup := menu_button.get_popup()
	
	bound_id = popup.item_count
	popup.add_item(button_name)
	
	popup.id_pressed.connect(_on_id_pressed)

func _on_id_pressed(id:int):
	if id == bound_id:
		_toggle_speaker_colors()

func _toggle_speaker_colors():
	var nodes := get_tree().get_nodes_in_group(toggle_group)
	var any_visible := false
	for node in nodes:
		if node.get("visible"):
			any_visible = true
			break
	for node in nodes:
		node.set("visible", !any_visible)

func _input(event: InputEvent) -> void:
	if event.is_action_pressed(accel):
		_toggle_speaker_colors()
