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

func get_connected_controller() -> PSController:
	return connected_controller

# --- connecting / disconnecting ------------------------------------------
#
# Connecting and disconnecting are local concepts: they manage which
# controller THIS peer's UI drives (`connected_controller`). The only thing
# a connection can do that needs the server is *start* a brand new
# conversation, because that runs on the authoritative simulator. Following
# an active conversation, or failing to find one, works entirely on state
# that every peer already has, so those cases never touch the network.

func request_controller(controller:PSController) -> void:
	if !is_instance_valid(controller):
		return
	if !multiplayer.has_multiplayer_peer():
		return
	if controller.is_active():
		connect_local(controller)
		return
	
	for c in get_tree().get_nodes_in_group(controller_group):
		if c is PSController and c != controller and c.is_active() and c.simulator != null:
			if controller.follow(c):
				connect_local(controller)
				return
	
	_request_start.rpc_id(1, controller.get_path(), controller.speak_as, controller.listen_to)

func connect_local(controller:PSController) -> void:
	if connected_controller == controller:
		return
	
	connected_controller = controller
	event.emit(EventType.CONNECT, controller)

func disconnect_controller(controller:PSController) -> void:
	if connected_controller == controller:
		connected_controller = null
		event.emit(EventType.DISCONNECT, controller)

# --- choices -------------------------------------------------------------

func request_choice(controller:PSController, choice: String) -> void:
	if !is_instance_valid(controller) or choice.is_empty():
		return
	if !multiplayer.has_multiplayer_peer():
		return
	if multiplayer.is_server():
		_request_choice(controller.get_path(), choice)
	else:
		_request_choice.rpc_id(1, controller.get_path(), choice)

#the server is authoritative over the conversation, so it computes the move
#here, then repeats the update down to all peers. other controllers in the
#same branch are marked to follow this one.
@rpc("any_peer", "reliable")
func _request_choice(controller_path:NodePath, choice: String) -> void:
	if !multiplayer.is_server():
		return
	var controller := get_node_or_null(controller_path) as PSController
	if controller == null:
		invalid_path_error(controller_path, get_node_or_null(controller_path))
		return
	if !controller.traverse(choice):
		push_warning("Choice ", choice, " ignored for controller ", controller_path, " (not in a conversation yet)")
		return
	var remote_authority := get_requester_id()
	sync_controller_data.rpc(EventType.CHOICE, remote_authority, controller_path, controller.data)
	for c in get_tree().get_nodes_in_group(controller_group):
		if c is PSController and c != controller and c.simulator != null and c.follow(controller):
			sync_controller_data.rpc(EventType.CHOICE, remote_authority, c.get_path(), c.data)

# --- starting ------------------------------------------------------------

#computes the start of a conversation on the authoritative simulator and
#replicates it, so the requesting peer can connect to the result.
@rpc("any_peer", "call_local", "reliable")
func _request_start(controller_path:NodePath, speak_as:PackedStringArray, listen_to:PackedStringArray) -> void:
	if !multiplayer.is_server():
		return
	var controller := get_node_or_null(controller_path) as PSController
	if controller == null:
		invalid_path_error(controller_path, get_node_or_null(controller_path))
		return
	controller.speak_as = speak_as
	controller.listen_to = listen_to
	
	var remote_authority := get_requester_id()
	if controller.is_active():
		sync_controller_data.rpc(EventType.CONNECT, remote_authority, controller_path, controller.data)
		return
	if !controller.start():
		push_error("Failed to start simulation on controller ", controller_path, "!")
		return
	sync_controller_data.rpc(EventType.CONNECT, remote_authority, controller_path, controller.data)

#sync uses paths because they are able to transmit over a network, unlike actual references
@rpc("authority", "call_local", "reliable")
func sync_controller_data(event_type:EventType, original_authority:int, controller_path: NodePath, data: Dictionary) -> void:
	var node := get_node_or_null(controller_path)
	if !(node is PSController):
		return
	var controller:PSController = node
	controller.data = data
	match event_type:
		EventType.CONNECT:
			if multiplayer.get_unique_id() == original_authority:
				if connected_controller != controller:
					
					connected_controller = controller
					event.emit(EventType.CONNECT, controller)
		EventType.DISCONNECT:
			if multiplayer.get_unique_id() == original_authority:
				if connected_controller == controller:
					connected_controller = null
					event.emit(EventType.DISCONNECT, controller)
		EventType.CHOICE:
			event.emit(EventType.CHOICE, controller)

func get_requester_id() -> int:
	var id := multiplayer.get_remote_sender_id()
	if id == 0:
		id = multiplayer.get_unique_id()
	return id

func invalid_path_error(path:NodePath, result) -> void:
	push_error("Path ", path, " not a valid controller! Found ", result, " instead.")
