extends Node

@export var hub_resource:HubResource
@export var hub:CustomGraphHub

func _ready() -> void:
	print("import test: inspect screen to see it matches the hub described in the editor")
	hub.import(hub_resource)
	print("export test:")
	print(hub.export())
