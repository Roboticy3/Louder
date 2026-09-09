extends VBoxContainer

const BodyProperties := PSObjects.BodyProperties
const ChoiceProperties := PSObjects.ChoiceProperties

@export var element_container:Container
@export var choice_container:Container
@export var element_background:NinePatchRect
@export var disabled := false
@export var choice_packed:PackedScene = load("res://demos/september/ui/scrolling_dialogue_choice.tscn")

@export var textures:Array[Texture]

signal choice_pressed(choice: String)

func set_textures(to:Array[Texture]):
	textures = to
	if element_background: element_background.texture = get_random_texture()

#replace children in the element container with the body's speaker, text, and choices
func set_controller(controller:PSController):
	for c in element_container.get_children():
		element_container.remove_child(c)
		c.queue_free()
	
	var index := get_index()
	var index_label := Label.new()
	index_label.text = str(index)
	
	var data := controller.data
	var speaker := str(data.get(BodyProperties.speaker, ""))
	if !speaker.is_empty():
		var speaker_label := Label.new()
		speaker_label.text = speaker
		speaker_label.add_theme_font_size_override("font_size", 18)
		element_container.add_child(speaker_label)
	
	var text := str(data.get(BodyProperties.text, ""))
	if !text.is_empty():
		var text_label := RichTextLabel.new()
		text_label.bbcode_enabled = true
		text_label.fit_content = true
		text_label.text = text
		text_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
		element_container.add_child(text_label)
	
	if !disabled: 
		var choices := controller.get_choices()
		for i in choices.size():
			var choice := choices[i]
			if !(choice is Dictionary): continue
			
			var choice_instance = choice_packed.instantiate()
			if !choice_instance is ScrollingDialogueChoice:
				push_error("Choice ", choice_instance, " loaded from ", choice_packed.resource_path, ". Expected ScrollingDialogueChoice")
				break
			
			var choice_text := str(choice.get(ChoiceProperties.from_choice, ""))
			choice_instance.set_choice(choice_text)
			choice_instance.set_texture(get_random_texture())
			
			if i < 10:
				choice_instance.set_index(i + 1)
			
			choice_instance.choice_pressed.connect(_on_choice_pressed)
			choice_container.add_child(choice_instance)
	

func clear_choices() -> void:
	for c in choice_container.get_children():
		choice_container.remove_child(c)
		c.queue_free()

func get_random_texture() -> Texture:
	if textures.is_empty():
		return Texture.new()
	var i := randi_range(0, textures.size() - 1)
	return textures[i]

func _on_choice_pressed(choice: String) -> void:
	choice_pressed.emit(choice)
