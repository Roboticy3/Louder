@tool
extends EditorScript

func _run() -> void:
	var tl := TestLinkage.new()
	tl.say_hello()
