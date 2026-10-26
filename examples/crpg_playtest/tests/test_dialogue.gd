extends Node

## Drives the real dialogue scene headlessly and checks that the engine's flow
## actually reaches the UI.
##
## The flow is the world's: the scene is added, a conversation is started
## against it the way character_avatar_controller_single starts one, and lines
## and choices come out the other side. What is checked here is that the buttons
## on screen are the choices the engine is actually offering, and that pressing
## one moves the conversation on.

const SCENE := "res://ui/scrolling_dialogue.tscn"
const DOCUMENT := "res://conversations/marrow.json"
const CONFIG := "res://configs/config.json"
const STEPS := 6

var _dialogue: Control
var _failures := 0


func _ready() -> void:
	_check(has_node("/root/LouderContextGlobal"), "LouderContextGlobal is loaded as an autoload")

	# The dialogue's gate asks who the local peer plays: rig the lobby so this
	# process plays Benjamin, the speaker the scene is exported with.
	var benjamin: CharacterData = null
	for character in Network.get_characters():
		if character.name == "Benjamin":
			benjamin = character
	_check(benjamin != null, "Benjamin is in the catalog")
	if benjamin == null:
		_finish()
		return
	Network._apply_allocations({1: benjamin.get_id()})

	var config_result: int = LouderContextGlobal.load_conversation_config_file(CONFIG)
	_check(config_result == 0, "the speaker config loaded (got %d)" % config_result)

	var scene := load(SCENE) as PackedScene
	_check(scene != null, "the dialogue scene loads")
	if scene != null:
		_dialogue = scene.instantiate() as Control
		add_child(_dialogue)
		await get_tree().process_frame

		# What the controller sends when the player's avatar reaches someone.
		LouderContextGlobal.start_conversation_rpc(DOCUMENT, _dialogue.get_path(), benjamin.name)
		for frame in 6:
			await get_tree().process_frame

		var conversation: int = _dialogue.conversation
		var actor: int = _dialogue.player_actor
		_check(conversation != -1, "the conversation loaded (got %d)" % conversation)
		_check(actor != -1, "the player actor was added and kept (got %d)" % actor)
		_check(_dialogue.lines.get_child_count() >= 1, "the opening line rendered")
		if conversation != -1 and actor != -1:
			await _walk(conversation, actor)

	if _failures > 0:
		get_tree().quit(1)


## Take a few choices in a row. Every step compares the buttons on screen against
## what the engine is offering, which is where the translation from a button's row
## to an engine choice index gets checked: the engine only offers some of the
## choices on a hub and its indices skip the rest.
func _walk(conversation: int, actor: int) -> void:
	var lines: VBoxContainer = _dialogue.lines
	var choices: VBoxContainer = _dialogue.choices
	var walked := 0

	for step in STEPS:
		var position: int = LouderContextGlobal.get_actor_position(conversation, actor)
		var indices: Array = LouderContextGlobal.get_available_choice_indices(conversation, actor)
		print("--- step %d at hub %d, engine offers %s" % [step, position, indices])

		if indices.is_empty():
			print("    the conversation ran out of choices here")
			break

		_check(indices.size() == choices.get_child_count(),
			"step %d: one button per offered choice (%d offered, %d on screen)" % [step, indices.size(), choices.get_child_count()])
		for row in mini(indices.size(), choices.get_child_count()):
			var wanted: String = LouderContextGlobal.get_choice_text(conversation, position, actor, indices[row])
			_check((choices.get_child(row) as Button).text == wanted,
				"step %d: row %d reads the same as engine choice %d" % [step, row, indices[row]])

		var before := lines.get_child_count()
		(choices.get_child(0) as Button).emit_signal("pressed")
		await get_tree().process_frame
		_check(lines.get_child_count() > before,
			"step %d: pressing a choice moved the conversation on (%d lines became %d)" % [step, before, lines.get_child_count()])
		walked += 1

	_check(walked > 0, "the loop ran for at least one choice")


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if !condition:
		_failures += 1


func _finish() -> void:
	if _failures > 0:
		get_tree().quit(1)