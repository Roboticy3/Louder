extends Node

## Checks that a motion carries each choice's rule as well as its text, and that
## the scrolling dialogue conceals text for a choice opened by the spoke rule.

const SCENE := "res://ui/scrolling_dialogue.tscn"
const SECRET := "This line must not be previewed."

var _failures := 0
var _conversation := -1
var _events: Array[Dictionary] = []


func _ready() -> void:
	var saved_context := LouderContextGlobal.context
	LouderContextGlobal.context = LouderContext.new()

	var benjamin: CharacterData = null
	for character in Network.get_characters():
		if character.name == "Benjamin":
			benjamin = character
			break
	_check(benjamin != null, "Benjamin is in the character catalog")
	if benjamin == null:
		LouderContextGlobal.context = saved_context
		_finish()
		return
	Network._apply_allocations({1: benjamin.get_id()})

	_conversation = LouderContextGlobal.context.make_name("spoke_choice_test")
	var loaded: int = LouderContextGlobal.context.load_conversation(_conversation, _document())
	_check(loaded == 0, "the two-actor conversation loads (code %d)" % loaded)

	var dialogue := (load(SCENE) as PackedScene).instantiate() as ScrollingDialogue
	add_child(dialogue)
	await get_tree().process_frame
	LouderContextGlobal.event_received.connect(_on_event)

	# Geno holds the voice first. Benjamin's otherwise identical choice is then
	# available only through the spoke rule, rather than through speaking as Geno.
	var geno_actor: int = LouderContextGlobal.connect_actor(_conversation, ["start"], 0, ["Geno"], [])
	var benjamin_actor: int = LouderContextGlobal.connect_actor(_conversation, ["start"], 0, ["Benjamin"], [])
	_check(dialogue.player_actor == benjamin_actor, "the dialogue claims Benjamin's actor")
	_check(LouderContextGlobal.get_choice_rule(_conversation, 0, benjamin_actor, 0) == "spoke",
		"the adapter reports Benjamin's choice as spoke")

	var geno_choices := _event_choices(geno_actor)
	_check(geno_choices.size() == 1 and str(geno_choices[0].get("rule", "")) == "standard",
		"the motion also carries an ordinary choice's standard rule")

	var benjamin_choices := _event_choices(benjamin_actor)
	_check(benjamin_choices.size() == 1, "Benjamin's motion carries one choice")
	if benjamin_choices.size() == 1:
		_check(str(benjamin_choices[0].get("rule", "")) == "spoke",
			"Benjamin's motion carries the spoke rule")
		_check(str(benjamin_choices[0].get("text", "")) == SECRET,
			"the choice data still pairs the rule with the engine's text")

	_check(dialogue.choices.get_child_count() == 1, "the dialogue renders one choice")
	if dialogue.choices.get_child_count() == 1:
		_check((dialogue.choices.get_child(0) as Button).text == "CONTINUE.",
			"the spoke choice renders CONTINUE. instead of its target's text")

	# Non-spoke choices retain their own text in the same renderer.
	dialogue._on_choices_revealed([{"rule": "standard", "text": "A visible choice."}])
	_check((dialogue.choices.get_child(0) as Button).text == "A visible choice.",
		"an ordinary choice still renders its own text")

	LouderContextGlobal.event_received.disconnect(_on_event)
	dialogue.queue_free()
	LouderContextGlobal.context = saved_context
	_finish()


func _document() -> String:
	return JSON.stringify([
		{"type": "hub", "conversation_id": 0, "tags": "start", "text": "A quiet room."},
		{"type": "hub", "conversation_id": 1, "speaker": "Geno", "text": SECRET},
		{"type": "connection", "from_id": 0, "from_port": 0, "to_id": 1},
	])


func _on_event(event: Dictionary) -> void:
	if int(event.get("conversation", -1)) == _conversation:
		_events.append(event)


func _event_choices(actor: int) -> Array:
	for index in range(_events.size() - 1, -1, -1):
		var event := _events[index]
		if int(event.get("actor", -1)) != actor:
			continue
		var motion: Dictionary = event.get("motion", {})
		return motion.get("choices", [])
	return []


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if not condition:
		_failures += 1


func _finish() -> void:
	if _failures > 0:
		get_tree().quit(1)
