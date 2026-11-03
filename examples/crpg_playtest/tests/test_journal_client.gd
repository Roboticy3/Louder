extends Node

const SCENE := "res://ui/conversation_log.tscn"

var _failures := 0

func _ready() -> void:
	var saved_context := LouderContextGlobal.context
	LouderContextGlobal.context = LouderContext.new()

	var log := (load(SCENE) as PackedScene).instantiate()
	log.player_actor = 3
	add_child(log)
	await get_tree().process_frame

	# Simulate a client receiving a motion event for its actor.
	LouderContextGlobal._motion({
		"conversation": 1,
		"actor": 3,
		"kind": 3,
		"motion": {
			"target_position": 5,
			"background": false,
			"speaker": -1,
			"speaker_text": "Benjamin",
			"text": "Hello",
		},
	})
	await get_tree().process_frame

	_check(log.entries.size() == 1, "client journal collected the event (got %d)" % log.entries.size())
	if log.entries.size() == 1:
		_check(str(log.entries[0].get("text", "")) == "Hello", "text preserved")

	# Event for another actor should be ignored.
	LouderContextGlobal._motion({
		"conversation": 1,
		"actor": 4,
		"kind": 3,
		"motion": {
			"target_position": 6,
			"background": false,
			"speaker": -1,
			"speaker_text": "Other",
			"text": "Other line",
		},
	})
	await get_tree().process_frame
	_check(log.entries.size() == 1, "other actor ignored (got %d)" % log.entries.size())

	# Repeated same position should be ignored.
	LouderContextGlobal._motion({
		"conversation": 1,
		"actor": 3,
		"kind": 3,
		"motion": {
			"target_position": 5,
			"background": false,
			"speaker": -1,
			"speaker_text": "Benjamin",
			"text": "Hello again",
		},
	})
	await get_tree().process_frame
	_check(log.entries.size() == 1, "repeated position ignored (got %d)" % log.entries.size())

	log.queue_free()
	LouderContextGlobal.context = saved_context
	if _failures > 0:
		get_tree().quit(1)
	else:
		print("PASS client journal works")

func _check(cond: bool, msg: String) -> void:
	print("%s %s" % ["PASS" if cond else "FAIL", msg])
	if not cond:
		_failures += 1