extends Node

@export var simulator: PSSimulator

var _components: Array[Node] = []

func _ready() -> void:
	_collect_ui_components(self, _components)
	for component in _components:
		if component is PSChoicesContainer:
			component.choice_pressed.connect(_on_choice_pressed)
	update()

func _collect_ui_components(node: Node, components: Array[Node]) -> void:
	for child in node.get_children():
		if child.is_in_group("ui_component"):
			components.append(child)
		_collect_ui_components(child, components)

func update() -> void:
	for component in _components:
		if component is RichTextLabel:
			component.text = simulator.get_current_property(component.get_meta("property"))
		elif component is PSChoicesContainer:
			component.set_choices(simulator.list_choices())

func _on_choice_pressed(index: int) -> void:
	if simulator.traverse_index(index):
		update()
