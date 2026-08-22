extends Node

@export var group := &"ps_conversation_view"

@export var controller: PSController

var _components: Array[Node] = []

func _ready() -> void:
	_collect_ui_components(self, _components)
	
	#is simulator ready to feed choices?
	if controller:
		if controller.is_active():
			update()
		controller.updated.connect(update)

func _collect_ui_components(node: Node, components: Array[Node]) -> void:
	for child in node.get_children():
		if child.is_in_group("ui_component"):
			components.append(child)
		if child is PSChoicesContainer and controller is PSController:
			child.choice_pressed.connect(controller.traverse)
		_collect_ui_components(child, components)

func update() -> void:
	for component in _components:
		if component is RichTextLabel:
			component.text = controller.data.get(component.name)
		elif component is PSChoicesContainer:
			component.set_choices(controller.get_choices())
