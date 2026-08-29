extends Node

@export var lobby_listener_group := &"lobby"
@export var server_listener_group := &"server_listener"
const LOBBY := preload("res://demos/september/lobby_menu.tscn")

var peer:ENetMultiplayerPeer

enum ClientMode {
	NONE = 0,
	JOIN,
	HOST
}

func _ready() -> void:
	add_child(LOBBY.instantiate())

	peer = ENetMultiplayerPeer.new()

func start(mode:ClientMode, ip_address:String, port:int, data:Dictionary) -> bool:
	
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
	
	multiplayer.multiplayer_peer = peer
	if mode == ClientMode.HOST:
		if error == OK: get_tree().call_group(server_listener_group, &"_on_server_created", port)
	var authority := get_multiplayer_authority()
	return true
