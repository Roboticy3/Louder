extends Node

## Starting a conversation creates an actor for every speaker the config tags
## as a player character, each speaking as its own speaker and all beginning on
## the conversation's opening hub, before the conversation is announced to the
## interface that asked for it. Starting the same conversation again adds
## nobody; it only announces the actor of the peer that walked up this time.
##
## Choices are asked for by speaker name, because that is what an interface
## knows. The server resolves the speaker to its actor, which is why the test
## never needs an actor id outside the server.

const DOCUMENT := "res://conversations/marrow.json"
const CONFIG := "res://configs/config.json"
const EXPECTED_SPEAKERS := ["Reka", "Geno", "Note", "Benjamin", "Freya"]

var _failures := 0
var _events: Array[Dictionary] = []


func _ready() -> void:
	var saved_context := LouderContextGlobal.context
	LouderContextGlobal.context = LouderContext.new()
	LouderContextGlobal.event_received.connect(_on_event)

	var config_result: int = LouderContextGlobal.load_conversation_config_file(CONFIG)
	_check(config_result == 0, "the speaker config loaded (got %d)" % config_result)
	_check(LouderContextGlobal.player_character_speakers == EXPECTED_SPEAKERS,
		"the config's player_character speakers are read in file order (got %s)" % [
			LouderContextGlobal.player_character_speakers])

	var source := NodePath("/root/player_character_actors_test")
	LouderContextGlobal.start_conversation(DOCUMENT, source, "Benjamin")

	var conversation: int = LouderContextGlobal.context.make_name(DOCUMENT)
	var actor_ids: Array = LouderContextGlobal.context.get_actor_ids(conversation)
	_check(actor_ids.size() == EXPECTED_SPEAKERS.size(),
		"one actor exists per player character (%d actors, %d speakers)" % [
			actor_ids.size(), EXPECTED_SPEAKERS.size()])

	var start: int = LouderContextGlobal.context.next_hub_from_tags(
		conversation, [LouderContextGlobal.context.make_name("start")], false, -1)
	_check(start != -1, "the conversation has an opening hub (got %d)" % start)

	var actor_by_speaker := {}
	var first_start_events := _events.size()
	for index in EXPECTED_SPEAKERS.size():
		var speaker: String = EXPECTED_SPEAKERS[index]
		var actor: int = LouderContextGlobal.get_actor_for_speaker(conversation, speaker)
		_check(actor != -1, "%s has an actor" % speaker)
		if actor == -1:
			continue
		actor_by_speaker[speaker] = actor
		_check(actor == index, "%s's actor is numbered in config order (got %d, wanted %d)" % [
			speaker, actor, index])

		var data: Actor = LouderContextGlobal.context.get_actor(conversation, actor)
		var position: int = data.get_position() if data != null else -1
		_check(data != null and LouderContextGlobal.context.get_hub(conversation, position) != null,
			"%s's actor was placed in the conversation (hub %d)" % [speaker, position])
		var speak_as: Array = data.speak_as() if data != null else []
		_check(speak_as.size() == 1 and LouderContextGlobal.resolve_name(int(speak_as[0])) == speaker,
			"%s's actor speaks only as %s (got %s)" % [
				speaker, speaker, _names(speak_as)])

		# Everyone who moved was standing on the opening hub first. The actor of
		# the peer that asked drags the rest along, so this is where the whole
		# cast can be seen to have started together.
		_check(_moved_from(conversation, actor, start, 0, first_start_events),
			"%s's actor began on the opening hub (hub %d)" % [speaker, start])

	# Asking again must not add a second cast. The peer that asks this time does
	# get its own opening move, so that its interface has a line to show.
	var geno_before: int = LouderContextGlobal.get_actor_position(conversation, actor_by_speaker.get("Geno", -1))
	var second_start_events := _events.size()
	LouderContextGlobal.start_conversation(DOCUMENT, NodePath("/root/second"), "Geno")
	var after: Array = LouderContextGlobal.context.get_actor_ids(conversation)
	_check(after == actor_ids, "restarting the conversation adds no actors (got %s, wanted %s)" % [
		after, actor_ids])
	for speaker in EXPECTED_SPEAKERS:
		_check(LouderContextGlobal.get_actor_for_speaker(conversation, speaker) == actor_by_speaker[speaker],
			"restarting the conversation leaves %s's actor where it was" % speaker)
	_check(_moved_from(conversation, actor_by_speaker.get("Geno", -1), geno_before,
		second_start_events, _events.size()),
		"the second requesting speaker's opening move was announced")

	# A choice is named by speaker, and the actor it belongs to is found by the
	# server rather than by the caller.
	_check(LouderContextGlobal.get_actor_for_speaker(conversation, "Marrow") == -1,
		"a speaker with no actor does not resolve to one")
	var benjamin_before: int = LouderContextGlobal.get_actor_position(
		conversation, actor_by_speaker["Benjamin"])
	_check(LouderContextGlobal.select_choice(conversation, "Benjamin", 0),
		"Benjamin's choice is accepted by speaker name")
	_check(LouderContextGlobal.get_actor_position(conversation, actor_by_speaker["Benjamin"]) != benjamin_before,
		"Benjamin's actor moved after taking the choice")

	LouderContextGlobal.context = saved_context
	_finish()


## Whether an actor moved away from `from` between two points in the event log.
## The second start is checked from its own point so that the first start's
## moves cannot answer for it.
func _moved_from(conversation: int, actor: int, from: int, begin: int, end: int) -> bool:
	for index in range(begin, mini(end, _events.size())):
		var event: Dictionary = _events[index]
		if int(event.get("conversation", -1)) != conversation or int(event.get("actor", -1)) != actor:
			continue
		var motion: Dictionary = event.get("motion", {})
		if int(motion.get("source_position", -1)) == from:
			return true
	return false


func _names(names: Array) -> Array:
	var resolved: Array = []
	for name in names:
		resolved.append(LouderContextGlobal.resolve_name(int(name)))
	return resolved


func _on_event(event: Dictionary) -> void:
	_events.append(event)


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if not condition:
		_failures += 1


func _finish() -> void:
	if _failures > 0:
		get_tree().quit(1)
