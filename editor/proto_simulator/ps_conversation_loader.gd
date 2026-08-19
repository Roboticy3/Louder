class_name PSConversationLoader extends Node

@export var simulator: PSSimulator

#recieve a delayed update after conversation load for visuals
#"update" is called on this group
@export var update_group := &"ps_conversation_view"

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

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("graph_load"):
		dialog.show()

func _on_file_selected(path: String) -> void:
	get_tree().call_group.call_deferred(conversation_changed_listener_group, &"_on_conversation_changed", path)
	get_tree().call_group.call_deferred(update_group, &"update")
