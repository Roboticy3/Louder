extends Node

@export var choices_container:Container
@export var speaker_label:RichTextLabel
@export var text_label:RichTextLabel
@export var simulator:PSSimulator

func update():
	update_text_label()
	update_choices_container()

func update_text_label():
	speaker_label.text = simulator.get_current_property("Speaker")
	text_label.text = simulator.get_current_property("TextContent")

func update_choices_container():
	for child in choices_container.get_children():
		child.queue_free()

	var choices := simulator.list_choices()
	for i in choices.size():
		var button := Button.new()
		button.text = str(choices[i].get("from_choice", "Choice %d" % (i + 1)))
		if button.text.is_empty():
			button.text = "CONTINUE"
		button.pressed.connect(_on_choice_pressed.bind(i))
		choices_container.add_child(button)

func _on_choice_pressed(index: int) -> void:
	if simulator.traverse_index(index):
		update()
