extends Node

var mode := NetworkLayer.ClientMode.NONE

var play_as := ""
var username := ""
var ip_address := "localhost"
var port := 45678
var use_upnp := false
var custom_conversation := ""
var custom_config := ""

@export var progress_message_listener_group := &"progress_message_listener"
@export var progress_status_listener_group := &"progress_status_listener"
@export var start_listener_group := &"start_listener"

@export var level:PackedScene
@export var alternate_level:PackedScene

func _ready():
	
	var args = OS.get_cmdline_args()
	for a in args:
		if a == "--quickstart=1":
			play_as = "Geno"
			mode = NetworkLayer.ClientMode.HOST
			start()
			break
		elif a == "--quickstart=2":
			play_as = "Reka"
			mode = NetworkLayer.ClientMode.CLIENT
			start()
			break
	
	validate()

func set_mode(to:NetworkLayer.ClientMode):
	mode = to
	validate()

func set_use_upnp(to:bool):
	use_upnp = to

func set_play_as(to:String):
	play_as = to
	if mode == NetworkLayer.ClientMode.NONE:
		mode = NetworkLayer.ClientMode.CLIENT
	validate()

func set_username(to:String):
	username = to
	if mode == NetworkLayer.ClientMode.NONE:
		mode = NetworkLayer.ClientMode.CLIENT
	validate()

func set_custom_conversation(to:String):
	if FileAccess.file_exists(to):
		custom_conversation = to
	else:
		custom_conversation = ""
	validate()
	
func set_custom_config(to:String):
	if FileAccess.file_exists(to):
		custom_config = to
	else:
		custom_config = ""
	validate()

func validate():
	get_tree().call_group(progress_status_listener_group, &"set", &"disabled", get_progress_disabled())
	get_tree().call_group(progress_message_listener_group, &"set", &"text", get_progress_text())

func get_progress_disabled() -> bool:
	if mode == NetworkLayer.ClientMode.NONE:
		return true
	if mode == NetworkLayer.ClientMode.CLIENT:
		if ip_address == "":
			return true
	return false

func set_server_ip(to:String) -> void:
	ip_address = to

func set_server_port_sus(to:float) -> void:
	set_server_port(to)

func set_server_port(to:int) -> void:
	port = to

func get_progress_text() -> String:
	var part1 := ""
	match mode:
		NetworkLayer.ClientMode.NONE:
			part1 = "Awaiting command"
		NetworkLayer.ClientMode.CLIENT:
			part1 = "Press to join game"
		NetworkLayer.ClientMode.HOST:
			part1 = "Press to host game"
	
	var part2 := ""
	if !play_as.is_empty():
		part2 = " as %s" % [play_as]
	
	var part3 := ""
	if !username.is_empty():
		part3 = " (%s)" % [username]
	
	var part4 := ""
	if !custom_conversation.is_empty():
		part4 = " with custom conversation %s" % [custom_conversation]
	
	var part5 := ""
	if !custom_conversation.is_empty() and !custom_config.is_empty():
		part5 = " and custom config %s" % [custom_config]
	
	var full := part1 + part2 + part3 + part4 + part5 + "..."
	return full

func start():
	if get_progress_disabled():
		push_error("MainMenuStarter::start tried to start game after get_progress_disabled() returned true")
	
	CustomConversationLayer.conversation_path = custom_conversation
	CustomConversationLayer.config_path = custom_config
	NetworkLayer.start(mode, ip_address, port, use_upnp)
	
	if !custom_conversation.is_empty():
		#alternate scene
		get_tree().change_scene_to_packed.call_deferred(alternate_level)
	else:
		get_tree().change_scene_to_packed.call_deferred(level)
	
