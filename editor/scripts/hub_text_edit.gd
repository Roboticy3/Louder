class_name CustomGraphNodeTextEdit extends CodeEdit

@export var oneliner := true
@export var inliner := false

func _ready() -> void:
	text_changed.connect(_on_text_changed)

func _on_text_changed():
	if oneliner:
		var cleaned = text.replace("\n", "").replace("\r", "")
		if cleaned != text:
			text = cleaned

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