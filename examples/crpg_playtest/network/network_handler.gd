extends Node

signal lobby_changed
signal connection_failed
signal server_disconnected
signal selection_rejected(result: int)
signal selection_accepted(character: StringName)

const SERVER_ID := 1
const NO_PEER := 0

var _allocator := CharacterAllocator.new()
var _characters: Dictionary = {}
var _allocations: Dictionary = {}
var _endpoint := ""

func _ready() -> void:
	_allocator.changed.connect(_broadcast)

	multiplayer.peer_connected.connect(_on_peer_connected)
	multiplayer.peer_disconnected.connect(_on_peer_disconnected)
	multiplayer.connection_failed.connect(_on_connection_failed)
	multiplayer.server_disconnected.connect(_on_server_disconnected)

	var characters := CharacterCatalog.load_characters()
	for character in characters:
		_characters[character.get_id()] = character
	_allocator.setup(characters)

func host(port: int) -> Error:
	var peer := ENetMultiplayerPeer.new()
	var error := peer.create_server(port, 16)
	if error != OK:
		return error

	multiplayer.multiplayer_peer = peer
	_endpoint = "port %d" % port
	return OK

func join(address: String, port: int) -> Error:
	var peer := ENetMultiplayerPeer.new()
	var error := peer.create_client(address, port)
	if error != OK:
		return error

	multiplayer.multiplayer_peer = peer
	_endpoint = "%s:%d" % [address, port]
	return OK

func leave() -> void:
	multiplayer.multiplayer_peer = null
	_endpoint = ""
	_allocator.reset()

func is_in_lobby() -> bool:
	return multiplayer.has_multiplayer_peer()

func is_host() -> bool:
	return is_in_lobby() and multiplayer.is_server()

func get_endpoint() -> String:
	return _endpoint

func get_local_peer_id() -> int:
	return multiplayer.get_unique_id()

func get_characters() -> Array[CharacterData]:
	return _allocator.get_characters()

func get_character_of(peer_id: int = get_local_peer_id()) -> CharacterData:
	var character_id: StringName = _allocations.get(peer_id, &"")
	if character_id == &"":
		return null
	return _characters.get(character_id) as CharacterData

func get_local_character() -> CharacterData:
	return get_character_of(get_local_peer_id())

func get_peer_of(character_id: StringName) -> int:
	for peer_id in _allocations:
		if _allocations[peer_id] == character_id:
			return peer_id
	return NO_PEER

func get_roster() -> Array[int]:
	var local_peer_id := get_local_peer_id()
	var roster: Array[int] = []
	roster.append(local_peer_id)
	for peer_id in multiplayer.get_peers():
		if peer_id != local_peer_id:
			roster.append(peer_id)
	roster.sort()
	return roster

func request_select(character_id: StringName) -> void:
	if is_host():
		_handle_selection(get_local_peer_id(), character_id)
		return

	_request_selection.rpc_id(SERVER_ID, character_id)

@rpc("any_peer", "call_remote", "reliable")
func _request_selection(character_id: StringName) -> void:
	_handle_selection(multiplayer.get_remote_sender_id(), character_id)

func _handle_selection(peer_id: int, character_id: StringName) -> void:
	var result: CharacterAllocator.Result
	if character_id == &"":
		result = _allocator.deselect(peer_id)
	else:
		result = _allocator.select(peer_id, character_id)

	if result == CharacterAllocator.Result.SELECTED:
		if peer_id == multiplayer.get_unique_id():
			_selection_accepted(character_id)
		else:
			_selection_accepted.rpc_id(peer_id, character_id)
		return

	if peer_id == multiplayer.get_unique_id():
		_selection_rejected(result)
	else:
		_selection_rejected.rpc_id(peer_id, result)

func _broadcast() -> void:
	_apply_allocations(_allocator.to_state())
	if is_host():
		_apply_allocations.rpc(_allocator.to_state())

@rpc("authority", "call_remote", "reliable")
func _apply_allocations(allocations: Dictionary) -> void:
	_allocations = allocations
	lobby_changed.emit()

@rpc("authority", "call_remote", "reliable")
func _selection_rejected(result: int) -> void:
	selection_rejected.emit(result)

@rpc("authority", "call_remote", "reliable")
func _selection_accepted(character: StringName) -> void:
	selection_accepted.emit(character)

func _on_peer_connected(_peer_id: int) -> void:
	if is_host():
		_broadcast()

func _on_peer_disconnected(peer_id: int) -> void:
	if is_host():
		_allocator.release_peer(peer_id)
		_broadcast()

func _on_connection_failed() -> void:
	leave()
	connection_failed.emit()

func _on_server_disconnected() -> void:
	leave()
	server_disconnected.emit()
