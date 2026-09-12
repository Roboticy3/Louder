extends Node

@export var state_listener_group := &"state_listener"
@export var controller_group := &"ps_controller"

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	add_to_group(state_listener_group)
	
	ControllerEventLayer.choice_event.connect(_on_choice)

var victims:Array
var warp:int = -1
func _on_state_changed(s:PSSimulator, variable:String, value:Variant):
	if !multiplayer.is_server():
		return
	
	if !variable.ends_with("warp"):
		return
	
	warp = int(value)
	if !s.is_position(warp):
		return
	
	var wow := who_owns_what()
	for w in wow.keys():
		if variable.begins_with(w):
			victims = wow[w]
			break

func _on_choice(_c:PSController):
	if !multiplayer.is_server():
		return
	if !victims.is_empty():
		var victims_copy = victims.duplicate()
		victims = []
		for v in victims_copy:
			if !(v is PSController): continue
			ControllerEventLayer.request_warp(v, warp)

func who_owns_what() -> Dictionary[String, Array]:
	var result:Dictionary[String, Array]
	for c in get_tree().get_nodes_in_group(controller_group):
		if c is PSController:
			for a in c.speak_as:
				if a in result:
					result[a].append(c)
				else:
					result[a] = [c]
	return result
