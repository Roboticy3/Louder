class_name CrpgGame extends Node

const REJECTION_MESSAGES := {
	CharacterAllocator.Result.UNKNOWN_CHARACTER: "That character does not exist.",
	CharacterAllocator.Result.TAKEN: "Another player already has that character.",
}

@export var world: CrpgWorld
@export var main_menu: MainMenu
@export var character_select: CharacterSelect

var _dedicated := false

func _ready() -> void:
	main_menu.host_requested.connect(_on_host_requested)
	main_menu.join_requested.connect(_on_join_requested)
	character_select.character_requested.connect(_on_character_requested)
	character_select.leave_requested.connect(_on_leave_requested)
	character_select.close_requested.connect(_on_close_requested)

	Network.lobby_changed.connect(_on_lobby_changed)
	Network.selection_rejected.connect(_on_selection_rejected)
	Network.connection_failed.connect(_on_connection_failed)
	Network.server_disconnected.connect(_on_server_disconnected)

	LouderContextGlobal.load_conversation_config_file("configs/config.json")

	_show_main_menu("")

func _on_host_requested(port: int, dedicated: bool) -> void:
	var error := Network.host(port)
	if error != OK:
		_show_main_menu("Could not start the lobby: %s" % error_string(error))
		return

	_dedicated = dedicated
	_show_character_select()

func _on_join_requested(address: String, port: int) -> void:
	var error := Network.join(address, port)
	if error != OK:
		_show_main_menu("Could not start the lobby: %s" % error_string(error))
		return

	_dedicated = false
	_show_character_select()

func _on_character_requested(character_id: StringName) -> void:
	Network.request_select(character_id)

func _on_leave_requested() -> void:
	Network.leave()
	_show_main_menu("")

func _on_close_requested() -> void:
	_refresh_lobby()

func _on_lobby_changed() -> void:
	if Network.is_in_lobby():
		_refresh_lobby()

func _on_selection_rejected(result: int) -> void:
	character_select.set_status(str(REJECTION_MESSAGES.get(result, "That character could not be selected.")))

func _on_connection_failed() -> void:
	_show_main_menu("Could not reach the server.")

func _on_server_disconnected() -> void:
	_show_main_menu("The server closed the lobby.")

func _show_main_menu(status: String) -> void:
	_dedicated = false
	main_menu.visible = true
	character_select.visible = false
	main_menu.set_status(status)
	_refresh_world()

func _show_character_select() -> void:
	main_menu.visible = false
	character_select.visible = true
	_refresh_lobby()

func _refresh_lobby() -> void:
	var players := _get_players()
	character_select.set_lobby(Network.get_endpoint(), _dedicated)
	character_select.refresh(Network.get_characters(), players, Network.get_roster(), Network.get_local_peer_id())
	character_select.visible = not players.has(Network.get_local_peer_id())
	_refresh_world()

func _refresh_world() -> void:
	world.set_players(_get_players())
	world.play_as(Network.get_local_character())

func _get_players() -> Dictionary:
	var players: Dictionary = {}
	for peer_id in Network.get_roster():
		var character := Network.get_character_of(peer_id)
		if character:
			players[peer_id] = character
	return players
