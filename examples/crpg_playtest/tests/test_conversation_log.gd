extends Node

const SCENE := "res://ui/conversation_log.tscn"

var _failures := 0

func _ready() -> void:
	var saved_context := LouderContextGlobal.context
	var context := LouderContext.new()
	LouderContextGlobal.context = context

	var log := (load(SCENE) as PackedScene).instantiate()
	log.max_logs = 2
	add_child(log)
	await get_tree().process_frame

	_check(log.visible or true, "the log runs")
	_check(log.container != null, "the scene gives the log its container")
	if log.container == null:
		_finish(saved_context)
		return

	_check(log.container.get_child_count() == 0, "a fresh trace has no labels")
	_check(log.entries.size() == 0, "a fresh trace has no entries")

	var file := FileAccess.open("res://conversations/anonymous.json", FileAccess.READ)
	var json_text := file.get_as_text()
	var conversation := context.make_name("res://conversations/anonymous.json")
	var loaded := context.load_conversation(conversation, json_text)
	_check(loaded == 0, "the conversation loads (code %d)" % loaded)

	var actor := context.add_actor(conversation, 0, 0, [], [])
	context.flush_events()
	# Announce the opening hub so a motion event is traced.
	if actor >= 0:
		LouderContextGlobal.announce_position(conversation, actor)
	await get_tree().process_frame

	print("after one motion: labels=%d entries=%d" % [log.container.get_child_count(), log.entries.size()])
	_check(log.entries.size() >= 1, "a hub appearance was logged (got %d)" % log.entries.size())

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