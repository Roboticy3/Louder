extends Node

var _failures := 0

func _ready() -> void:
	var scenes := [
		"res://ui/conversation_log.tscn",
		"res://ui/conversation_log_menu.tscn",
		"res://ui/conversation_history.tscn",
		"res://ui/dialogue_log.tscn",
		"res://ui/chat_log.tscn",
	]
	for path in scenes:
		var scene := load(path) as PackedScene
		_check(scene != null, "scene loads: %s" % path)
		if scene == null:
			continue
		var node := scene.instantiate()
		add_child(node)
		_check(node.container != null, "container present: %s" % path)
		node.queue_free()
	await get_tree().process_frame
	if _failures > 0:
		get_tree().quit(1)
	else:
		print("PASS all alias scenes load")

func _check(cond: bool, msg: String) -> void:
	print("%s %s" % ["PASS" if cond else "FAIL", msg])
	if not cond:
		_failures += 1