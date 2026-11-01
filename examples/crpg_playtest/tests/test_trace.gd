extends Node

## Checks that the engine keeps a trace of every event and ask it creates.
##
## The trace is the history, not the work in progress: flushing the event queue
## and answering asks must leave it untouched, and the whole thing has to be
## readable both entry by entry and as the JSON document `LouderJson.trace`
## prints.


func _ready() -> void:
	var ctx := LouderContext.new()
	var conversation := ctx.make_name(&"trace")

	var loaded := ctx.load_conversation(conversation, "[]")
	if loaded != 0:
		_fail("loading an empty conversation failed with %d" % loaded)
		return

	if ctx.get_traced_event_count() != 0:
		_fail("loading an empty conversation should not trace events")

	var actor := ctx.add_actor(conversation, 0, 0, [], [])
	if actor == -1:
		_fail("adding an actor failed")
		return

	if ctx.get_event_count() != 1:
		_fail("adding an actor should queue one event")
	if ctx.get_traced_event_count() != 1:
		_fail("adding an actor should trace one event")

	var traced := ctx.get_traced_event(0)
	if traced == null:
		_fail("the traced event should be readable")
		return
	if traced.get_kind() != 12:
		_fail("the traced event should be ActorAdded")
	if traced.get_actor() != actor:
		_fail("the traced event should name the actor")

	ctx.flush_events()
	if ctx.get_event_count() != 0:
		_fail("flushing should empty the event queue")
	if ctx.get_traced_event_count() != 1:
		_fail("flushing should leave the event trace alone")

	var ask := LouderAsk.new()
	ctx.push_ask(ask)
	if ctx.get_traced_ask_count() != 1:
		_fail("pushing an ask should trace it")
	if not ctx.answer_ask(0, 1):
		_fail("answering the ask should succeed")
	if ctx.get_traced_ask_count() != 1:
		_fail("answering should leave the ask trace alone")

	var json := LouderJson.trace(ctx)
	if not json.contains("\"events\""):
		_fail("the trace JSON should have an events array")
	if not json.contains("\"asks\""):
		_fail("the trace JSON should have an asks array")
	if not json.contains("\"kind\":12"):
		_fail("the trace JSON should describe the traced event")

	ctx.clear_traced_events()
	if ctx.get_traced_event_count() != 0 or ctx.get_traced_ask_count() != 1:
		_fail("clearing events should only empty the event trace")

	ctx.clear_trace()
	if ctx.get_traced_event_count() != 0 or ctx.get_traced_ask_count() != 0:
		_fail("clearing the trace should empty both halves")

	print("PASS traces survive processing and are readable from Godot")


func _fail(message: String) -> void:
	print("FAIL ", message)
	get_tree().quit(1)
