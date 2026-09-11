extends Control

@export var data:CharacterSelectItemData
@export var description_label:RichTextLabel
@export var name_label:RichTextLabel
@export var portrait_rect:TextureRect
@export var button:CharacterSelectButton

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	description_label.text = data.description
	name_label.text =data.name
	portrait_rect.texture = data.portrait
	button.character = data.name
