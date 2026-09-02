class_name PSChoicesContainer
extends VBoxContainer

const ChoiceProperties := PSObjects.ChoiceProperties

signal choice_pressed(choice: String)

func set_choices(choices: Array) -> void:
	for child in get_children():
		child.queue_free()

	for i in choices.size():
		var button := Button.new()
		var choice := str(choices[i].get(ChoiceProperties.from_id, ""))
		if choice.is_empty():
			button.text = "CONTINUE"
		else:
			button.text = choice
		button.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		
		button.pressed.connect(_on_choice_pressed.bind(choice))
		add_child(button)
	

func set_thinking(thinking:Array) -> void:
	if !thinking.is_empty():
		var thinking_label := Label.new()
		thinking_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
		
		var text := ""
		if thinking.size() > 1:
			text = ", ".join(thinking.slice(0, -1))
			text += " and " + thinking[-1] + " are thinking..."
		else:
			text = thinking[0] + " is thinking..."
		thinking_label.text = text
		add_child(thinking_label)

func _on_choice_pressed(choice: String) -> void:
	choice_pressed.emit(choice)
