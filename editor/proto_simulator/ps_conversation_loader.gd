class_name PSConversationLoader extends Node

@export var simulator: PSSimulator
@export var load_now := "res://demos/conversations/cordon.json"

#recieve an update immediately at conversation load for technical changes- simulation/editor etc.
#"_on_conversation_changed" is called on this group with the new conversation path passed as an argument.
@export var conversation_changed_listener_group := &"conversation_changed_listener"

var dialog: FileDialog

func _ready():
	dialog = FileDialog.new()
	dialog.title = "Open Conversation"
	dialog.access = FileDialog.ACCESS_RESOURCES
	dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	dialog.filters = PackedStringArray(["*.json"])
	dialog.file_selected.connect(_on_file_selected)
	add_child(dialog)
	
	if !load_now.is_empty():
		_on_file_selected(load_now)

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("graph_load"):
		dialog.show()
	if event.is_action_pressed("simulator_reset"):
		_on_file_selected(current_loaded)

var current_loaded := ""
func _on_file_selected(path: String) -> void:
	current_loaded = path
	get_tree().call_group.call_deferred(conversation_changed_listener_group, &"_on_conversation_changed", path)
