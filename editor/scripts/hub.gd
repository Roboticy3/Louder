class_name CustomGraphHub extends NetworkEntity

@export var grabber:Control
@export var choices_section_logic:ChoicesSectionLogic
@export var start_check:CheckBox
@export var auto_check:CheckBox

func get_choices_logic() -> ChoicesSectionLogic:
	return choices_section_logic

func get_start_check() -> CheckBox:
	return start_check

func get_auto_check() -> CheckBox:
	return auto_check

func get_grabber() -> Control:
	return grabber

func update_title_stylebox() -> void:
	var start := start_check.button_pressed
	var auto := auto_check.button_pressed
	var spoke := choices_section_logic.choices.is_empty()

	var stylebox := NetworkEntityDefaults.TITLE_STYLEBOXES["default"]
	if start:
		stylebox = NetworkEntityDefaults.TITLE_STYLEBOXES["start"]
	elif auto:
		if spoke:
			stylebox = NetworkEntityDefaults.TITLE_STYLEBOXES["auto_spoke"]
		else:
			stylebox = NetworkEntityDefaults.TITLE_STYLEBOXES["auto"]
	elif spoke:
		stylebox = NetworkEntityDefaults.TITLE_STYLEBOXES["spoke"]
	
	add_theme_stylebox_override("titlebar", stylebox)
	add_theme_stylebox_override("titlebar_selected", stylebox)

func _ready() -> void:
	super._ready()
	
	auto_check.pressed.connect(update_title_stylebox)
	start_check.pressed.connect(update_title_stylebox)
	choices_section_logic.choices_changed.connect(update_title_stylebox)
	update_title_stylebox.call_deferred()
