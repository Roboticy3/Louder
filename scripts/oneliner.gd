extends CodeEdit

func _ready() -> void:
	text_changed.connect(_on_text_changed)

func _on_text_changed():
	var cleaned = text.replace("\n", "").replace("\r", "")
	if cleaned != text:
		text = cleaned
