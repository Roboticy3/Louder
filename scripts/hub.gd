class_name CustomGraphHub extends GraphNode

@export var hub_resource:HubResource :
	set(new_hub_resource):
		if new_hub_resource:
			import(new_hub_resource)
		hub_resource = new_hub_resource

@export var speaker:LineEdit
@export var text_content:TextEdit
@export var dc_check:CheckBox
@export var dc_skill_option:OptionButton
@export var dc_level_spin:SpinBox
@export var start_check:CheckBox
@export var choices_logic:ChoicesSectionLogic

func export() -> HubResource:
	var res := HubResource.new()
	res.speaker = speaker.text
	res.text_content = text_content.text
	res.dc_enabled = dc_check.button_pressed
	res.position_offset = position_offset

	var idx := dc_skill_option.selected
	res.dc_skill = dc_skill_option.get_item_text(idx) if idx >= 0 else ""

	res.dc_level = int(dc_level_spin.value)

	res.start_enabled = start_check.button_pressed

	for c in choices_logic.choices:
		res.choices.append(c.text)

	return res

func import(res: HubResource) -> void:
	speaker.text = res.speaker
	text_content.text = res.text_content
	dc_check.button_pressed = res.dc_enabled

	if not res.dc_skill.is_empty():
		for i in dc_skill_option.item_count:
			if dc_skill_option.get_item_text(i) == res.dc_skill:
				dc_skill_option.selected = i
				break

	dc_level_spin.value = res.dc_level
	
	start_check.button_pressed = res.start_enabled

	choices_logic.set_choice_count(res.choices.size())
	for i in res.choices.size():
		choices_logic.choices[i].text = res.choices[i]

func _ready() -> void:
	choices_logic.choices_changed.connect(_on_choices_changed)
	_on_choices_changed()

func _on_slot_updated(_slot):
	_on_choices_changed()

func _on_choices_changed():
	if choices_logic.choices.is_empty():
		title = "Spoke"
	else:
		title = "Hub"
	
	if hub_resource:
		title += " " + str(hub_resource.get_rid().get_id())
	
	name = title
