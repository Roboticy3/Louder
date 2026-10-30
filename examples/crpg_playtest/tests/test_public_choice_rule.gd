extends Node

## Marrow is public but is not in any speaker's listening lists. Her unplayed
## lines therefore use spoke, for one player or several, rather than skill.

const DOCUMENT := "res://conversations/marrow.json"
const CONFIG := "res://configs/config.json"

var _failures := 0


func _ready() -> void:
	var saved_context := LouderContextGlobal.context
	LouderContextGlobal.context = LouderContext.new()
	var context := LouderContextGlobal.context

	var config_result: int = LouderContextGlobal.load_conversation_config_file(CONFIG)
	_check(config_result == 0, "the real speaker config loads (code %d)" % config_result)
	var conversation: int = LouderContextGlobal.load_conversation_file(DOCUMENT)
	_check(conversation != -1, "the real Marrow conversation loads")

	# Case A: nobody plays Marrow, and Benjamin is the only actor. Public
	# visibility is not membership of Benjamin's listening lists.
	var benjamin := _add_actor(context, conversation, "Benjamin")
	_check(LouderContextGlobal.get_choice_rule(conversation, 0, benjamin, 0) == "spoke",
		"one player advancing Marrow's public line uses spoke")

	# The main hub's anonymous choices inherit Benjamin's voice, so Standard
	# takes priority over Skill and Spoke. Its explicit Geno choice remains Spoke.
	var main_benjamin := _add_actor(context, conversation, "Benjamin", 6)
	var main_rules: Array[String] = []
	for choice_index in 4:
		main_rules.append(LouderContextGlobal.get_choice_rule(conversation, 6, main_benjamin, choice_index))
	_check(main_rules == ["spoke", "standard", "standard", "standard"],
		"the main hub labels anonymous Benjamin choices standard (got %s)" % str(main_rules))

	# Case B: adding other players does not make Marrow a played voice or add her
	# to anyone's listening lists, so every player's classification stays spoke.
	var reka := _add_actor(context, conversation, "Reka")
	var geno := _add_actor(context, conversation, "Geno")
	for pair in [[benjamin, "Benjamin"], [reka, "Reka"], [geno, "Geno"]]:
		_check(LouderContextGlobal.get_choice_rule(conversation, 0, pair[0], 0) == "spoke",
			"%s still advances Marrow's public line through spoke with several players" % pair[1])

	# Control: Future really is in Benjamin's configured listening list. A hub
	# spoken by Future must retain the skill classification.
	var skill_conversation: int = context.make_name("public_choice_skill_control")
	var document := JSON.stringify([
		{"type": "hub", "conversation_id": 0, "tags": "start", "text": "start"},
		{"type": "hub", "conversation_id": 1, "speaker": "Future", "text": "A thought."},
		{"type": "connection", "from_id": 0, "from_port": 0, "to_id": 1},
	])
	_check(context.load_conversation(skill_conversation, document) == 0,
		"the configured-listener control conversation loads")
	var skill_actor := _add_actor(context, skill_conversation, "Benjamin")
	_check(LouderContextGlobal.get_choice_rule(skill_conversation, 0, skill_actor, 0) == "skill",
		"a speaker in Benjamin's listening list still uses skill")

	LouderContextGlobal.context = saved_context
	_finish()


func _add_actor(context: LouderContext, conversation: int, speaker: String, position: int = 0) -> int:
	var speak_as: Array[int] = [context.make_name(speaker)]
	return context.add_actor(conversation, position, 0, speak_as, [])


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if not condition:
		_failures += 1


func _finish() -> void:
	if _failures > 0:
		get_tree().quit(1)
