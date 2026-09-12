extends Node

@export_file_path("*.json") var conversation_path:String
@export var ps_conversation:PSConversation

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	var custom_path := CustomConversationLayer.conversation_path
	if FileAccess.file_exists(custom_path):
		conversation_path = custom_path
	
	if FileAccess.file_exists(conversation_path):
		ps_conversation.input_path = conversation_path
	
	if ps_conversation.is_node_ready():
		ps_conversation._ready()
