extends Node

@export var valid_speakers:PackedStringArray
@export var outer_voice: AudioStreamPlayer
@export var inner_voice: AudioStreamPlayer
# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	ControllerEventLayer.choice_event.connect(_on_choice)

func _on_choice(controller:PSController):
	
	if !valid_speakers.is_empty():
		var good := false
		for s in controller.speak_as:
			if s in valid_speakers:
				good = true
				break
		if !good:
			return
	
	#Now that we have access to the speaker we can do things like assigning
	#a speaker an audio stream.
	if controller.is_successful():
		if controller.is_listen_to():
			inner_voice.play()
		else:
			outer_voice.play()
