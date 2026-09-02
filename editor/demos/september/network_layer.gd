extends Node

var peer:ENetMultiplayerPeer

enum ClientMode {
	NONE = 0,
	CLIENT,
	HOST
}

signal client_mode_error(error:Error)
signal create_server_error(error:Error)
signal create_client_error(error:Error)
signal peer_already_configured_error
signal server_created

const LOG := preload("res://demos/september/basic_log.tscn")
func _ready() -> void:
	add_child(LOG.instantiate())

func start(mode:ClientMode, ip_address:String, port:int):
	if peer:
		peer_already_configured_error.emit()
		return
	
	match mode:
		ClientMode.NONE:
			client_mode_error.emit(Error.ERR_INVALID_PARAMETER)
			return
		ClientMode.HOST:
			peer = ENetMultiplayerPeer.new()
			var error := peer.create_server(port)
			if error != OK:
				create_server_error.emit(error)
				return
			multiplayer.multiplayer_peer = peer
			server_created.emit()
		ClientMode.CLIENT:
			peer = ENetMultiplayerPeer.new()
			var error := peer.create_client(ip_address, port)
			if error != OK:
				create_client_error.emit(error)
				return
			multiplayer.multiplayer_peer = peer
