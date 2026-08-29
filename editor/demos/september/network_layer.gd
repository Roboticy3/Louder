extends Node

@export var client_data_listener_group := &"client_data_listener"
@export var server_listener_group := &"server_listener"

var client_data:Dictionary[int, Dictionary] = {}

const LOBBY := preload("res://demos/september/lobby_menu.tscn")
const STATUS := preload("res://demos/september/network_layer_status.tscn")

var status_ui:CanvasLayer

var peer:ENetMultiplayerPeer

enum ClientMode {
	NONE = 0,
	Client,
	Host
}

func _ready() -> void:
	var sync := MultiplayerSynchronizer.new()
	var config := SceneReplicationConfig.new()
	
	var prop := NodePath(":client_data") #MultiplayerSynchronizer defaults to reading properties off the parent
	config.add_property(prop)
	config.property_set_replication_mode(prop, SceneReplicationConfig.REPLICATION_MODE_ALWAYS)
	
	sync.replication_config = config
	sync.delta_interval = 1.0
	add_child(sync)
	
	var timer := Timer.new()
	timer.wait_time = 1.0
	timer.autostart = true
	timer.one_shot = false
	timer.timeout.connect(noitfy_client_data_listeners)
	add_child(timer)
	
	add_child(LOBBY.instantiate())

	status_ui = STATUS.instantiate()
	multiplayer.connected_to_server.connect(func():
		status_ui.post("Connected to server."))
	multiplayer.connection_failed.connect(func():
		status_ui.post("Connection to server failed."))
	multiplayer.server_disconnected.connect(func():
		status_ui.post("Server disconnected."))
	multiplayer.peer_disconnected.connect(func(id:int):
		var username:String = client_data.get(id, {}).get("username", "")
		if username.is_empty():
			status_ui.post("Peer %d disconnected." % id)
		else:
			status_ui.post("%s disconnected." % username))
	add_child(status_ui)

	peer = ENetMultiplayerPeer.new()
	
func start(mode:ClientMode, ip_address:String, port:int, data:Dictionary) -> bool:
	
	#peer is initialized (and checked for errors)
	var error := OK
	match mode:
		ClientMode.Client:
			error = peer.create_client(ip_address, port)
			#assign authority based on the client?
		ClientMode.Host:
			error = peer.create_server(port)
	if error != OK:
		push_error("NetworkLayer::start: ", error_string(error))
		return false
	
	#peer is fully established. Do actual multiplayer stuff after this point
	multiplayer.multiplayer_peer = peer
	
	#wait for successful connection, then sync data across the client and server
	#once the server has the data, MultiplayerSynchronizer will dispatch the
	#	updated data to all connected clients
	var peer_id := multiplayer.get_unique_id()
	if mode == ClientMode.Client:
		status_ui.post("Connecting to server %s:%d..." % [ip_address, port])
		await multiplayer.connected_to_server
		_merge_client_data.rpc_id(1, peer_id, data)
	else:
		status_ui.post("Hosting server on port %d." % port)
		client_data[peer_id] = data
		

	if mode == ClientMode.Host:
		if error == OK: get_tree().call_group(server_listener_group, &"_on_server_created", port)
	
	return true

@rpc("any_peer", "call_local", "reliable")
func _merge_client_data(authority:int, data:Dictionary) -> void:
	if !multiplayer.is_server():
		return
	
	var client_data_dup := client_data.duplicate()
	client_data_dup[authority] = data
	client_data = client_data_dup
	
	var username:String = data.get("username", "")
	if username.is_empty():
		status_ui.post("Peer %d joined." % authority)
	else:
		status_ui.post("%s joined." % username)

func noitfy_client_data_listeners():
	get_tree().call_group(client_data_listener_group, &"_on_client_data_changed", client_data)
