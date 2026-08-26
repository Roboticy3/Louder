class_name NetworkEntity extends GraphNode

@export var component_group := &"hub_component"
@export var default_source := &"hub"
@export var choices_section_logic:ChoicesSectionLogic

@export var hub_data:Dictionary :
	set(new_hub_data):
		import(new_hub_data)
		hub_data = new_hub_data

func export() -> Dictionary:
	var data := {
		"conversation_id": id,
		"position_offset": position_offset,
	}
	var seen_names := {}
	for child in find_children("*", "", true, false):
		if child.is_in_group(component_group):
			if seen_names.has(child.name):
				push_error("Hub \"%s\" has duplicate component name \"%s\"; aborting export" % [name, child.name])
				return {}
			seen_names[child.name] = true
			data[child.name as String] = _export_node(child)
	return _filter_to_non_default(data)

func import(data: Dictionary) -> void:
	hub_data.merge(_filter_to_non_default(data), true)
	IdSystem.assign_id(self, data.get("conversation_id", id), id)
	id = hub_data.get("conversation_id", id)
	
	position_offset = hub_data.get("position_offset", position_offset)

	for key in hub_data:
		if key is String:
			var node := find_child(key, true, false)
			if node:
				_import_node(node, hub_data[key])

func _filter_to_non_default(data: Dictionary) -> Dictionary:
	var defaults: Dictionary = NetworkEntityDefaults.default_data.get(default_source, {})
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
			var text:String = c.text
			if text.is_empty():
				text = c.placeholder_text
			choices.append(text)
		return choices
	elif node is CodeEdit:
		return node.text
	elif node is ColorPickerButton:
		return node.color.to_html()
	elif node is OptionButton:
		var idx:int = node.selected
		return node.get_item_text(idx) if idx >= 0 else ""
	elif node is CheckBox:
		return node.button_pressed
	elif node is SpinBox:
		return node.value
	elif node is Button:
		return node.text
	return null

func _import_node(node: Node, value: Variant) -> void:
	if node is ChoicesSectionLogic:
		var choices: PackedStringArray = value
		node.set_choice_count(choices.size())
		for i in choices.size():
			if choices[i] != node.choices[i].placeholder_text:
				node.choices[i].text = choices[i]
	elif node is CodeEdit:
		node.text = value
	elif node is ColorPickerButton:
		node.color = Color.html(value)
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

func get_entity_type() -> String:
	return "network_entity"

func get_choices_logic() -> ChoicesSectionLogic:
	return choices_section_logic

func get_data(field:String, default):
	return hub_data.get(field, default)

var id := -1
const ID_GROUP := &"CONVERSATION"

func _ready() -> void:
	if id < 0:
		id = IdSystem.assign_id(self)
