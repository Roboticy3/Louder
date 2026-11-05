extends Node

## Checks that assignments raise interrupts, including one whose right-hand side
## is a plain value and one that goes through an identifier.

var _failures := 0
var _interrupts := []

func _ready() -> void:
	var ctx := LouderContext.new()
	var conversation := ctx.make_name(&"assignment_test")

	var loaded := ctx.load_conversation(conversation, JSON.stringify([
		{"type": "hub", "conversation_id": "0", "tags": "start", "text": "start"},
		{"type": "hub", "conversation_id": "1", "choices": "0", "expressions": "alpha = 1\nbeta = gamma", "text": "hub1"},
		{"type": "connection", "from_id": "0", "from_port": 0, "to_id": "1"},
	]))
	_check(loaded == 0, "the test conversation loaded (code %d)" % loaded)

	var actor := ctx.add_actor(conversation, 0, 0, [], [])
	_check(actor != -1, "an actor was added")

	ctx.force_position(conversation, actor, 0, -1, 0)
	LouderContextGlobal.context = ctx
	LouderContextGlobal.interrupt_raised.connect(_on_interrupt)
	LouderContextGlobal._drain()

	var ok := ctx.select_choice(conversation, actor, 0)
	_check(ok, "the actor took the choice into hub 1")
	LouderContextGlobal._drain()

	_check(_interrupts.size() >= 2, "at least two interrupts were raised (got %d)" % _interrupts.size())
	var texts := []
	for entry in _interrupts:
		texts.append(str(entry.identifier_text))
	_check("alpha = 1" in texts, "the simple assignment raised \"alpha = 1\"")
	_check("gamma" in texts, "the identifier on the right-hand side raised its own interrupt")

	LouderContextGlobal.context = LouderContext.new()
	if _failures > 0:
		get_tree().quit(1)


func _on_interrupt(rule: String, identifier: String, conversation: int, actor: int, hub: int) -> void:
	_interrupts.append({"identifier_text": identifier})


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if !condition:
		_failures += 1
