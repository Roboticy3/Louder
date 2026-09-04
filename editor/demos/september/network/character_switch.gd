extends Node

const ClientProperties = ClientDataLayer.ClientProperties

@export var character:String

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	ClientDataLayer.data_updated.connect(_on_data_updated)
	toggle_based_on_character()

func _on_data_updated(_auth: int, _data: Dictionary):
	toggle_based_on_character()

func toggle_based_on_character():
	var my_data := ClientDataLayer.get_local_data()
	var character_data: String = my_data.get(ClientProperties.character, "")
	var enable := !character_data.is_empty() and character_data == character
	set("visible", enable)
