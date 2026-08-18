extends Node

@export var simulator: PSSimulator
@export var update_group := &"ps_conversation_view"

var dialog: FileDialog

func _ready():
	dialog = FileDialog.new()
	dialog.title = "Open Conversation"
	dialog.access = FileDialog.ACCESS_RESOURCES
	dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	dialog.filters = PackedStringArray(["*.json"])
	dialog.file_selected.connect(_on_file_selected)
	add_child(dialog)

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("graph_load"):
		dialog.show()

func _on_file_selected(path: String) -> void:
	simulator.conversation_path = path
	simulator._ready()
	get_tree().call_group.call_deferred(update_group, &"update")
