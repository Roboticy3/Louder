extends SpinBox

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	get_line_edit().gui_input.connect(_on_line_edit)

func _on_line_edit(event: InputEvent) -> void:
	if event.is_action("ui_up") and event.is_pressed():
		value += 1
	elif event.is_action("ui_down") and event.is_pressed():
		value -= 1
