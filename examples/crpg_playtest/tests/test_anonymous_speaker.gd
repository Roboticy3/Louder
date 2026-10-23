extends Node

## Checks that a hub without a speaker of its own is spoken in the voice of
## whoever walks onto it, and that the line on screen says so.
##
## The conversation is a fixture of two hubs: an anonymous one the player starts
## on, and one that names a speaker of its own.

const DOCUMENT := "res://conversations/anonymous.json"
const CONFIG := "res://configs/config.json"
const PLAYER_SPEAKER := "Benjamin"
const OPENING_TEXT := "The room is quiet."

var _failures := 0


func _ready() -> void:
	var config_result: int = LouderContextGlobal.load_conversation_config_file(CONFIG)
	_check(config_result == 0, "the speaker config loaded (got %d)" % config_result)

	var scene := load("res://ui/scrolling_dialogue.tscn") as PackedScene
	_check(scene != null, "the dialogue scene loads")
	if scene == null:
		_finish()
		return

	var dialogue := scene.instantiate() as ScrollingDialogue
	add_child(dialogue)
	await get_tree().process_frame

	LouderContextGlobal.start_conversation_rpc(DOCUMENT, dialogue.get_path())
	for frame in 6:
		await get_tree().process_frame

	_check(dialogue.conversation != -1, "the dialogue joined the conversation")
	_check(dialogue.player_actor != -1, "the player's actor was added")
	_check(dialogue.lines.get_child_count() >= 1, "the opening line rendered")

	if dialogue.lines.get_child_count() >= 1:
		var line := dialogue.lines.get_child(0) as DialogueLine
		_check(line.text_label.text == OPENING_TEXT,
			"the line carries the text of the hub, saw \"%s\"" % line.text_label.text)
		# The hub names nobody, so the voice is the one of the actor who walked
		# onto it, and that is what the line is signed with.
		_check(line.speaker_label.text == PLAYER_SPEAKER,
			"the anonymous hub is spoken by the player, saw \"%s\"" % line.speaker_label.text)
		_check(line.speaker_label.visible, "the speaker is shown on the line")

	_finish()


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if !condition:
		_failures += 1


func _finish() -> void:
	if _failures > 0:
		get_tree().quit(1)
