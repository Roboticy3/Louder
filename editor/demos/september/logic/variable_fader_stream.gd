extends AudioStreamPlayer

@export var group := &"state_listener"
@export var show_property:StringName = &"name"
@export var show_trigger:PackedStringArray
@export var hide_by_default := true
@export var scan_state_on_connect := true

var initial_volume := volume_linear

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	if hide_by_default:
		volume_linear = 0.0
		
	add_to_group(group)

func _on_state_changed(_s, variable:String, value):
	if variable in show_trigger:
		if value:
			fade_in()
		else:
			fade_out()
	elif !show_property.is_empty() and variable == str(get(show_property)):
		if value:
			fade_in()
		else:
			fade_out()

func fade_in():
	if not playing:
		play()
	volume_linear = 0.0
	var t := create_tween()
	t.tween_property(self, "volume_linear", initial_volume, 0.65)

func fade_out():
	var t := create_tween()
	t.tween_property(self, "volume_linear", 0.0, 0.65)
	t.finished.connect(stop)
