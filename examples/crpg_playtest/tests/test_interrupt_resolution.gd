extends Node

## Checks that an identifier in an expression interrupts evaluation, that the
## game code resolves it with the placeholder "0", and that listeners see the
## rule, identifier, conversation, actor, and hub it came from.

var _failures := 0
var _interrupts := []

func _ready() -> void:
	var ctx := LouderContext.new()
	var conversation := ctx.make_name(&"interrupt_test")

	var loaded := ctx.load_conversation(conversation, JSON.stringify([
		{"type": "hub", "conversation_id": "0", "tags": "start", "text": "start"},
		{"type": "hub", "conversation_id": "1", "choices": "0", "expressions": "value = foo", "text": "hub1"},
		{"type": "connection", "from_id": "0", "from_port": 0, "to_id": "1"},
	]))
	_check(loaded == 0, "the test conversation loaded (code %d)" % loaded)

	var actor := ctx.add_actor(conversation, 0, 0, [], [])
	_check(actor != -1, "an actor was added")

	ctx.force_position(conversation, actor, 0, -1, 0)

	# Listen the way game code does: the interrupt carries every detail about
	# where it came from.
	LouderContextGlobal.context = ctx
	LouderContextGlobal.interrupt_raised.connect(_on_interrupt)
	LouderContextGlobal._drain()

	var ok := ctx.select_choice(conversation, actor, 0)
	_check(ok, "the actor took the choice into hub 1")
	LouderContextGlobal._drain()

	_check(_interrupts.size() == 1, "one interrupt was raised (got %d)" % _interrupts.size())
	if _interrupts.size() == 1:
		var details: Dictionary = _interrupts[0]
		_check(str(details.rule) == "ask", "the interrupt's rule is \"ask\" (got %s)" % str(details.rule))
		_check(str(details.identifier) == "foo", "the interrupt resolves identifier \"foo\" (got %s)" % str(details.identifier))
		_check(int(details.conversation) == conversation, "the interrupt names its conversation")
		_check(int(details.actor) == actor, "the interrupt names its actor")
		_check(int(details.hub) == 1, "the interrupt names its hub (got %d)" % int(details.hub))

	# The placeholder resolution answered the ask, so it no longer accepts an answer.
	var ask := ctx.get_ask(0)
	_check(ask != null, "the ask is still cached")
	if ask != null:
		_check(not ctx.answer_ask(0, 0), "the ask was already resolved with the placeholder")

	# Restore the shared context the other tests expect.
	LouderContextGlobal.context = LouderContext.new()

	if _failures > 0:
		get_tree().quit(1)


func _on_interrupt(rule: String, identifier: String, conversation: int, actor: int, hub: int) -> void:
	_interrupts.append({
		"rule": rule,
		"identifier": identifier,
		"conversation": conversation,
		"actor": actor,
		"hub": hub,
	})


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if !condition:
		_failures += 1
