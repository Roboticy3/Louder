extends Node

@export var conversation_group := &"ps_conversation"
@export var config_group := &"ps_config"

@export var simulator:PSSimulator
@export var controller:PSController

#Happens a little before _ready, and before parents emit the child_entered_tree signal
func _enter_tree() -> void:
	set_multiplayer_authority(int(name), true)

func _ready() -> void:
	var c = get_tree().get_first_node_in_group(conversation_group)
	if c is PSConversation:
		join_conversation(c)

func join_conversation(c:PSConversation):
	simulator._on_conversation_changed_to_node(c)
