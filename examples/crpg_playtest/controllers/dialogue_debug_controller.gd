extends Node

@export_file("*.json") var config_path := "res://configs/config.json"
@export_file("*.json") var document_path := "res://conversations/jail.json"


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	LouderContextGlobal.load_conversation_config_file(config_path)
	LouderContextGlobal.conversation_started.connect(_on_conversation_started)
	LouderContextGlobal.start_conversation_rpc.rpc_id.call_deferred(1, document_path, get_path())

func _on_conversation_started(c:int, source:NodePath):
	if source == get_path(): for s in get_tree().get_nodes_in_group("scrolling_dialogue"):
		s.call("_on_conversation_started", c, s.get_path())
	
