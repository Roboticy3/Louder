extends Node

enum StarterMode {
	NONE = 0,
	JOIN,
	HOST
}
var mode := StarterMode.NONE

var play_as := ""
var username := ""

@export var progress_message_listener_group := &"progress_message_listener"
@export var progress_status_listener_group := &"progress_status_listener"

func _ready():
	validate()

func set_mode(to:StarterMode):
	mode = to
	validate()

func set_play_as(to:String):
	play_as = to
	if mode == StarterMode.NONE:
		mode = StarterMode.JOIN
	validate()

func set_username(to:String):
	username = to
	if mode == StarterMode.NONE:
		mode = StarterMode.JOIN
	validate()

func validate():
	get_tree().call_group(progress_status_listener_group, &"set", &"disabled", get_progress_disabled())
	get_tree().call_group(progress_message_listener_group, &"set", &"text", get_progress_text())

func get_progress_disabled() -> bool:
	if mode == StarterMode.NONE:
		return true
	if mode == StarterMode.JOIN:
		if MerchantNetworkHandler.server_ip == "":
			return true
	if play_as.is_empty():
		return true
	if username.is_empty():
		return true
	return false

func set_server_ip(to:String) -> void:
	MerchantNetworkHandler.server_ip = to

func set_server_port(to:int) -> void:
	MerchantNetworkHandler.server_port = to

func get_progress_text() -> String:
	var part1 := ""
	match mode:
		StarterMode.NONE:
			part1 = "Awaiting command"
		StarterMode.JOIN:
			part1 = "Press to join game"
		StarterMode.HOST:
			part1 = "Press to host game"
	
	var part2 := ""
	if !play_as.is_empty():
		part2 = " as %s" % [play_as]
	
	var full := part1 + part2 + "..."
	return full

func start():
	pass
