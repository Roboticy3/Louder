extends Control

@export var feed: Container
@export var decay: Timer

@export var time_scale := 5.0

@export var mute_server_created: bool
@export var mute_connected_to_server: bool
@export var mute_server_disconnected: bool
@export var mute_peer_connected: bool
@export var mute_peer_disconnected: bool
@export var mute_create_server_error: bool
@export var mute_create_client_error: bool
@export var mute_client_mode_error: bool
@export var mute_data_updated: bool = true
@export var mute_data_error: bool = true
@export var mute_character_selected: bool
@export var mute_controller_event: bool

func _ready() -> void:
	decay.timeout.connect(_on_decay_timeout)

	var mp = multiplayer
	mp.connected_to_server.connect(_on_connected_to_server)
	mp.server_disconnected.connect(_on_server_disconnected)
	mp.peer_connected.connect(_on_peer_connected)
	mp.peer_disconnected.connect(_on_peer_disconnected)

	NetworkLayer.server_created.connect(_on_server_created)
	NetworkLayer.create_server_error.connect(_on_create_server_error)
	NetworkLayer.create_client_error.connect(_on_create_client_error)
	NetworkLayer.client_mode_error.connect(_on_client_mode_error)

	ClientDataLayer.data_updated.connect(_on_data_updated)
	ClientDataLayer.data_error.connect(_on_data_error)
	ClientDataLayer.character_selected.connect(_on_character_selected)

	ControllerEventLayer.choice_event.connect(_on_controller_choice)
	ControllerEventLayer.connect_event.connect(_on_controller_connect)
	ControllerEventLayer.disconnect_event.connect(_on_controller_disconnect)

func _on_decay_timeout() -> void:
	if feed.get_child_count() > 0:
		decay.wait_time = time_scale / (log(feed.get_child_count()) / log(2) + 1)
		feed.get_child(0).queue_free()
		decay.start()

func _add_message(text: String) -> void:
	var label = Label.new()
	label.text = text
	feed.add_child(label)
	decay.start()

func _on_server_created() -> void:
	if mute_server_created: return
	_add_message("Server created")

func _on_connected_to_server() -> void:
	if mute_connected_to_server: return
	_add_message("Connected to server")

func _on_server_disconnected() -> void:
	if mute_server_disconnected: return
	_add_message("Disconnected from server")

func _on_peer_connected(id: int) -> void:
	if mute_peer_connected: return
	_add_message("Peer connected: %d" % id)

func _on_peer_disconnected(id: int) -> void:
	if mute_peer_disconnected: return
	_add_message("Peer disconnected: %d" % id)

func _on_create_server_error(error: Error) -> void:
	if mute_create_server_error: return
	_add_message("Failed to create server: %s" % error_string(error))

func _on_create_client_error(error: Error) -> void:
	if mute_create_client_error: return
	_add_message("Failed to connect: %s" % error_string(error))

func _on_client_mode_error(error: Error) -> void:
	if mute_client_mode_error: return
	_add_message("Invalid client mode: %s" % error_string(error))

func _on_data_updated(peer_id: int, data: Dictionary) -> void:
	if mute_data_updated: return
	var message := "Data updated for peer %d:"
	var values := [peer_id]
	for key in data:
		message += "\n  %s: %s"
		values.append_array([ClientDataLayer.ClientProperties.find_key(key), data[key]])
	
	_add_message(message % values)

func _on_data_error(error: ClientDataLayer.DataError) -> void:
	if mute_data_error: return
	_add_message("Data error: %s" % ClientDataLayer.DataError.find_key(error))

func _on_character_selected(peer_id: int, character: String) -> void:
	if mute_character_selected: return
	_add_message("Peer %d selected character: %s" % [peer_id, character])

func _on_controller_choice(controller: PSController) -> void:
	if mute_controller_event: return
	_add_message("Controller %s, %s made a choice" % [controller.speak_as, controller.get_parent().name] )

func _on_controller_connect(controller: PSController) -> void:
	if mute_controller_event: return
	_add_message("Controller %s, %s connected" % [controller.speak_as, controller.get_parent().name])

func _on_controller_disconnect(controller: PSController) -> void:
	if mute_controller_event: return
	_add_message("Controller %s, %s disconnected" % [controller.speak_as, controller.get_parent().name])
