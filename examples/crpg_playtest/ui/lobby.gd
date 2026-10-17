class_name Lobby extends CanvasLayer

const NO_PEER := 0

@export var avatar: CharacterAvatar
@export var controller: CharacterAvatarControllerSingle

@onready var root: Control = $Root
@onready var background: ColorRect = $Root/Background
@onready var menu: VBoxContainer = $Root/Center/Panel/Rows/Menu
@onready var address_edit: LineEdit = $Root/Center/Panel/Rows/Menu/AddressEdit
@onready var port_edit: LineEdit = $Root/Center/Panel/Rows/Menu/PortEdit
@onready var host_button: Button = $Root/Center/Panel/Rows/Menu/Buttons/Host
@onready var join_button: Button = $Root/Center/Panel/Rows/Menu/Buttons/Join
@onready var dedicated_button: Button = $Root/Center/Panel/Rows/Menu/Buttons/Dedicated
@onready var select: VBoxContainer = $Root/Center/Panel/Rows/Select
@onready var character_list: VBoxContainer = $Root/Center/Panel/Rows/Select/CharacterList
@onready var deselect_button: Button = $Root/Center/Panel/Rows/Select/Deselect
@onready var players: VBoxContainer = $Root/Center/Panel/Rows/Players
@onready var player_list: VBoxContainer = $Root/Center/Panel/Rows/Players/PlayerList
@onready var leave_button: Button = $Root/Center/Panel/Rows/Players/Leave
@onready var status_label: Label = $Root/Center/Panel/Rows/Status
@onready var open_button: Button = $Open

var _dedicated := false

func _ready() -> void:
	host_button.pressed.connect(_on_host_pressed)
	join_button.pressed.connect(_on_join_pressed)
	dedicated_button.pressed.connect(_on_dedicated_pressed)
	deselect_button.pressed.connect(_on_deselect_pressed)
	leave_button.pressed.connect(_on_leave_pressed)
	open_button.pressed.connect(_on_open_pressed)

	Network.lobby_changed.connect(_on_lobby_changed)
	Network.selection_rejected.connect(_on_selection_rejected)
	Network.connection_failed.connect(_on_connection_failed)
	Network.server_disconnected.connect(_on_server_disconnected)

	_show_menu("")

func _on_host_pressed() -> void:
	_start(Network.host(_get_port()), false)

func _on_join_pressed() -> void:
	_start(Network.join(address_edit.text.strip_edges(), _get_port()), false)

func _on_dedicated_pressed() -> void:
	_start(Network.host(_get_port()), true)

func _start(error: Error, dedicated: bool) -> void:
	if error != OK:
		_show_menu("Could not start the lobby: %s" % error_string(error))
		return

	_dedicated = dedicated
	_show_lobby()

func _on_character_pressed(character: CharacterData) -> void:
	Network.request_select(character.get_id())

func _on_deselect_pressed() -> void:
	Network.request_select(&"")

func _on_leave_pressed() -> void:
	Network.leave()
	_show_menu("")

func _on_open_pressed() -> void:
	_show_lobby()

func _on_lobby_changed() -> void:
	if !Network.is_in_lobby():
		return

	var character := Network.get_local_character()
	if character == null:
		_show_lobby()
	else:
		_hide_lobby()
	_play_as(character)

func _on_selection_rejected(result: int) -> void:
	_set_status("That character is taken by another player." if result == CharacterAllocator.Result.TAKEN else "Unknown character.")

func _on_connection_failed() -> void:
	_show_menu("Could not reach the server.")

func _on_server_disconnected() -> void:
	_show_menu("The server closed the lobby.")

func _show_menu(status: String) -> void:
	_dedicated = false
	root.visible = true
	menu.visible = true
	select.visible = false
	players.visible = false
	open_button.visible = false
	_refresh()
	_set_status(status)

func _show_lobby() -> void:
	root.visible = true
	background.visible = true
	menu.visible = false
	select.visible = not _dedicated
	players.visible = true
	open_button.visible = Network.get_local_character() != null
	_refresh()

func _hide_lobby() -> void:
	root.visible = false
	open_button.visible = true

func _refresh() -> void:
	_refresh_characters()
	_refresh_players()
	_refresh_status()

func _refresh_characters() -> void:
	for child in character_list.get_children():
		character_list.remove_child(child)
		child.queue_free()

	for character in Network.get_characters():
		var button := Button.new()
		button.text = _character_label(character)
		button.disabled = _is_taken_by_another_player(character)
		button.pressed.connect(_on_character_pressed.bind(character))
		character_list.add_child(button)

	deselect_button.disabled = Network.get_local_character() == null

func _refresh_players() -> void:
	for child in player_list.get_children():
		player_list.remove_child(child)
		child.queue_free()

	for peer_id in Network.get_roster():
		var character := Network.get_character_of(peer_id)
		var label := Label.new()
		label.text = "player %d: %s" % [peer_id, character.name if character else "no character"]
		player_list.add_child(label)

func _refresh_status() -> void:
	if _dedicated:
		_set_status("This server does not select a character.")
		return

	var character := Network.get_local_character()
	if character == null:
		_set_status("Pick a character to start playing.")
	else:
		_set_status("You are %s." % character.name)

func _character_label(character: CharacterData) -> String:
	var label := character.name
	if character.is_debug:
		label += " (debug actor)"

	var peer_id := Network.get_peer_of(character.get_id())
	if peer_id == NO_PEER:
		return label
	if peer_id == Network.get_local_peer_id():
		return "%s [you]" % label
	return "%s [player %d]" % [label, peer_id]

func _is_taken_by_another_player(character: CharacterData) -> bool:
	var peer_id := Network.get_peer_of(character.get_id())
	return peer_id != NO_PEER and peer_id != Network.get_local_peer_id()

func _play_as(character: CharacterData) -> void:
	avatar.character_data = character
	controller.character_data = character
	if character != null:
		controller.locate_character()

func _set_status(status: String) -> void:
	status_label.text = status

func _get_port() -> int:
	return port_edit.text.to_int() if port_edit.text.is_valid_int() else 7000