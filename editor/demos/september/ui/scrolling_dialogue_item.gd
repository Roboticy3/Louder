extends VBoxContainer

const BodyProperties := PSObjects.BodyProperties
const ChoiceProperties := PSObjects.ChoiceProperties

@export var element_container:Container

signal choice_pressed(choice: String)

#replace children in the element container with the body's speaker, text, and choices
func import(data:Dictionary[BodyProperties, Variant]):
	for c in element_container.get_children():
		element_container.remove_child(c)
		c.queue_free()
	
	var speaker := str(data.get(BodyProperties.speaker, ""))
	if !speaker.is_empty():
		var speaker_label := Label.new()
		speaker_label.text = speaker
		speaker_label.add_theme_font_size_override("font_size", 18)
		element_container.add_child(speaker_label)
	
	var text := str(data.get(BodyProperties.text, ""))
	if !text.is_empty():
		var text_label := Label.new()
		text_label.text = text
		text_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		element_container.add_child(text_label)
	
	for choice in data.get(BodyProperties.choices, []):
		if !(choice is Dictionary): continue
		var button := Button.new()
		button.text = str(choice.get(ChoiceProperties.from_choice, ""))
		button.pressed.connect(_on_choice_pressed.bind(button.text))
		element_container.add_child(button)

func clear_choices() -> void:
	for c in element_container.get_children():
		if c is Button:
			element_container.remove_child(c)
			c.queue_free()

func _on_choice_pressed(choice: String) -> void:
	choice_pressed.emit(choice)
