class_name MainMenu extends Control

signal host_requested(port: int, dedicated: bool)
signal join_requested(address: String, port: int)

const DEFAULT_ADDRESS := "127.0.0.1"
const DEFAULT_PORT := 7000

@export var address_edit: LineEdit
@export var port_edit: LineEdit
@export var host_button: Button
@export var join_button: Button
@export var dedicated_button: Button
@export var status_label: Label

func _ready() -> void:
	address_edit.text = DEFAULT_ADDRESS
	port_edit.text = str(DEFAULT_PORT)

	host_button.pressed.connect(_on_host_pressed)
	join_button.pressed.connect(_on_join_pressed)
	dedicated_button.pressed.connect(_on_dedicated_pressed)

func set_status(status: String) -> void:
	status_label.text = status

func get_address() -> String:
	return address_edit.text.strip_edges()

func get_port() -> int:
	return port_edit.text.to_int() if port_edit.text.is_valid_int() else DEFAULT_PORT

func _on_host_pressed() -> void:
	host_requested.emit(get_port(), false)

func _on_join_pressed() -> void:
	join_requested.emit(get_address(), get_port())

func _on_dedicated_pressed() -> void:
	host_requested.emit(get_port(), true)
