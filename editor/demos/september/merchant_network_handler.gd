extends Node

var server_ip := "localhost"
var server_port := 45678

var peer:ENetMultiplayerPeer

func start_server() -> void:
	peer = ENetMultiplayerPeer.new()
	peer.create_server(server_port)
	multiplayer.multiplayer_peer = peer

func create_client() -> void:
	peer = ENetMultiplayerPeer.new()
	peer.create_client(server_ip, server_port)
	multiplayer.multiplayer_peer = peer
