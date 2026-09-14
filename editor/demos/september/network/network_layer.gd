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
signal upnp_mapping_failed_tcp(error:int)
signal upnp_mapping_failed_udp(error:int)
signal upnp_mapping_established(external_ip:String, port:int)
signal host_details_changed()

const LOG := preload("res://demos/september/network/basic_log.tscn")
const LOBBY := preload("res://demos/september/network/lobby_menu.tscn")

func _ready() -> void:
	add_child(LOBBY.instantiate())
	add_child(LOG.instantiate())
	
	var timer := Timer.new()
	timer.autostart = true
	timer.one_shot = false
	timer.timeout.connect(_poll_upnp_thread)
	add_child(timer)

var host_ip := ""
var host_port := -1
func start(mode:ClientMode, ip_address:String, port:int, use_upnp:bool):
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
				push_error("Failed to create server on port ", port)
				create_server_error.emit(error)
				return
			multiplayer.multiplayer_peer = peer
			server_created.emit()
			
			upnp_mapping_established.connect(_check_upnp_against_host.bind(port))
			if use_upnp:
				port_forward(port)
			
			if host_ip.is_empty():
				host_ip = ip_address
			host_port = port
			host_details_changed.emit()
			
		ClientMode.CLIENT:
			peer = ENetMultiplayerPeer.new()
			var error := peer.create_client(ip_address, port)
			if error != OK:
				push_error("Failed to connect to server ", ip_address, ":", port)
				create_client_error.emit(error)
				return
			multiplayer.multiplayer_peer = peer
			
			host_ip = ip_address
			host_port = port
			host_details_changed.emit()

func _check_upnp_against_host(external_ip_address:String, mapped_port:int, port:int):
	if mapped_port == port:
		host_ip = external_ip_address
		host_details_changed.emit()

func get_host_ip() -> String:
	return host_ip

func get_host_port() -> int:
	return host_port

const UPNP_MAPPING_DESCRIPTION := "east_breach"
var upnp:UPNP
var upnp_mapping_thread:Thread
var external_ip := ""
var port_forwarded := -1
func port_forward(port:int) -> void:
	_reset_upnp_thread()
	_deploy_upnp_thread(port)

func _poll_upnp_thread() -> void:
	if upnp_mapping_thread and upnp_mapping_thread.is_started() and !upnp_mapping_thread.is_alive():
		var passed = (upnp_mapping_thread.wait_to_finish())
		if passed is Dictionary:
			if "external_ip" in passed and "port" in passed:
				external_ip = passed["external_ip"]
				port_forwarded = passed["port"]
				upnp_mapping_established.emit(passed["external_ip"], passed["port"])
				return
			if "error" in passed:
				var port = passed.get("port", "?")
				var code = passed.get("code", "?")
				if passed["error"] == "tcp":
					push_error("Unable to establish UPNP port mapping: TCP ", port, "->", port, " with or without description. Returned code ", code)
					upnp_mapping_failed_tcp.emit(code)
				if passed["error"] == "udp":
					push_error("Unable to establish UPNP port mapping: UDP ", port, "->", port, " with or without description. Returned code ", code)
					upnp_mapping_failed_udp.emit(code)

func _deploy_upnp_thread(port_to_forward:int):
	upnp_mapping_thread = Thread.new()
	upnp_mapping_thread.start(_port_forward_synchronous.bind(port_to_forward))

func _reset_upnp_thread():
	if upnp_mapping_thread and upnp_mapping_thread.is_started() and upnp_mapping_thread.is_alive(): 
		print("closing port mapping thread...")
		upnp_mapping_thread.wait_to_finish()
		print("done")

func _port_forward_synchronous(port:int) -> Dictionary:
	upnp = UPNP.new()
	var discover_result := upnp.discover()
	
	if discover_result == UPNP.UPNP_RESULT_SUCCESS:
		#port gateway exists and is valid for port forwarding
		if upnp.get_gateway() and upnp.get_gateway().is_valid_gateway():
			
			var map_result_udp := upnp.add_port_mapping(port,port,UPNP_MAPPING_DESCRIPTION, "UDP")
			var map_result_tcp := upnp.add_port_mapping(port,port,UPNP_MAPPING_DESCRIPTION, "TCP")
			
			if not map_result_udp == UPNP.UPNP_RESULT_SUCCESS:
				map_result_udp = upnp.add_port_mapping(port, port,"", "UDP")
			
			if not map_result_tcp == UPNP.UPNP_RESULT_SUCCESS:
				map_result_tcp = upnp.add_port_mapping(port, port,"", "TCP")
			
			if not map_result_udp == UPNP.UPNP_RESULT_SUCCESS:
				return {"error": "udp", "port": port, "code": map_result_udp}
			
			if not map_result_tcp == UPNP.UPNP_RESULT_SUCCESS:
				return {"error": "tcp", "port": port, "code": map_result_tcp}
	
	return {
		"external_ip": upnp.query_external_address(),
		"port": port
	}

func _exit_tree() -> void:
	disconnect_peer()

func disconnect_peer() -> void:
	peer = null
	multiplayer.multiplayer_peer = null
	host_ip = ""
	host_port = -1
	
	if upnp and port_forwarded != -1:
		print("closing port mappings...")
		upnp.delete_port_mapping(port_forwarded, "UDP")
		upnp.delete_port_mapping(port_forwarded, "TCP")
		print("done")
	_reset_upnp_thread()
	external_ip = ""
	port_forwarded = -1
