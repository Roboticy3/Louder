extends Node

const TITLE_STYLEBOXES := {
	"default": 		preload("res://style/hub_title_stylebox.tres"),
	"start": 			preload("res://style/hub_title_stylebox_start.tres"),
	"auto": 			preload("res://style/hub_title_stylebox_auto.tres"),
	"spoke": 			preload("res://style/hub_title_stylebox_spoke.tres"),
	"auto_spoke": preload("res://style/hub_title_stylebox_auto_spoke.tres"),
}

var default_data:Dictionary[String, Dictionary]

func _ready() -> void:
	process_mode = Node.PROCESS_MODE_DISABLED
	
	var hub:NetworkEntity = load("res://entities/hub_entity.tscn").instantiate()
	default_data["hub"] = hub.export()
	hub.free()
	
	var speaker:NetworkEntity = load("res://entities/speaker_entity.tscn").instantiate()
	default_data["speaker"] = speaker.export()
	speaker.free()
	
	var choice:NetworkEntity = load("res://entities/choice_entity.tscn").instantiate()
	default_data["choice"] = choice.export()
	choice.free()
