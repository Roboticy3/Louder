class_name ScrollingDialogue extends Control

@export var scroll_container: ScrollContainer
@export var lines: VBoxContainer
@export var choices: VBoxContainer
@export var line_scene: PackedScene
@export var choice_scene: PackedScene
@export_file("*.json") var document_path := "res://conversations/jail.json"
@export var start_tag := &"start"
@export var player_speaker := &"Benjamin"

var _session: DialogueSession

func _ready() -> void:
	_session = DialogueSession.new()
	_session.line_revealed.connect(_on_line_revealed)
	_session.choices_revealed.connect(_on_choices_revealed)
	_session.callback_raised.connect(_on_callback_raised)
	_session.conversation_finished.connect(_on_conversation_finished)

	var failure := _session.start(document_path, start_tag, player_speaker)
	if !failure.is_empty():
		push_error(failure)

func _on_line_revealed(speaker: String, text: String, background: bool) -> void:
	var line := line_scene.instantiate() as DialogueLine
	line.setup(speaker, text, background)
	lines.add_child(line)
	_scroll_to_bottom()

func _on_choices_revealed(texts: PackedStringArray) -> void:
	for child in choices.get_children():
		choices.remove_child(child)
		child.queue_free()

	for row in texts.size():
		var choice := choice_scene.instantiate() as DialogueChoice
		choice.setup(row, texts[row])
		choice.chosen.connect(_on_choice_chosen)
		choices.add_child(choice)

func _on_choice_chosen(row: int) -> void:
	_session.select_choice(row)

func _on_callback_raised(callback: String) -> void:
	# Callbacks are the engine's hook for game state. Nothing acts on them yet, so
	# they are printed to keep the playtest visible.
	print("Dialogue callback: %s" % callback)

func _on_conversation_finished() -> void:
	print("Dialogue conversation finished.")

func _scroll_to_bottom() -> void:
	# New children have not been laid out yet, so the scroll range is the one
	# from before the line was added. Reading it next frame is the difference
	# between following the log and stopping a line short of it.
	await get_tree().process_frame
	scroll_container.scroll_vertical = int(scroll_container.get_v_scroll_bar().max_value)
