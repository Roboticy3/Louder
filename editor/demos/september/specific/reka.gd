extends Node

@export var view:MainDialogueView
@export var item_backgrounds:Array[Texture]
@export var text_color_main := Color.BLACK

func _ready() -> void:
	view.scrolling_dialogue_feed.item_backgrounds = item_backgrounds 
	view.scrolling_dialogue_feed.text_color_main = text_color_main
