class_name HubEntityFlags
extends Node

var flags:int = 0 : 
	set(new_flags):
		if flags != new_flags:
			flags = new_flags
			for i in checkboxes.size():
				var c = checkboxes[i]
				c.set_pressed_no_signal(flags & 1 << i)
var checkboxes: Array[CheckBox] = []

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	var i := 0
	for c in get_children():
		if c is CheckBox:
			print("deserializing checkbox ", c, " with flag ", i)
			checkboxes.append(c)
			c.button_pressed = bool(flags & 1 << i)
			c.toggled.connect(_on_checkbox_toggled.bind(c))
			i += 1

func _on_checkbox_toggled(pressed: bool, checkbox: CheckBox) -> void:
	if pressed:
		for i in checkboxes.size():
			var c = checkboxes[i]
			if c == checkbox:
				print("serializing checkbox ", c, " with flag ", i)
				if pressed:
					flags |= 1 << i
				else:
					flags &= ~(1 << i)