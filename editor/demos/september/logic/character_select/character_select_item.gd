extends Control

@export var data:CharacterSelectItemData
@export var description_label:RichTextLabel
@export var name_label:RichTextLabel
@export var portrait_rect:TextureRect
@export var background_rect:NinePatchRect
@export var button:CharacterSelectButton

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	description_label.text = data.description
	name_label.text =data.name
	description_label.add_theme_color_override("default_color", data.text_color)
	name_label.add_theme_color_override("default_color", data.text_color)
	portrait_rect.texture = data.portrait
	background_rect.texture = data.background
	button.character = data.name
