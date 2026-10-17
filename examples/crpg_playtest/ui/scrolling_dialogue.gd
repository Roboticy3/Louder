class_name ScrollingDialogue extends Control

@export var scroll_container: ScrollContainer
@export var lines: VBoxContainer
@export var choices: VBoxContainer
@export var line_scene: PackedScene
@export var choice_scene: PackedScene
@export_file("*.json") var document_path := "res://conversations/jail.json"
@export_file("*.json") var config_path := "res://configs/config.json"
@export var start_tag := &"start"
@export var player_speaker := &"Benjamin"

var conversation := -1
var player_actor := -1

func _ready() -> void:
	LouderContextGlobal.load_conversation_config_file(config_path)
	conversation = LouderContextGlobal.load_conversation_file(document_path)
	LouderContextGlobal.add_actor(conversation, ["start"], 1, [player_speaker])

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
	if conversation != -1:
		LouderContextGlobal.select_choice(conversation, player_actor, row)

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
