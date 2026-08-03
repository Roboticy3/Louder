extends Node

var default_data:Dictionary[String, Dictionary]

func _ready() -> void:
	process_mode = Node.PROCESS_MODE_DISABLED
	
	var hub:CustomGraphHub = load("res://entities/hub.tscn").instantiate()
	default_data["hub"] = hub.export()
	hub.free()
