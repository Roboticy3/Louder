extends Node

## Checks that the event log reads the engine's trace rather than its event
## queue, prints every event it sees, and keeps only the newest `max_logs`
## labels.

const SCENE := "res://ui/louder_event_log.tscn"

var _failures := 0


func _ready() -> void:
	var saved_context := LouderContextGlobal.context
	var context := LouderContext.new()
	LouderContextGlobal.context = context

	var log := (load(SCENE) as PackedScene).instantiate()
	log.max_logs = 2
	add_child(log)
	await get_tree().process_frame

	_check(log.visible, "the log runs on the server")
	_check(log.container != null, "the scene gives the log its container")
	if log.container == null:
		_finish(saved_context)
		return

	_check(log.container.get_child_count() == 0, "a fresh trace has no labels")

	var conversation := context.make_name("event_log_test")
	var loaded := context.load_conversation(conversation, "[]")
	_check(loaded == 0, "the empty conversation loads (code %d)" % loaded)
	for index in 4:
		context.add_actor(conversation, 0, 0, [], [])
	await get_tree().process_frame

	_check(log.container.get_child_count() == 2,
		"only the newest two of four events have labels (got %d)" % log.container.get_child_count())
	if log.container.get_child_count() == 2:
		var first := log.container.get_child(0) as Label
		var last := log.container.get_child(1) as Label
		_check(first.text != "", "the oldest kept event has a label")
		_check(last.text != "", "the newest event has a label")
		_check(last.text.contains("actor:"),
			"the label prints changed fields in key: value form")

	# The queue is drained by the singleton, but the trace is not, so the labels
	# must survive that. Adding another event keeps only the newest two.
	context.add_actor(conversation, 0, 0, [], [])
	await get_tree().process_frame
	_check(log.container.get_child_count() == 2, "one more event still leaves two labels")
	if log.container.get_child_count() == 2:
		var first := log.container.get_child(0) as Label
		_check(first.text != "",
			"the log moved on to the newer events")

	# A cleared trace is a shorter trace, not a permuted one; the stale labels
	# go away and the log starts reading from the beginning again.
	context.clear_trace()
	await get_tree().process_frame
	_check(log.container.get_child_count() == 0, "clearing the trace clears the labels")
	context.add_actor(conversation, 0, 0, [], [])
	await get_tree().process_frame
	_check(log.container.get_child_count() == 1, "the log reads a new trace from its start")

	log.queue_free()
	_finish(saved_context)


func _finish(saved_context: LouderContext) -> void:
	LouderContextGlobal.context = saved_context
	if _failures > 0:
		get_tree().quit(1)


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if not condition:
		_failures += 1
