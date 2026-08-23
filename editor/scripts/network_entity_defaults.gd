extends Node

const TITLE_STYLEBOXES := {
	"default": 		preload("res://style/hub_title_stylebox.tres"),
	"start": 			preload("res://style/hub_title_stylebox_start.tres"),
	"auto": 			preload("res://style/hub_title_stylebox_auto.tres"),
	"spoke": 			preload("res://style/hub_title_stylebox_spoke.tres"),
	"auto_spoke": preload("res://style/hub_title_stylebox_auto_spoke.tres"),
}

const ENTITY_SOURCES:Dictionary[String, PackedScene] = {
	"hub":preload("res://entities/hub_entity.tscn"),
	"choice":preload("res://entities/choice_entity.tscn"),
	"speaker":preload("res://entities/speaker_entity.tscn"),
}

var entity_defaults:Dictionary[String, NetworkEntity]

var default_data:Dictionary[String, Dictionary]

func _ready() -> void:
	process_mode = Node.PROCESS_MODE_DISABLED
	
	for et in ENTITY_SOURCES:
		var e := ENTITY_SOURCES[et]
		var entity:NetworkEntity = e.instantiate()
		entity_defaults[entity.get_entity_type()] = entity
		default_data[entity.get_entity_type()] = entity.export()

func _exit_tree() -> void:
	for entity_type in entity_defaults:
		var entity := entity_defaults[entity_type]
		if is_instance_valid(entity):
			entity.free()
