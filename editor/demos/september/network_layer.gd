extends Node

@export var client_data_listener_group := &"client_data_listener"
@export var server_listener_group := &"server_listener"

var client_data:Dictionary[int, Dictionary]

const LOBBY := preload("res://demos/september/lobby_menu.tscn")

var peer:ENetMultiplayerPeer

enum ClientMode {
	NONE = 0,
	JOIN,
	HOST
}

func _ready() -> void:
	var sync := MultiplayerSynchronizer.new()
	var config := SceneReplicationConfig.new()
	
	var prop := ^":client_data" #MultiplayerSynchronizer defaults to reading properties off the parent
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
	#Problem here: joining client's data is overwritten by host data. Desired effect is one of merging.
	#Cause: MultiplayerSynchronizer constructed in _ready is syncing the host's data OVER the client's.
	#This makes sense normally. Maybe more advanced rpc behavior is required?
	#client_data = client_data.merged({
	#	get_multiplayer_authority() : data
	#})
	
	if mode == ClientMode.HOST:
		if error == OK: get_tree().call_group(server_listener_group, &"_on_server_created", port)
	
	return true

func submit_client_data():
	get_tree().call_group(client_data_listener_group, &"_on_client_data_changed", client_data)
