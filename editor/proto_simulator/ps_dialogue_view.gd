extends Node

@export var group := &"ps_conversation_view"

@export var controller: PSController
@export var debug_speaker_label: Label

var _components: Array[Node] = []

func _ready() -> void:
	_collect_ui_components(self, _components)
	
	#is simulator ready to feed choices?
	if controller:
		if controller.is_active():
			update()
		controller.updated.connect(update)
	
	if debug_speaker_label:
		if controller:
			debug_speaker_label.text = ",".join(controller.speak_as) + ((" (branch: %s)" % controller.get_branch()))
		else:
			debug_speaker_label.text = "(no controller connected)"
	
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
			var text = controller.data.get(component.name)
			if text is String:
				component.text = text
		elif component is PSChoicesContainer:
			component.set_choices(controller.get_choices())
