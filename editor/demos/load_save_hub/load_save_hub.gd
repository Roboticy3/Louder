extends Node

@export var hub_data:Dictionary
@export var hub:NetworkEntity

func _ready() -> void:
	print("import test: inspect screen to see it matches the hub described in the editor")
	hub.import(hub_data)
	print("export test:")
	print(hub.export())
