class_name PSChoicesContainer
extends VBoxContainer

signal choice_pressed(index: int)

func set_choices(choices: Array) -> void:
	for child in get_children():
		child.queue_free()

	for i in choices.size():
		var button := Button.new()
		button.text = str(choices[i].get("from_choice", ""))
		if button.text.is_empty():
			button.text = "CONTINUE"
		button.pressed.connect(_on_choice_pressed.bind(i))
		add_child(button)

	if choices.is_empty():
		var button := Button.new()
		button.text = "END"
		add_child(button)

func _on_choice_pressed(index: int) -> void:
	choice_pressed.emit(index)
