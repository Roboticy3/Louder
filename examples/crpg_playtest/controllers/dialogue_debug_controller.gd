extends Node

@export_file("*.json") var config_path := "res://configs/config.json"
@export_file("*.json") var document_path := "res://conversations/jail.json"


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	LouderContextGlobal.load_conversation_config_file(config_path)
	
