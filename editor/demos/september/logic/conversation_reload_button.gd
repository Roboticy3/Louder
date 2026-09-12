extends Button

@export var controller_group := &"ps_controller"
@export var conversation_path_edit:LineEdit
@export var conversation:PSConversation

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pressed.connect(_on_pressed)


func _on_pressed():
	var current_path := conversation.input_path
	var new_path := conversation_path_edit.text
	if current_path == new_path:
		return
	
	if !FileAccess.file_exists(new_path):
		return
	
	conversation.input_path = new_path
	conversation._ready()
	
	for c in get_tree().get_nodes_in_group(controller_group):
		if c is PSController:
			ControllerEventLayer.request_warp(c, c.get_start())
