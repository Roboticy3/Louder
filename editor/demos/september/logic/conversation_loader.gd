extends Node

@export_file_path("*.json") var conversation_path:String
@export_file_path("*.json") var config_path:String
@export var ps_conversation:PSConversation
@export var ps_config:PSConfig

signal all_done

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	var cconv := CustomConversationLayer.conversation_path
	if FileAccess.file_exists(cconv):
		conversation_path = cconv
	
	if FileAccess.file_exists(conversation_path):
		ps_conversation.input_path = conversation_path
	
	if ps_conversation.is_node_ready():
		ps_conversation._ready()
	
	var cconf := CustomConversationLayer.config_path
	if FileAccess.file_exists(cconf):
		config_path = cconf
	
	if FileAccess.file_exists(config_path):
		ps_config.config_path = config_path
	
	if ps_conversation.is_node_ready():
		ps_config._ready()
	
	all_done.emit()
