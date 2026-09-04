extends Node

enum EventType {
	CHOICE = 0,
	CONNECT,
	DISCONNECT,
}

signal event(type:EventType, controller:PSController)

@export var controller_group := &"ps_controller"

#send a choice to the server
#the server then uses the `authority` and `conversation_path` to find a matching
#controller.
#if found, the resulting controller makes the choice, if available
#the server computes the move on that controller, then repeats the update down
#to all peers.
@rpc("any_peer", "call_local", "reliable")
func send_choice_to_server(controller_path:NodePath, choice: String) -> void:
	if !multiplayer.is_server():
		return
	
	var node := get_node_or_null(controller_path)
	if !(node is PSController):
		invalid_path_error(controller_path, node)
	
	var controller:PSController = node
	controller.traverse(choice)
	
	#other controllers in the same branch should be marked to follow this one
	var sync_paths:Array = []
	for c in get_tree().get_nodes_in_group(controller_group):
		if c is PSController:
			if c.get_branch() == controller.get_branch():
				#don't sync the controllers yet. 
				#sync_controller_data will call locally too and handle that
				sync_paths.append(c.get_path())
	
	sync_controller_data.rpc(controller_path, sync_paths)

@rpc("authority", "call_local", "reliable")
func sync_controller_data(controller_path_leader:NodePath, controller_paths:Array):
	var node_leader := get_node_or_null(controller_path_leader)
	if !(node_leader is PSController):
		invalid_path_error(controller_path_leader, node_leader)
		return
	
	var controller_leader:PSController = node_leader
	event.emit(EventType.CHOICE, controller_leader)
	
	for c in controller_paths:
		if !(c is NodePath):
			invalid_path_error(c, null)
			continue
		var controller = get_node_or_null(c)
		if controller is PSController:
			controller.follow(controller_leader)
			event.emit(EventType.CHOICE, controller)
		else:
			invalid_path_error(c, controller)
			continue

func invalid_path_error(path, result):
	push_error("Path ", path, " not a valid controller! Found ", result, " instead.")

#start a given controller, if not already started
#Eventually would want this to result in a sync pattern like above, since technically the
#server should be in charge of where controllers enter dialogue as well as how
#they traverse it.
var peer_controller:PSController
func connect_controller(controller_path:NodePath):
	var node := get_node_or_null(controller_path)
	if !(node is PSController):
		invalid_path_error(controller_path, node)
	
	var controller:PSController = node
	if controller.is_active():
		return
	
	var result := controller.start()
	
	if !result:
		push_error("Failed to start simulation on controller ", controller_path, "!")
		return
	
	if peer_controller:
		event.emit(EventType.DISCONNECT, peer_controller)
	peer_controller = controller
	event.emit(EventType.CONNECT, controller)

func disconnect_controller(controller_path:NodePath):
	var node := get_node_or_null(controller_path)
	if !(node is PSController):
		invalid_path_error(controller_path, node)
	
	var controller:PSController = node
	if peer_controller == controller:
		peer_controller = null
		event.emit(EventType.DISCONNECT, controller)
