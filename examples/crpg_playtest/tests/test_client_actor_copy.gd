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
## of actor_added. The client's half is therefore driven by hand: a second
## LouderContext that knows exactly what a real client knows - the config and
## the document, and no actors - and the real _actor_added handler is called
## against it with the data the server relays, exactly as the server's rpc
## would.

const DOCUMENT := "res://conversations/marrow.json"
const CONFIG := "res://configs/config.json"
const SCENE := "res://ui/scrolling_dialogue.tscn"

var _failures := 0
var _announced: Array[int] = []


func _ready() -> void:
	var config_result: int = LouderContextGlobal.load_conversation_config_file(CONFIG)
	_check(config_result == 0, "the speaker config loaded (got %d)" % config_result)

	# -- the server's seat ---------------------------------------------------
	# The client started a conversation and asked for an actor; this is what
	# the server does when the client's connect_actor.rpc_id(1, ...) arrives.
	var conversation: int = LouderContextGlobal.load_conversation_file(DOCUMENT)
	_check(conversation != -1, "the conversation document loads")
	if conversation == -1:
		_finish()
		return

	var actor: int = LouderContextGlobal.connect_actor(conversation, ["start"], 0, ["Benjamin"], [])
	_check(actor != -1, "the server added the client's actor (got %d)" % actor)

	# -- the client's seat ---------------------------------------------------
	# A real client's context knows the config and the document (loaded when
	# _started arrived) and nothing else: actors live in the server's context.
	var client_context := LouderContext.new()
	var config_text := FileAccess.open(CONFIG, FileAccess.READ).get_as_text()
	_check(client_context.load_conversation_config(config_text) == 0,
		"the client's context loads the config")
	var document_text := FileAccess.open(DOCUMENT, FileAccess.READ).get_as_text()
	_check(client_context.load_conversation(conversation, document_text) == 0,
		"the client's context loads the document")

	# The client's dialogue, wired the way its controller left it. It is made
	# only now, after the server-side add, so the server's local emission of
	# actor_added cannot reach it: everything it gets arrives the way a real
	# client's would, through the rpc handler.
	var scene := load(SCENE) as PackedScene
	_check(scene != null, "the dialogue scene loads")
	if scene == null:
		_finish()
		return
	var dialogue := scene.instantiate() as ScrollingDialogue
	dialogue.player_speaker = &"Benjamin"
	add_child(dialogue)
	await get_tree().process_frame

	LouderContextGlobal.actor_added.connect(_on_actor_added)

	# What the server relays about the actor it added: the actor's data, not
	# just its id, because the client's context never had the original.
	var original: Actor = LouderContextGlobal.context.get_actor(conversation, actor)
	_check(original != null, "the server's context holds the actor to relay")
	if original == null:
		_finish()
		return

	# What the server's _actor_added.rpc(...) runs on the client: the real
	# handler, against the client's own context.
	var server_context := LouderContextGlobal.context
	LouderContextGlobal.context = client_context
	LouderContextGlobal._actor_added(conversation, actor,
		original.get_position(), original.get_flags(), original.speak_as(), original.listen_to())
	LouderContextGlobal.context = server_context

	_check(actor in _announced, "the client is told its actor was added")
	_check(client_context.get_actor(conversation, actor) != null,
		"the client's context holds a copy of the actor")
	_check(dialogue.player_actor != -1 and dialogue.visible,
		"the client's dialogue appears (actor %d, visible %s)" % [dialogue.player_actor, dialogue.visible])

	dialogue.queue_free()
	_finish()


func _on_actor_added(_conversation: int, actor: int) -> void:
	_announced.append(actor)


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if !condition:
		_failures += 1


func _finish() -> void:
	if _failures > 0:
		get_tree().quit(1)
