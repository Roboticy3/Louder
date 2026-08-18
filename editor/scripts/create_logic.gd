extends Node

@export var editor:GraphEdit
@export var pointer:PointerLogic
@export var debugger:OnScreenDebugger
@export var type_dropdown:OptionButton

const HUB_SCENE := preload("res://entities/hub.tscn")
const SPEAKER_SCENE := preload("res://entities/speaker.tscn")

var spawns_on_point := 0

func _ready() -> void:
	if type_dropdown:
		type_dropdown.add_item("Hub")
		type_dropdown.add_item("Speaker")
		type_dropdown.select(0)

func _input(event: InputEvent) -> void:
	
	if event.is_action_pressed("graph_create"):
		spawn_node(_selected_scene())
		return
	if event is InputEventMouseMotion:
		spawns_on_point = 0

func _gui_input(event:InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_1:
				if type_dropdown:
					type_dropdown.select(0)
				spawn_node(HUB_SCENE)
				return
			KEY_2:
				if type_dropdown:
					type_dropdown.select(1)
				spawn_node(SPEAKER_SCENE)
				return

func _selected_scene() -> PackedScene:
	if type_dropdown and type_dropdown.selected == 1:
		return SPEAKER_SCENE
	return HUB_SCENE

func spawn_node(scene: PackedScene) -> void:
	var node := scene.instantiate()
	editor.add_child(node)
	node.position_offset = pointer.get_graph_pointer_position() + Vector2.LEFT * node.size.x * 0.5 + Vector2.ONE * 0.1 * spawns_on_point 
	node.owner = editor.owner
	editor.set_selected(node)
	
	get_viewport().set_input_as_handled()
	spawns_on_point += 1
