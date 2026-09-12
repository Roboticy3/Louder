extends Button

const MAIN_MENU_PATH := "res://demos/september/main_menu.tscn"
# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pressed.connect(_on_pressed)

func _on_pressed():
	NetworkLayer.disconnect_peer()
	ClientDataLayer.client_data.clear()
	get_tree().change_scene_to_file.bind(MAIN_MENU_PATH)
	
