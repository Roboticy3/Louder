extends Node

## Drives the world's way into the dialogue headlessly: an interactable object
## asking for a conversation, the scrolling dialogue joining it, and the engine's
## opening hub reaching the UI as a line on screen.
##
## The flow is a chain of calls, so when no line appears the chain is walked a
## link at a time afterwards, printing the engine's queue after each one. The
## first call that leaves no motion behind is the one worth reporting.

const DOCUMENT := "res://conversations/marrow.json"
const CONFIG := "res://configs/config.json"

var _failures := 0
var _motions: Array = []


func _ready() -> void:
	var config_result: int = LouderContextGlobal.load_conversation_config_file(CONFIG)
	_check(config_result == 0, "the speaker config loaded (got %d)" % config_result)

	var benjamin: CharacterData = null
	for character in Network.get_characters():
		if character.name == "Benjamin":
			benjamin = character
	Network._apply_allocations({1: benjamin.get_id()})

	var scene := load("res://ui/scrolling_dialogue.tscn") as PackedScene
	_check(scene != null, "the dialogue scene loads")
	if scene == null:
		_finish()
		return

	var dialogue := scene.instantiate() as Control
	add_child(dialogue)
	await get_tree().process_frame

	LouderContextGlobal.motion.connect(_on_motion)

	# What character_avatar_controller_single does when an avatar reaches the
	# object it meant to talk to.
	LouderContextGlobal.start_conversation_rpc(DOCUMENT, dialogue.get_path(), "Benjamin")
	for frame in 6:
		await get_tree().process_frame

	print("after start: conversation=%d actor=%d lines=%d choices=%d motions=%s" % [
		dialogue.conversation, dialogue.player_actor,
		dialogue.lines.get_child_count(), dialogue.choices.get_child_count(), _motions])

	_check(dialogue.conversation != -1, "the dialogue joined the conversation")
	_check(dialogue.player_actor != -1, "the player's actor was added")
	_check(dialogue.lines.get_child_count() >= 1, "the opening line rendered")

	if dialogue.lines.get_child_count() < 1:
		_walk_the_chain(dialogue)

	_finish()


## Repeat what connect_actor does, one call at a time, printing the queue after
## each of them.
func _walk_the_chain(dialogue: Control) -> void:
	var context := LouderContextGlobal.context
	var conversation: int = context.make_name(DOCUMENT)

	var start: int = context.next_hub_from_tags(conversation, [context.make_name(&"start")], false, -1)
	print("chain: conversation=%d loaded=%s start=%d" % [conversation, context.has_conversation(conversation), start])
	if start == -1:
		return
	context.flush_events()

	var actor: int = LouderContextGlobal._add_actor(conversation, start, 0,
		[context.make_name(&"Benjamin")], [])
	print("chain: actor=%d queue after add: %s" % [actor, _queue()])
	context.flush_events()

	var position: int = context.get_actor_position(conversation, actor)
	print("chain: where the actor stands: %d" % position)

	var announced: bool = LouderContextGlobal.announce_position(conversation, actor)
	print("chain: announce_position=%s queue: %s" % [announced, _queue()])
	var forced: bool = context.force_position(conversation, actor, position, -1, 0)
	print("chain: force_position=%s queue: %s" % [forced, _queue()])
	print("chain: the dialogue's own actor is %d in conversation %d" % [
		dialogue.player_actor, dialogue.conversation])
	context.flush_events()


func _on_motion(conversation: int, actor: int, target: int, background: int, speaker: String, _kind: int) -> void:
	_motions.append([conversation, actor, target, background, speaker])


## The queue as kinds, since the names of the kinds are what says which call
## failed to produce one.
func _queue() -> String:
	var context := LouderContextGlobal.context
	var kinds := PackedStringArray()
	var index := 0
	while index < context.get_event_count():
		var event: LouderEvent = context.get_event(index)
		if event == null:
			break
		kinds.append("%d@actor%d" % [event.get_kind(), event.get_actor()])
		index += 1
	return "[%s]" % ", ".join(kinds)


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if !condition:
		_failures += 1


func _finish() -> void:
	if _failures > 0:
		get_tree().quit(1)
