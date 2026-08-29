extends Node

@export var client_data_listener_group := &"client_data_listener"
@export var server_listener_group := &"server_listener"

var client_data:Dictionary[int, Dictionary] = {}

const LOBBY := preload("res://demos/september/lobby_menu.tscn")
const STATUS := preload()

var peer:ENetMultiplayerPeer

enum ClientMode {
	NONE = 0,
	JOIN,
	HOST
}

func _ready() -> void:
	var sync := MultiplayerSynchronizer.new()
	var config := SceneReplicationConfig.new()
	
	var prop := NodePath(":client_data") #MultiplayerSynchronizer defaults to reading properties off the parent
	config.add_property(prop)
	config.property_set_replication_mode(prop, SceneReplicationConfig.REPLICATION_MODE_ON_CHANGE)
	
	sync.replication_config = config
	add_child(sync)
	
	add_child(LOBBY.instantiate())

	peer = ENetMultiplayerPeer.new()
	
func start(mode:ClientMode, ip_address:String, port:int, data:Dictionary) -> bool:
	
	#peer is initialized (and checked for errors)
	var error := OK
	match mode:
		ClientMode.JOIN:
			error = peer.create_client(ip_address, port)
			#assign authority based on the client?
		ClientMode.HOST:
			error = peer.create_server(port)
	if error != OK:
		push_error("NetworkLayer::start: ", error_string(error))
		return false
	
	#peer is fully established. Do actual multiplayer stuff after this point
	multiplayer.multiplayer_peer = peer
	
	#peer data is connected
	#Merge this peer's own data under its unique id on the server instead of letting
	#the MultiplayerSynchronizer overwrite a joining client's data with the host's
	#snapshot. Each peer keeps its own entry; the server aggregates them all.
	var peer_id := multiplayer.get_unique_id()
	if mode == ClientMode.JOIN:
		await multiplayer.connected_to_server
		_merge_client_data.rpc_id(1, peer_id, data)
	else:
		client_data[peer_id] = data

	if mode == ClientMode.HOST:
		if error == OK: get_tree().call_group(server_listener_group, &"_on_server_created", port)
	
	return true

@rpc("any_peer", "call_local", "reliable")
func _merge_client_data(authority:int, merged_data:Dictionary) -> void:
	if !multiplayer.is_server():
		return
	client_data[authority] = merged_data
	noitfy_client_data_listeners()

func noitfy_client_data_listeners():
	get_tree().call_group(client_data_listener_group, &"_on_client_data_changed", client_data)
