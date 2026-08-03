class_name CustomGraphHub extends GraphNode

const COMPONENT_GROUP := &"hub_component"

@export var hub_data:Dictionary :
	set(new_hub_data):
		import(new_hub_data)
		hub_data = new_hub_data

func export() -> Dictionary:
	var data := {
		"conversation_id": id,
		"position_offset": position_offset,
	}
	for child in find_children("*", "", true, false):
		if child.is_in_group(COMPONENT_GROUP):
			data[get_path_to(child) as String] = _export_node(child)
	return data

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

func import(data: Dictionary) -> void:
	IdSystem.assign_id(self, data.get("conversation_id", id), id)
	id = data.get("conversation_id", id)
	
	position_offset = data.get("position_offset", position_offset)

	for path in data:
		if path is String:
			var node := get_node_or_null(NodePath(path))
			if node:
				_import_node(node, data[path])

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
		node.button_pressed = value
	elif node is SpinBox:
		node.value = value
	elif node is Button:
		node.text = value

func get_choices_logic() -> ChoicesSectionLogic:
	return get_node_or_null("ChoicesSection/ChoicesSectionLogic") as ChoicesSectionLogic

func get_start_check() -> CheckBox:
	return get_node_or_null("ChoicesSection/CheckBox") as CheckBox

var id := -1
const ID_GROUP := &"CONVERSATION"

func _ready() -> void:
	id = IdSystem.assign_id(self)
