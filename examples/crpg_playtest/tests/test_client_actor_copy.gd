extends Node

## The server owns the engine; a client keeps a copy of each actor the server
## adds, relayed to it as data, so that its own context can answer the UI's
## questions (which choices may this actor take where it stands) and its
## dialogue is told the actor exists. The actor only ever existed in the
## server's context, so _actor_added takes the actor's data across the wire
## rather than asking the client to look it up.
##
## Offline, server and client share one autoload and one context, and the
## loopback masks the relay: a dialogue hears the server's own local emission
## of actor_added, which is what lets it appear here at all. The client's half
## is therefore also driven by hand: a second LouderContext that knows exactly
## what a real client knows - the config and the document, and no actors - and
## the real _actor_added handler is called against it with the data the server
## relays, exactly as the server's rpc would.

const DOCUMENT := "res://conversations/marrow.json"
const CONFIG := "res://configs/config.json"
const SCENE := "res://ui/scrolling_dialogue.tscn"

var _failures := 0


func _ready() -> void:
	# Isolate from any context state left by earlier tests in the same process:
	# both the server's context and the client's must start empty so the
	# engine assigns the same actor id on both, which is what a real client
	# relies on.
	var saved_context := LouderContextGlobal.context
	LouderContextGlobal.context = LouderContext.new()

	var characters := Network.get_characters()
	_check(characters.size() >= 1, "the catalog holds at least one character")
	if characters.is_empty():
		LouderContextGlobal.context = saved_context
		_finish()
		return
	var client_character := characters[0]
	Network._apply_allocations({1: client_character.get_id()})

	var config_result: int = LouderContextGlobal.load_conversation_config_file(CONFIG)
	_check(config_result == 0, "the speaker config loaded (got %d)" % config_result)

	# The client's dialogue, wired the way its controller left it.
	var scene := load(SCENE) as PackedScene
	_check(scene != null, "the dialogue scene loads")
	if scene == null:
		_finish()
		return
	var dialogue := scene.instantiate() as ScrollingDialogue
	dialogue.player_speaker = client_character.name
	add_child(dialogue)
	await get_tree().process_frame

	# The request the client's controller sends: the gate passes and the
	# server adds the actor, which loops back into this same context and lets
	# the dialogue claim it.
	LouderContextGlobal.start_conversation_rpc(DOCUMENT, dialogue.get_path(), client_character.name)
	await _settle()

	var actor: int = dialogue.player_actor
	_check(actor != -1 and dialogue.visible,
		"the client's dialogue appears (actor %d, visible %s)" % [dialogue.player_actor, dialogue.visible])
	_check(dialogue.lines.get_child_count() >= 1,
		"the client's dialogue shows the opening line (got %d)" % dialogue.lines.get_child_count())

	var conversation: int = dialogue.conversation

	# What a real client cannot hear that way is driven by hand: a context
	# that knows exactly what a real client knows - the config and the
	# document, and no actors - and the real relay handlers called against
	# it with the data the server relays.
	var original: Actor = LouderContextGlobal.context.get_actor(conversation, actor)
	_check(original != null, "the server's context holds the actor to relay")
	if original == null:
		LouderContextGlobal.context = saved_context
		dialogue.queue_free()
		_finish()
		return

	var client_context := LouderContext.new()
	var config_text := FileAccess.open(CONFIG, FileAccess.READ).get_as_text()
	_check(client_context.load_conversation_config(config_text) == 0,
		"the client's context loads the config")
	var document_text := FileAccess.open(DOCUMENT, FileAccess.READ).get_as_text()
	_check(client_context.load_conversation(conversation, document_text) == 0,
		"the client's context loads the document")

	var server_context := LouderContextGlobal.context
	LouderContextGlobal.context = client_context
	LouderContextGlobal._actor_added(conversation, actor,
		original.get_position(), original.get_flags(), original.speak_as(), original.listen_to())
	LouderContextGlobal.context = server_context

	_check(client_context.get_actor(conversation, actor) != null,
		"the client's context holds a copy of the actor")

	var position: int = original.get_position()
	LouderContextGlobal.context = client_context
	LouderContextGlobal._motion(conversation, actor, position, 0, client_character.name, 3)
	LouderContextGlobal.context = server_context

	var copy: Actor = client_context.get_actor(conversation, actor)
	_check(copy != null and copy.get_position() == position,
		"the client's copy followed the motion")

	LouderContextGlobal.context = saved_context
	dialogue.queue_free()
	_finish()


func _settle() -> void:
	for frame in 6:
		await get_tree().process_frame


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if !condition:
		_failures += 1


func _finish() -> void:
	if _failures > 0:
		get_tree().quit(1)