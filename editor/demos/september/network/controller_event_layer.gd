extends Node

#interface with controllers through this script to make PSController's traverse
#and follow methods into a server-authoritative choice interface.

enum EventType {
	CHOICE = 0,
	CONNECT,
	DISCONNECT,
	CATCH_UP,
}

signal choice_event(controller:PSController)
signal connect_event(controller:PSController)
signal disconnect_event(controller:PSController)
signal catch_up_event(controller:PSController)

#eventually want to replace this with more intentional selection of simulators based on user input in the world
@export var simulator_group := &"ps_simulator"
@export var controller_group := &"ps_controller"

var connected_controller:PSController

func get_connected_controller() -> PSController:
	return connected_controller

# --- connecting / disconnecting ------------------------------------------
#
# Connecting and disconnecting are local concepts: they manage which
# controller THIS peer's UI drives (`connected_controller`). The only things
# a connection can do that need the server are *starting* a brand new
# conversation and *joining* an active one, because both decide the
# controller's state where the authoritative simulator lives. Failing to find
# an active conversation works entirely on state every peer already has, so
# that case never touches the network.

func request_controller(controller:PSController) -> void:
	if !is_instance_valid(controller):
		return
	if !multiplayer.has_multiplayer_peer():
		return
	if controller.is_active():
		connect_local(controller)
		return
	
	_request_start.rpc_id(1, controller.get_path(), controller.speak_as, controller.listen_to)
	if !multiplayer.is_server():
		_request_catch_up.rpc_id(1, controller.get_path())

func connect_local(controller:PSController) -> void:
	if connected_controller == controller:
		return
	
	connected_controller = controller
	connect_event.emit(controller)

func disconnect_controller(controller:PSController) -> void:
	if connected_controller == controller:
		connected_controller = null
		disconnect_event.emit(controller)

# --- choices -------------------------------------------------------------

func request_choice(controller:PSController, choice: String) -> void:
	if !is_instance_valid(controller):
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
	
	#var followers:Array[PSController]
	var nonfollowers:Array[PSController]
	for c in get_tree().get_nodes_in_group(controller_group):
		if c is PSController and c != controller and c.is_active():
			if c.should_follow(controller):
				#followers.push_back(c)
				pass
			else:
				nonfollowers.push_back(c)
	
	if !controller.traverse(choice):
		push_warning("Choice ", choice, " failed for controller ", controller_path, "!")
		return
	
	if controller.branch_changed(): for nf in nonfollowers:
		if controller.should_follow(nf):
			controller.follow(nf)
			break
	
	var remote_authority := get_requester_id()
	
	#precomputed followers doesn't account for unforseen details about the new dialogue state which can affect visibility
	#for f in followers:
		#f.follow(controller)
		#sync_controller_data.rpc(EventType.CHOICE, remote_authority, f.get_path(), f.data)
	
	for c in get_tree().get_nodes_in_group(controller_group):
		if c is PSController and c != controller and c.is_active() and c.should_follow(controller):
			c.follow(controller)
			sync_controller_data.rpc(EventType.CHOICE, remote_authority, c.get_path(), c.data)
	
	sync_controller_data.rpc(EventType.CHOICE, remote_authority, controller_path, controller.data)

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
	
	for c in get_tree().get_nodes_in_group(controller_group):
		if c is PSController and c.is_active() and controller.should_follow(c):
			controller.data = c.data
			break
	
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
					connect_event.emit(controller)
		EventType.DISCONNECT:
			if multiplayer.get_unique_id() == original_authority:
				if connected_controller == controller:
					connected_controller = null
					disconnect_event.emit(controller)
		EventType.CHOICE:
			choice_event.emit(controller)

#late-to-join peers might have missed out on updates
func request_catch_up(controller: PSController):
	_request_catch_up.rpc_id(1, controller.get_path())

@rpc("any_peer", "call_remote", "reliable")
func _request_catch_up(exclude_path:NodePath):
	var remote_authority := get_requester_id()
	for c in get_tree().get_nodes_in_group(controller_group):
		if c is PSController:
			var c_path := c.get_path()
			if c_path == exclude_path:
				continue
			
			_sync_catch_up.rpc_id(remote_authority, c_path, c.data, c.speak_as, c.listen_to)
	

@rpc("authority", "call_remote", "reliable")
func _sync_catch_up(controller_path:NodePath, data:Dictionary, speak_as:Array, listen_to:Array) -> void:
	var node := get_node_or_null(controller_path)
	if !(node is PSController):
		return
	var controller:PSController = node
	controller.data = data
	controller.speak_as = speak_as
	controller.listen_to = listen_to
	catch_up_event.emit(controller)

func get_requester_id() -> int:
	var id := multiplayer.get_remote_sender_id()
	if id == 0:
		id = multiplayer.get_unique_id()
	return id

func invalid_path_error(path:NodePath, result) -> void:
	push_error("Path ", path, " not a valid controller! Found ", result, " instead.")
