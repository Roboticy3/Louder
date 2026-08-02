class_name CustomGraphHub extends GraphNode

@export var hub_data:Dictionary :
	set(new_hub_data):
		import(new_hub_data)
		hub_data = new_hub_data

@export var speaker:CodeEdit
@export var text_content:CodeEdit
@export var dc_check:CheckBox
@export var dc_skill_option:OptionButton
@export var dc_level_spin:SpinBox
@export var expressions_editor:CodeEdit
@export var start_check:CheckBox
@export var choices_logic:ChoicesSectionLogic

func export() -> Dictionary:
	var data := {
		"conversation_id": id,
		"speaker": speaker.text,
		"text_content": text_content.text,
		"dc_enabled": dc_check.button_pressed,
		"position_offset": position_offset,
	}

	var idx := dc_skill_option.selected
	data["dc_skill"] = dc_skill_option.get_item_text(idx) if idx >= 0 else ""

	data["expressions"] = expressions_editor.text

	data["dc_level"] = int(dc_level_spin.value)

	data["start_enabled"] = start_check.button_pressed

	var choices: PackedStringArray = []
	for c in choices_logic.choices:
		choices.append(c.text)
	data["choices"] = choices

	return data

func import(data: Dictionary) -> void:
	IdSystem.assign_id(self, data.get("conversation_id", id), id)
	id = data.get("conversation_id", id)
	
	speaker.text = data.get("speaker", "")
	text_content.text = data.get("text_content", "")
	dc_check.button_pressed = data.get("dc_enabled", false)

	var dc_skill := data.get("dc_skill", "") as String
	if not dc_skill.is_empty():
		for i in dc_skill_option.item_count:
			if dc_skill_option.get_item_text(i) == dc_skill:
				dc_skill_option.selected = i
				break

	dc_level_spin.value = data.get("dc_level", 0)
	
	expressions_editor.text = data.get("expressions", "")
	
	start_check.button_pressed = data.get("start_enabled", false)

	var choices: PackedStringArray = data.get("choices", PackedStringArray())
	choices_logic.set_choice_count(choices.size())
	for i in choices.size():
		choices_logic.choices[i].text = choices[i]

var id := -1
const ID_GROUP := &"CONVERSATION"

func _ready() -> void:
	id = IdSystem.assign_id(self)
