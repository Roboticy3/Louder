extends Node

@export var simulator: PSSimulator
@export var view: Node

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
	if not simulator or not view:
		push_error("ps_load_conversation: simulator and view must be exported")
		return
	if not simulator.conversation.load_from_json(path):
		return
	simulator.position = _find_start_id()
	simulator.state = PSConversationState.new()
	view.update()

func _find_start_id() -> int:
	var hubs := simulator.conversation.hubs_by_id
	if hubs.is_empty():
		return 0
	var start := int(hubs.keys()[0])
	for id in hubs:
		if hubs[id].get(PSSimulator.HUB_SCHK):
			return id
	return start
