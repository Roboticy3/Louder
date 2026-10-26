extends Node

## The dialogue's "is this conversation mine?" gate, from a client's seat.
##
## The lobby is rigged so that THIS process is the client: the local peer plays
## one character, the other peer (the host) plays another. A conversation
## opened under the host's character must not open the client's dialogue; one
## opened under the client's own character must, and the request itself must
## name the client's character - it used to name the host's, because both ends
## of the flow resolved "my character" through get_multiplayer_authority(),
## which is the server (peer 1) on every peer.
##
## Offline the local peer is also the server, so connect_actor loops back into
## the same context and the actor is heard through the server's local emission.
## What a real client can never receive that way - the actor copy its own
## context needs - is covered by test_client_actor_copy.gd.

const DOCUMENT := "res://conversations/marrow.json"
const CONFIG := "res://configs/config.json"
const SCENE := "res://ui/scrolling_dialogue.tscn"

var _failures := 0


func _ready() -> void:
	_check(has_node("/root/Network"), "Network autoload is present")
	_check(has_node("/root/LouderContextGlobal"), "LouderContextGlobal autoload is present")

	# Rig the lobby from the client's seat: the local peer plays the client
	# character, and the host on the other peer plays the other one.
	var characters := Network.get_characters()
	_check(characters.size() >= 2, "the catalog holds at least two characters")
	if characters.size() < 2:
		_finish()
		return
	var host_character := characters[0]
	var client_character := characters[1]
	Network._apply_allocations({1: client_character.get_id(), 2: host_character.get_id()})
	print("lobby: this peer plays %s (the client), peer 2 plays %s (the host)" % [
		client_character.name, host_character.name])

	var config_result: int = LouderContextGlobal.load_conversation_config_file(CONFIG)
	_check(config_result == 0, "the speaker config loaded (got %d)" % config_result)

	var scene := load(SCENE) as PackedScene
	_check(scene != null, "the dialogue scene loads")
	if scene == null:
		_finish()
		return

	# What the client's controller broadcasts when the client starts a
	# conversation: the character of the peer asking, not the host's.
	var broadcast_name: String = Network.get_character_of().name
	print("client starts a conversation; the request goes out under '%s'" % broadcast_name)
	_check(broadcast_name == client_character.name,
		"the client's request names the client's character (got %s)" % broadcast_name)

	# A conversation opened under the HOST's character is not the client's.
	var bystander_dialogue := await _make_dialogue(scene, client_character.name)
	LouderContextGlobal.start_conversation_rpc(DOCUMENT, bystander_dialogue.get_path(), host_character.name)
	await _settle()
	_check(bystander_dialogue.player_actor == -1 and not bystander_dialogue.visible,
		"the client's dialogue stays out of the host's conversation (actor %d, visible %s)" % [
			bystander_dialogue.player_actor, bystander_dialogue.visible])
	bystander_dialogue.queue_free()

	# A dialogue that never asked to join a conversation must not claim someone
	# else's actor. Before its controller ever starts a conversation,
	# player_speaker is still the scene default - and that default is the host's
	# character's name, so the speaker check alone would claim the host's actor
	# and open the client's UI for the host's conversation.
	var idle_dialogue := scene.instantiate() as ScrollingDialogue
	add_child(idle_dialogue)
	await get_tree().process_frame
	_check(idle_dialogue.player_speaker == StringName(host_character.name),
		"setup: the idle dialogue still carries the scene default speaker (got %s)" % idle_dialogue.player_speaker)

	# The server adding the host's actor, as the host's own request produces.
	# actor_added announces it to every peer, including this idle client.
	var conversation: int = LouderContextGlobal.load_conversation_file(DOCUMENT)
	var host_actor: int = LouderContextGlobal.connect_actor(conversation, ["start"], 0, [host_character.name], [])
	_check(host_actor != -1, "the server adds the host's actor (got %d)" % host_actor)
	await _settle()
	_check(idle_dialogue.player_actor == -1 and not idle_dialogue.visible,
		"a dialogue that never joined stays closed when another player's actor is added (actor %d, visible %s)" % [
			idle_dialogue.player_actor, idle_dialogue.visible])
	idle_dialogue.queue_free()

	# A conversation opened under the CLIENT's character is the client's own.
	var client_dialogue := await _make_dialogue(scene, client_character.name)
	LouderContextGlobal.start_conversation_rpc(DOCUMENT, client_dialogue.get_path(), client_character.name)
	await _settle()
	_check(client_dialogue.player_actor != -1 and client_dialogue.visible,
		"the client's dialogue opens for the client's own conversation (actor %d, visible %s)" % [
			client_dialogue.player_actor, client_dialogue.visible])
	client_dialogue.queue_free()

	_finish()


## Instantiate the dialogue the way character_avatar_controller_single wires
## its own: the player's speaker is the local player's character, set before
## the conversation starts.
func _make_dialogue(scene: PackedScene, speaker: String) -> ScrollingDialogue:
	var dialogue := scene.instantiate() as ScrollingDialogue
	dialogue.player_speaker = speaker
	add_child(dialogue)
	await get_tree().process_frame
	return dialogue


## Give the signal chain - conversation_started, the connect_actor RPC
## loopback, and the actor_added that follows it - a few frames to land.
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
