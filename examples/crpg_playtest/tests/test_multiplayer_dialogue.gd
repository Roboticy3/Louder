extends Node

## Tests the multiplayer layer (LouderContextGlobal) without any UI.
##
## The engine is single-peer here (the offline peer is the authority), so this
## exercises the authority side of the dialogue logic: loading a conversation,
## connecting characters as actors that speak as one speaker, asking the engine
## what each peer may choose, and taking a choice. Nothing in this script touches
## a scene or a control.

const CONFIG := "res://configs/config.json"

var _failures := 0
var _motions: Array = []


func _ready() -> void:
	_check(has_node("/root/LouderContextGlobal"), "LouderContextGlobal autoload is present")

	var config_result: int = LouderContextGlobal.load_conversation_config_file(CONFIG)
	_check(config_result == 0, "speaker config loads (got %d)" % config_result)

	var conversation: int = _load_conversation()
	_check(conversation != -1, "conversation loads (got %d)" % conversation)
	if conversation == -1:
		_finish()
		return

	# Capture motions the multiplayer layer emits, in arrival order.
	LouderContextGlobal.motion.connect(func(c: int, a: int, t: int, bg: int, s: String, k: int) -> void:
		_motions.append({"conversation": c, "actor": a, "target": t, "background": bg, "speaker": s, "kind": k})
	)

	var benjamin := CharacterData.new()
	benjamin.name = "Benjamin"
	var geno := CharacterData.new()
	geno.name = "Geno"
	var reka := CharacterData.new()
	reka.name = "Reka"
	var debug_character := CharacterData.make_debug()

	# Each peer speaks as exactly one speaker, named by their CharacterData.
	_check(benjamin.get_speaker_names().size() == 1 and str(benjamin.get_speaker_names()[0]) == "Benjamin", "Benjamin speaks as Benjamin")
	_check(geno.get_speaker_names().size() == 1 and str(geno.get_speaker_names()[0]) == "Geno", "Geno speaks as Geno")
	_check(reka.get_speaker_names().size() == 1 and str(reka.get_speaker_names()[0]) == "Reka", "Reka speaks as Reka")
	_check(debug_character.get_speaker_names().is_empty(), "debug speaks as nobody")
	_check(debug_character.get_actor_flags() == CharacterData.ACTOR_FLAG_DEBUG, "debug is a debug actor")

	var actor_benjamin: int = LouderContextGlobal.connect_actor(conversation, ["start"], benjamin.get_actor_flags(), benjamin.get_speaker_names())
	var actor_geno: int = LouderContextGlobal.connect_actor(conversation, ["start"], geno.get_actor_flags(), geno.get_speaker_names())
	var actor_reka: int = LouderContextGlobal.connect_actor(conversation, ["start"], reka.get_actor_flags(), reka.get_speaker_names())
	var actor_debug: int = LouderContextGlobal.connect_actor(conversation, ["start"], debug_character.get_actor_flags(), debug_character.get_speaker_names())

	_check(actor_benjamin == 0, "Benjamin gets actor 0 (got %d)" % actor_benjamin)
	_check(actor_geno == 1, "Geno gets actor 1 (got %d)" % actor_geno)
	_check(actor_reka == 2, "Reka gets actor 2 (got %d)" % actor_reka)
	_check(actor_debug == 3, "debug gets actor 3 (got %d)" % actor_debug)

	# Each actor speaks as the voice its character named.
	for pair in [[actor_benjamin, "Benjamin"], [actor_geno, "Geno"], [actor_reka, "Reka"]]:
		var actor: int = pair[0]
		var entry = LouderContextGlobal.context.get_actor(conversation, actor)
		_check(entry != null, "actor %d exists" % actor)
		if entry != null:
			var speak_as: Array = entry.speak_as()
			_check(not speak_as.is_empty() and LouderContextGlobal.resolve_name(speak_as[0]) == pair[1],
				"actor %d speaks as %s (got %s)" % [actor, pair[1], LouderContextGlobal.resolve_name(speak_as[0]) if not speak_as.is_empty() else ""])

	# A peer sees only the branch spoken in its own voice.
	_check(LouderContextGlobal.get_available_choice_indices(conversation, actor_benjamin) == [0],
		"Benjamin sees only his branch (got %s)" % str(LouderContextGlobal.get_available_choice_indices(conversation, actor_benjamin)))
	_check(LouderContextGlobal.get_available_choice_indices(conversation, actor_geno) == [1],
		"Geno sees only his branch (got %s)" % str(LouderContextGlobal.get_available_choice_indices(conversation, actor_geno)))
	_check(LouderContextGlobal.get_available_choice_indices(conversation, actor_reka) == [2],
		"Reka sees only her branch (got %s)" % str(LouderContextGlobal.get_available_choice_indices(conversation, actor_reka)))
	_check(LouderContextGlobal.get_available_choice_indices(conversation, actor_debug).size() == 3,
		"debug sees every branch (got %d)" % LouderContextGlobal.get_available_choice_indices(conversation, actor_debug).size())

	# The choice text and speaker come back as the hub's own.
	_check(LouderContextGlobal.get_choice_text(conversation, 0, actor_benjamin, 0) == "\"Hello, Benjamin.\"",
		"Benjamin's choice reads the right line")
	_check(LouderContextGlobal.get_line_speaker(conversation, 1, actor_benjamin) == "Benjamin",
		"hub 1 is spoken by Benjamin")

	# Taking a choice moves the actor and emits a motion through the multiplayer layer.
	var motions_before := _motions.size()
	var took: bool = LouderContextGlobal.select_choice(conversation, actor_benjamin, 0)
	_check(took, "Benjamin's choice is accepted")

	# The peer that moved is now on hub 1.
	_check(LouderContextGlobal.get_actor_position(conversation, actor_benjamin) == 1,
		"Benjamin is on hub 1 (got %d)" % LouderContextGlobal.get_actor_position(conversation, actor_benjamin))

	# The multiplayer layer emitted at least one new motion for this choice.
	_check(_motions.size() > motions_before, "a motion was emitted for the choice")
	if _motions.size() > motions_before:
		var first_new: Dictionary = _motions[motions_before]
		_check(first_new["actor"] == actor_benjamin, "the first new motion is for Benjamin (got %d)" % first_new["actor"])
		_check(first_new["target"] == 1, "the first new motion targets hub 1 (got %d)" % first_new["target"])
		_check(first_new["speaker"] == "Benjamin", "the first new motion is spoken by Benjamin (got %s)" % first_new["speaker"])
		_check(first_new["kind"] == 1, "the first new motion is a chosen move (got %d)" % first_new["kind"])

	# Benjamin has nowhere left to go.
	_check(LouderContextGlobal.get_available_choice_indices(conversation, actor_benjamin).is_empty(),
		"Benjamin has no choices on hub 1")

	_finish()


func _load_conversation() -> int:
	var document := JSON.stringify([
		{"type": "hub", "conversation_id": 0, "tags": "start", "text": "start"},
		{"type": "hub", "conversation_id": 1, "speaker": "Benjamin", "text": "\"Hello, Benjamin.\""},
		{"type": "hub", "conversation_id": 2, "speaker": "Geno", "text": "\"Hello, Geno.\""},
		{"type": "hub", "conversation_id": 3, "speaker": "Reka", "text": "\"Hello, Reka.\""},
		{"type": "connection", "from_id": 0, "from_port": 0, "to_id": 1},
		{"type": "connection", "from_id": 0, "from_port": 1, "to_id": 2},
		{"type": "connection", "from_id": 0, "from_port": 2, "to_id": 3},
	])
	var conversation: int = LouderContextGlobal.context.make_name("multiplayer_dialogue")
	var result: int = LouderContextGlobal.context.load_conversation(conversation, document)
	if result != 0 and result != LouderContextGlobal.LOAD_ALREADY_LOADED:
		push_error("conversation failed to load: %d" % result)
		return -1
	return conversation


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if !condition:
		_failures += 1


func _finish() -> void:
	if _failures > 0:
		get_tree().quit(1)