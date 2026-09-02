extends Node

var peer:ENetMultiplayerPeer

enum ClientMode {
	HOST = 0,
	CLIENT,
	NONE
}

signal client_mode_error(error:Error)
signal create_server_error(error:Error)
signal create_client_error(error:Error)

func start(mode:ClientMode, ip_address:String, port:int):
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
		ClientMode.CLIENT:
			peer = ENetMultiplayerPeer.new()
			var error := peer.create_client(ip_address, port)
			if error != OK:
				create_client_error.emit(error)
				return
