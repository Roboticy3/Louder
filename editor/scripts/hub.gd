class_name CustomGraphHub extends NetworkEntity

@export var grabber:Control
@export var start_check:CheckBox
@export var auto_check:CheckBox

func get_start_check() -> CheckBox:
	return start_check

func get_auto_check() -> CheckBox:
	return auto_check

func get_grabber() -> Control:
	return grabber

const TYPE := "hub"

func get_entity_type() -> String:
	return TYPE

func _ready() -> void:
	super._ready()
	
	title = str(id)
