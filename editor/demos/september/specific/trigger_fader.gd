extends CanvasItem

@export var group := &"state_listener"
@export var show_trigger:PackedStringArray
@export var hide_by_default := true
@export var scan_state_on_connect := true

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	if hide_by_default:
		hide()
		
	add_to_group(group)

func _on_state_changed(_s, variable:String, value):
	if value:
		if variable in show_trigger: fade_in()
	else:
		if variable in show_trigger: fade_out()

func fade_in():
	modulate = Color.TRANSPARENT
	var t := create_tween()
	t.tween_property(self, "modulate", Color.WHITE, 0.65)
	show()

func fade_out():
	var t := create_tween()
	t.tween_property(self, "modulate", Color.TRANSPARENT, 0.65)
	t.finished.connect(hide)
	visible = true
