class_name CharacterSelect extends Control

signal character_requested(character_id: StringName)
signal leave_requested
signal close_requested

const NO_PEER := 0

@export var title_label: Label
@export var status_label: Label
@export var character_list: VBoxContainer
@export var deselect_button: Button
@export var close_button: Button
@export var leave_button: Button
@export var player_list: VBoxContainer

var _endpoint := ""
var _dedicated := false
var _message := ""
var _players: Dictionary = {}
var _local_peer_id := 1

func _ready() -> void:
	deselect_button.pressed.connect(_on_deselect_pressed)
	close_button.pressed.connect(_on_close_pressed)
	leave_button.pressed.connect(_on_leave_pressed)

func set_lobby(endpoint: String, dedicated: bool) -> void:
	_endpoint = endpoint
	_dedicated = dedicated

func set_status(message: String) -> void:
	_message = message
	_refresh_status()

func refresh(characters: Array[CharacterData], players: Dictionary, roster: Array[int], local_peer_id: int) -> void:
	_message = ""
	_players = players
	_local_peer_id = local_peer_id
	_refresh_title()
	_refresh_characters(characters)
	_refresh_players(roster)
	_refresh_status()

func _refresh_title() -> void:
	title_label.text = "Lobby on %s" % _endpoint

func _refresh_characters(characters: Array[CharacterData]) -> void:
	for child in character_list.get_children():
		character_list.remove_child(child)
		child.queue_free()

	character_list.visible = not _dedicated
	for character in characters:
		var button := Button.new()
		button.text = _character_label(character)
		button.disabled = _is_taken_by_another_player(character)
		button.pressed.connect(_on_character_pressed.bind(character.get_id()))
		character_list.add_child(button)

	deselect_button.visible = not _dedicated
	deselect_button.disabled = !_players.has(_local_peer_id)

func _refresh_players(roster: Array[int]) -> void:
	for child in player_list.get_children():
		player_list.remove_child(child)
		child.queue_free()

	for peer_id in roster:
		var character: CharacterData = _players.get(peer_id)
		var label := Label.new()
		label.text = "player %d: %s" % [peer_id, character.name if character else "no character"]
		player_list.add_child(label)

func _refresh_status() -> void:
	close_button.visible = !_dedicated and _players.has(_local_peer_id)

	if !_message.is_empty():
		status_label.text = _message
		return

	if _dedicated:
		status_label.text = "This server does not select a character."
		return

	var character: CharacterData = _players.get(_local_peer_id)
	if character == null:
		status_label.text = "Pick a character to start playing."
	else:
		status_label.text = "You are %s." % character.name

func _character_label(character: CharacterData) -> String:
	var label := character.name
	if character.is_debug:
		label += " (debug actor)"

	var peer_id := _get_holder(character)
	if peer_id == NO_PEER:
		return label
	if peer_id == _local_peer_id:
		return "%s [you]" % label
	return "%s [player %d]" % [label, peer_id]

func _is_taken_by_another_player(character: CharacterData) -> bool:
	var peer_id := _get_holder(character)
	return peer_id != NO_PEER and peer_id != _local_peer_id

func _get_holder(character: CharacterData) -> int:
	for peer_id in _players:
		if _players[peer_id] == character:
			return peer_id
	return NO_PEER

func _on_character_pressed(character_id: StringName) -> void:
	character_requested.emit(character_id)

func _on_deselect_pressed() -> void:
	character_requested.emit(&"")

func _on_close_pressed() -> void:
	close_requested.emit()

func _on_leave_pressed() -> void:
	leave_requested.emit()