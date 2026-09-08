extends Node

@export var group := &"state_listener"
@export var player:AudioStreamPlayer

@export var components:Dictionary[String, AudioStream]

@onready var playlist:AudioStreamPlaylist

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	add_to_group(group)
	
	if player.stream is AudioStreamPlaylist:
		playlist = player.stream.duplicate()
	else:
		playlist = AudioStreamPlaylist.new()
	player.stream = playlist

func _on_state_changed(_s, variable, value):
	if variable in components:
		var stream := components[variable]
		if value:
			var insert := playlist.stream_count
			playlist.stream_count += 1
			playlist.set_list_stream(insert, stream)
		else:
			for i in playlist.stream_count:
				if playlist.get_list_stream(i) == stream:
					playlist.set_list_stream(i, null)
