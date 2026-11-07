extends Node

## Smoke test for the readable conversation format.
##
## Loads a tiny conversation written in the line-oriented format and checks that
## its hubs, tags and default choice survive the trip into the engine.

var _failures := 0

func _ready() -> void:
	var ctx := LouderContext.new()
	var conv := ctx.make_name(&"readable_test")

	var document := "A\nHello\ntags: start\n\nB\nWorld\ntags: main\n\nfirst\n> start\n"
	var loaded := ctx.load_conversation_text(conv, document)
	_check(loaded == 0, "the readable conversation loaded (code %d)" % loaded)

	var hello := ctx.make_name(&"Hello")
	var world := ctx.make_name(&"World")
	_check(ctx.get_hub_text(conv, 0) == hello, "hub 0 keeps its text")
	_check(ctx.get_hub_text(conv, 1) == world, "hub 1 keeps its text")
	_check(ctx.get_hub_speaker(conv, 0) == ctx.make_name(&"A"), "hub 0 keeps its speaker")

	var start_tag := ctx.make_name(&"start")
	_check(ctx.next_hub_from_tags(conv, [start_tag], false, -1) == 0, "tags are lowercased")

	var a_name := ctx.make_name(&"A")
	var actor := ctx.add_actor(conv, 2, 0, [a_name], [])
	_check(actor != -1, "an actor was added")
	_check(ctx.get_hub_choice_count(conv, 2, actor) == 1, "the choice line became a choice")
	_check(ctx.get_hub_choice_link(conv, 2, actor, 0) == 0, "the choice leads to the tagged hub")

	# Unresolved tags are reported as load errors rather than dropped silently.
	var bad_conv := ctx.make_name(&"readable_bad")
	var bad_document := "first\n> missing\n\nsecond\nlink: missing\n"
	var bad_loaded := ctx.load_conversation_text(bad_conv, bad_document)
	_check(bad_loaded == 0, "a readable document with bad tags still loads (code %d)" % bad_loaded)

	var bad_failures := 0
	for i in range(ctx.get_event_count()):
		var event := ctx.get_event(i)
		if event.get_load_failure() != null:
			bad_failures += 1
	ctx.flush_events()
	_check(bad_failures == 2, "two unresolved tags are reported (saw %d)" % bad_failures)

	if _failures > 0:
		get_tree().quit(1)


func _check(condition: bool, message: String) -> void:
	if not condition:
		print("FAIL ", message)
		_failures += 1
