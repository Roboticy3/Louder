extends Node

## The dialogue session layer joins a peer to a conversation whose actors were
## created when it started. Joining must find the actor that already speaks for
## the peer rather than adding a second one, and a choice picked in this layer
## travels by speaker name like the interface's choices do.

const DOCUMENT := "res://conversations/marrow.json"
const CONFIG := "res://configs/config.json"

var _failures := 0


func _ready() -> void:
	var saved_context := LouderContextGlobal.context
	LouderContextGlobal.context = LouderContext.new()

	var config_result: int = LouderContextGlobal.load_conversation_config_file(CONFIG)
	_check(config_result == 0, "the speaker config loaded (got %d)" % config_result)

	var benjamin: CharacterData = null
	for character in Network.get_characters():
		if character.name == "Benjamin":
			benjamin = character
	_check(benjamin != null, "Benjamin is in the character catalog")
	if benjamin == null:
		LouderContextGlobal.context = saved_context
		_finish()
		return

	LouderContextGlobal.start_conversation(DOCUMENT, NodePath("/root/dialogue_session_test"), "Benjamin")
	var conversation: int = LouderContextGlobal.context.make_name(DOCUMENT)
	var auto_actor: int = LouderContextGlobal.get_actor_for_speaker(conversation, "Benjamin")
	var actors_before: Array = LouderContextGlobal.context.get_actor_ids(conversation)

	var session := DialogueSession.new()
	var joined: int = session.join(conversation, 1, benjamin)
	_check(joined == auto_actor, "joining finds the actor the conversation already made (got %d, wanted %d)" % [
		joined, auto_actor])
	_check(LouderContextGlobal.context.get_actor_ids(conversation) == actors_before,
		"joining adds no second actor for the same speaker")
	_check(session.get_speaker(1) == "Benjamin", "the session knows the peer's speaker")

	var indices := session.get_choice_indices(1)
	_check(indices.size() > 0, "the peer is offered a choice at the opening hub")
	if indices.size() > 0:
		var before: int = session.get_position(1)
		_check(session.select_choice(1, 0), "the session's choice is taken by speaker name")
		_check(session.get_position(1) != before, "the peer moved after choosing")

	LouderContextGlobal.context = saved_context
	_finish()


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if not condition:
		_failures += 1


func _finish() -> void:
	if _failures > 0:
		get_tree().quit(1)
