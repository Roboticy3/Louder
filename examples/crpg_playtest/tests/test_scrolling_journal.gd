extends Node

const SCENE := "res://character_ui/character_ui_placeholder.tscn"

var _failures := 0

func _ready() -> void:
	var scene := load(SCENE) as PackedScene
	_check(scene != null, "scene loads")
	if scene == null:
		_finish()
		return
	var d := scene.instantiate() as Control
	add_child(d)
	await get_tree().process_frame

	_check(d.menu_bar != null, "menu bar present")
	_check(d.journal_button != null, "journal button present")
	_check(d.journal_overlay != null, "journal overlay present")
	_check(d.journal_log != null, "journal log present")
	_check(not d.is_journal_open(), "journal starts closed")

	if d.journal_button != null:
		d.journal_button.emit_signal("pressed")
		await get_tree().process_frame
	_check(d.is_journal_open(), "journal opens (got %s)" % d.is_journal_open())

	if d.journal_overlay != null:
		var close_btn := d.journal_overlay.get_node_or_null("CloseButton") as Button
		if close_btn != null:
			close_btn.emit_signal("pressed")
			await get_tree().process_frame
	_check(not d.is_journal_open(), "journal closes")

	d.queue_free()
	_finish()

func _finish() -> void:
	if _failures > 0:
		get_tree().quit(1)
	else:
		print("PASS journal UI works")

func _check(cond: bool, msg: String) -> void:
	print("%s %s" % ["PASS" if cond else "FAIL", msg])
	if not cond:
		_failures += 1