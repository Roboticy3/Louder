class_name CustomGraphHub extends GraphNode

const COMPONENT_GROUP := &"hub_component"

@export var hub_data:Dictionary :
	set(new_hub_data):
		import(new_hub_data)
		hub_data = new_hub_data
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

func export() -> Dictionary:
	var data := {
		"conversation_id": id,
		"position_offset": position_offset,
	}
	var seen_names := {}
	for child in find_children("*", "", true, false):
		if child.is_in_group(COMPONENT_GROUP):
			if seen_names.has(child.name):
				push_error("Hub \"%s\" has duplicate component name \"%s\"; aborting export" % [name, child.name])
				return {}
			seen_names[child.name] = true
			data[child.name as String] = _export_node(child)
	return _filter_to_non_default(data)

func import(data: Dictionary) -> void:
	data = _filter_to_non_default(data)
	IdSystem.assign_id(self, data.get("conversation_id", id), id)
	id = data.get("conversation_id", id)
	
	position_offset = data.get("position_offset", position_offset)

	for key in data:
		if key is String:
			var node := find_child(key, true, false)
			if node:
				_import_node(node, data[key])

func _filter_to_non_default(data: Dictionary) -> Dictionary:
	var defaults: Dictionary = HubDefaults.default_data.get("hub", {})
	var filtered := {}
	for key in data:
		if not defaults.has(key) or _compare_data(data, defaults, key):
			filtered[key] = data[key]
	return filtered

func _compare_data(x:Dictionary, y:Dictionary, key):
	return str(x[key]) != str(y[key])

func _export_node(node: Node) -> Variant:
	if node is ChoicesSectionLogic:
		var choices: PackedStringArray = []
		for c in node.choices:
			choices.append(c.text)
		return choices
	if node is CodeEdit:
		return node.text
	if node is OptionButton:
		var idx:int = node.selected
		return node.get_item_text(idx) if idx >= 0 else ""
	if node is CheckBox:
		return node.button_pressed
	if node is SpinBox:
		return node.value
	if node is Button:
		return node.text
	return null

func _import_node(node: Node, value: Variant) -> void:
	if node is ChoicesSectionLogic:
		var choices: PackedStringArray = value
		node.set_choice_count(choices.size())
		for i in choices.size():
			node.choices[i].text = choices[i]
	elif node is CodeEdit:
		node.text = value
	elif node is OptionButton:
		var skill := value as String
		if not skill.is_empty():
			for i in node.item_count:
				if node.get_item_text(i) == skill:
					node.selected = i
					break
	elif node is CheckBox:
		node.button_pressed = bool(value)
	elif node is SpinBox:
		node.value = int(value)
	elif node is Button:
		node.text = value

func update_title_stylebox() -> void:
	var start := start_check.button_pressed
	var auto := auto_check.button_pressed
	var spoke := choices_section_logic.choices.is_empty()

	var stylebox := HubDefaults.TITLE_STYLEBOXES["default"]
	if start:
		stylebox = HubDefaults.TITLE_STYLEBOXES["start"]
	elif auto:
		if spoke:
			stylebox = HubDefaults.TITLE_STYLEBOXES["auto_spoke"]
		else:
			stylebox = HubDefaults.TITLE_STYLEBOXES["auto"]
	elif spoke:
		stylebox = HubDefaults.TITLE_STYLEBOXES["spoke"]
	
	add_theme_stylebox_override("titlebar", stylebox)
	add_theme_stylebox_override("titlebar_selected", stylebox)


var id := -1
const ID_GROUP := &"CONVERSATION"

func _ready() -> void:
	id = IdSystem.assign_id(self)

	auto_check.pressed.connect(update_title_stylebox)
	start_check.pressed.connect(update_title_stylebox)
	choices_section_logic.choices_changed.connect(update_title_stylebox)
	update_title_stylebox.call_deferred()
