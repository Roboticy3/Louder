extends TextEdit


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass # Replace with function body.

func _gui_input(event: InputEvent) -> void:
	if event.is_action("ui_focus_prev") and event.is_pressed():
		accept_event()
		var n := find_prev_valid_focus()
		if n:
			n.grab_focus()
			return
	if event.is_action("ui_focus_next") and event.is_pressed():
		#print("ui_focus_next pressed on ", self)
		accept_event()
		var n := find_next_valid_focus()
		if n:
			n.grab_focus()
			return
		
