extends Button

const MAIN_MENU_PATH := "res://demos/september/main_menu.tscn"
# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pressed.connect(_on_pressed)

func _on_pressed():
	NetworkLayer.disconnect_peer()
	ClientDataLayer.client_data.clear()
	ControllerEventLayer.connected_controller = null
	CustomConversationLayer.conversation_path = ""
	CustomConversationLayer.config_path = ""
	IdSystem.clear_ids()
	get_tree().change_scene_to_file.call_deferred(MAIN_MENU_PATH)
	
