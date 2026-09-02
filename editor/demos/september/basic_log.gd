extends Control

@export var feed: Container
@export var decay: Timer

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

func _on_decay_timeout() -> void:
	for child in feed.get_children():
		child.queue_free()

func _add_message(text: String) -> void:
	var label = Label.new()
	label.text = text
	feed.add_child(label)
	decay.start()

func _on_server_created() -> void:
	_add_message("Server created")

func _on_connected_to_server() -> void:
	_add_message("Connected to server")

func _on_server_disconnected() -> void:
	_add_message("Disconnected from server")

func _on_peer_connected(id: int) -> void:
	_add_message("Peer connected: %d" % id)

func _on_peer_disconnected(id: int) -> void:
	_add_message("Peer disconnected: %d" % id)

func _on_create_server_error(error: Error) -> void:
	_add_message("Failed to create server: %s" % error_string(error))

func _on_create_client_error(error: Error) -> void:
	_add_message("Failed to connect: %s" % error_string(error))

func _on_client_mode_error(error: Error) -> void:
	_add_message("Invalid client mode: %s" % error_string(error))
