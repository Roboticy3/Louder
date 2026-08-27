extends Node

var possessions:Dictionary[String, int]
var inverse_possessions:Dictionary[int, String]
var usernames:Dictionary[String, int]

@export var lobby_listener_group := &"lobby"
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
	
	var replicate_on_update:Array[NodePath] = [
		NodePath("..:possessions"),
		NodePath("..:inverse_possessions"),
		NodePath("..:usernames")
	]
	for r in replicate_on_update:
		config.add_property(r)
		config.property_set_replication_mode(r, SceneReplicationConfig.REPLICATION_MODE_ON_CHANGE)

	sync.replication_config = config
	add_child(sync)
	
	add_child(LOBBY.instantiate())

	peer = ENetMultiplayerPeer.new()

func can_possess(character:String) -> bool:
	return !(character in possessions)

func start(character:String, username:String, mode:ClientMode, ip_address:String, port:int) -> bool:
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
	var authority := get_multiplayer_authority()
	
	return possess(character, username, authority)

@rpc("any_peer", "reliable")
func possess(character:String, username:String, authority:int) -> bool:
	if !can_possess(character):
		return false
	possessions[character] = authority
	inverse_possessions[authority] = character
	usernames[username] = authority
	
	#trigger replication
	possessions = possessions
	inverse_possessions = inverse_possessions
	usernames = usernames
	return true

func find_possession(authority:int) -> String:
	return inverse_possessions.get(authority, "")

func find_authority(username:String) -> int:
	return usernames.get(username, -1)

func find_authority_by_possession(possession:String) -> int:
	return possessions.get(possession, -1)
