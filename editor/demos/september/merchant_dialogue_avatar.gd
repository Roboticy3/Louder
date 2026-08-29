extends Node

@export var conversation_group := &"ps_conversation"
@export var config_group := &"ps_config"

@export var simulator:PSSimulator

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	set_multiplayer_authority(int(name), true)

func join_conversation(c:PSConversation):
	simulator._on_conversation_changed_to_node(c)
