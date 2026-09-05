extends Node

enum EventType {
	CHOICE = 0,
	CONNECT,
	DISCONNECT,
}

signal event(type:EventType, controller:PSController)

#eventually want to replace this with more intentional selection of simulators based on user input in the world
@export var simulator_group := &"ps_simulator"
@export var controller_group := &"ps_controller"

var connected_controller:PSController

#send a choice to the server
#the server then uses the `authority` and `conversation_path` to find a matching
#controller.
#if found, the resulting controller makes the choice, if available
#the server computes the move on that controller, then repeats the update down
#to all peers.
@rpc("any_peer", "call_local", "reliable")
func request_choice(controller:PSController, choice: String) -> void:
	if !multiplayer.is_server():
		return
	
	if !is_instance_valid(controller):
		return
	var controller_path := controller.get_path()
	var remote_authority := multiplayer.get_remote_sender_id()
	
	controller.traverse(choice)
	sync_controller_data.rpc(EventType.CHOICE, remote_authority, controller_path, controller.data)
	
	#other controllers in the same branch should be marked to follow this one
	for c in get_tree().get_nodes_in_group(controller_group):
		if c is PSController:
			if c != controller and c.follow(controller):
				sync_controller_data.rpc(EventType.CHOICE, remote_authority, c.get_path(), c.data)

@rpc("any_peer", "call_local", "reliable")
func request_controller(controller_path:NodePath):
	if !multiplayer.is_server():
		return
	
	var remote_authority := multiplayer.get_remote_sender_id()
	
	var node := get_node_or_null(controller_path)
	if !(node is PSController):
		invalid_path_error(controller_path, node)
		return
	
	var controller:PSController = node
	
	if controller.is_active():
		sync_controller_data.rpc(EventType.CONNECT, remote_authority, controller_path, controller.data)
		return
	
	#look for active controllers to sync to
	for c in get_tree().get_nodes_in_group(controller_group):
		if c is PSController:
			if controller.follow(c):
				sync_controller_data.rpc(EventType.CONNECT, remote_authority, controller_path, controller.data)
				break

	#no sync target found => compute the start of the conversation on the server
	#and replicate that.
	var result := controller.start()
	
	if !result:
		push_error("Failed to start simulation on controller ", controller.get_path(), "!")
		return
	
	sync_controller_data.rpc(EventType.CONNECT, remote_authority, controller_path, controller.data)

func disconnect_controller(controller:PSController):
	if connected_controller == controller:
		connected_controller = null
		event.emit(EventType.DISCONNECT, controller)

#sync uses paths because they are able to transmit over a network, unlike actual references
@rpc("authority", "call_local", "reliable")
func sync_controller_data(event_type:EventType, original_authority:int, controller_path: NodePath, data: Dictionary):
	var node := get_node_or_null(controller_path)
	if !(node is PSController):
		invalid_path_error(controller_path, node)
		return
	
	var controller:PSController = node
	controller.data = data
	
	if event_type == EventType.CONNECT or event_type == EventType.DISCONNECT:
		if multiplayer.get_unique_id() == original_authority:
			connected_controller = controller
			event.emit(event_type, controller)
	else:
		event.emit(event_type, controller)

func get_connected_controller() -> PSController:
	return connected_controller

func invalid_path_error(path, result):
	push_error("Path ", path, " not a valid controller! Found ", result, " instead.")
