class_name PSChoicesContainer
extends VBoxContainer

signal choice_pressed(choice: String)

func set_choices(choices: Array) -> void:
	for child in get_children():
		child.queue_free()

	for i in choices.size():
		var button := Button.new()
		var choice := str(choices[i].get("from_choice", ""))
		button.text = choice
		if button.text.is_empty():
			button.text = "CONTINUE"
		button.pressed.connect(_on_choice_pressed.bind(choice))
		add_child(button)

	if choices.is_empty():
		var button := Button.new()
		button.text = "END"
		add_child(button)

func _on_choice_pressed(choice: String) -> void:
	choice_pressed.emit(choice)
