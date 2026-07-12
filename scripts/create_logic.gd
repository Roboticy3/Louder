extends Node

@export var editor:GraphEdit
@export var pointer:PointerLogic
@export var debugger:OnScreenDebugger

const HUB_SCENE := preload("res://entities/hub.tscn")

var spawns_on_point := 0

func _input(event: InputEvent) -> void:
	
	if event.is_action_pressed("graph_create"):
		
		
		var hub := HUB_SCENE.instantiate()
		editor.add_child(hub)
		hub.position_offset = pointer.get_graph_pointer_position() + Vector2.LEFT * hub.size.x * 0.5 + Vector2.ONE * 0.1 * spawns_on_point 
		hub.owner = editor.owner
		editor.set_selected(hub)
		
		get_viewport().set_input_as_handled()
		spawns_on_point += 1
	if event is InputEventMouseMotion:
		spawns_on_point = 0
