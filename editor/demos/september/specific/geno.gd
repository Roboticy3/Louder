extends Node

@export var view:MainDialogueView
@export var item_backgrounds:Array[Texture]

func _ready() -> void:
	view.scrolling_dialogue_feed.item_backgrounds = item_backgrounds 
