extends Node


var mode := NetworkLayer.ClientMode.NONE

var play_as := ""
var username := ""
var ip_address := "localhost"
var port := 45678

@export var progress_message_listener_group := &"progress_message_listener"
@export var progress_status_listener_group := &"progress_status_listener"
@export var start_listener_group := &"start_listener"

@export var level:PackedScene

func _ready():
	validate()

func set_mode(to:NetworkLayer.ClientMode):
	mode = to
	validate()

func set_play_as(to:String):
	play_as = to
	if mode == NetworkLayer.ClientMode.NONE:
		mode = NetworkLayer.ClientMode.Client
	validate()

func set_username(to:String):
	username = to
	if mode == NetworkLayer.ClientMode.NONE:
		mode = NetworkLayer.ClientMode.Client
	validate()

func validate():
	get_tree().call_group(progress_status_listener_group, &"set", &"disabled", get_progress_disabled())
	get_tree().call_group(progress_message_listener_group, &"set", &"text", get_progress_text())

func get_progress_disabled() -> bool:
	if mode == NetworkLayer.ClientMode.NONE:
		return true
	if mode == NetworkLayer.ClientMode.Client:
		if ip_address == "":
			return true
	if play_as.is_empty():
		return true
	if username.is_empty():
		return true
	return false

func set_server_ip(to:String) -> void:
	ip_address = to

func set_server_port(to:int) -> void:
	port = to

func get_progress_text() -> String:
	var part1 := ""
	match mode:
		NetworkLayer.ClientMode.NONE:
			part1 = "Awaiting command"
		NetworkLayer.ClientMode.Client:
			part1 = "Press to join game"
		NetworkLayer.ClientMode.Host:
			part1 = "Press to host game"
	
	var part2 := ""
	if !play_as.is_empty():
		part2 = " as %s" % [play_as]
	
	var part3 := ""
	if !username.is_empty():
		part3 = " (%s)" % [username]
	
	var full := part1 + part2 + part3 + "..."
	return full

func start():
	if get_progress_disabled():
		push_error("MainMenuStarter::start tried to start game after get_progress_disabled() returned true")
	
	NetworkLayer.start(mode, ip_address, port, {
		"mode": NetworkLayer.ClientMode.find_key(mode),
		"username": username,
		"play_as": play_as
	})
	
	get_tree().change_scene_to_packed(level)
