extends Node

@export var editor:GraphEdit
@export var pointer:PointerLogic
@export var debugger:OnScreenDebugger
@export var type_dropdown:OptionButton

var spawns_on_point := 0

var sources_ordered:Array[PackedScene]

func _ready() -> void:
	if type_dropdown:
		for et in NetworkEntityDefaults.ENTITY_SOURCES:
			type_dropdown.add_item(et)
			sources_ordered.append(NetworkEntityDefaults.ENTITY_SOURCES[et])
		type_dropdown.select(0)
	
	editor.gui_input.connect(_editor_gui_input)

func _editor_gui_input(event: InputEvent) -> void:
	
	if event.is_action_pressed("graph_create"):
		spawn_node(_selected_scene())
		editor.accept_event()
		return
	if event is InputEventMouseMotion:
		spawns_on_point = 0

func _selected_scene() -> PackedScene:
	if type_dropdown:
		return sources_ordered[type_dropdown.selected]
	return sources_ordered[0]

func spawn_node(scene: PackedScene) -> void:
	var node := scene.instantiate()
	editor.add_child(node)
	node.position_offset = pointer.get_graph_pointer_position() + Vector2.LEFT * node.size.x * 0.5 + Vector2.ONE * spawns_on_point 
	node.owner = editor.owner
	editor.set_selected(node)
	
	get_viewport().set_input_as_handled()
	spawns_on_point += 1
