extends CanvasItem

@export var group := &"state_listener"
@export var show_trigger:String
@export var hide_trigger:String
@export var hide_by_default := true

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	if hide_by_default:
		hide()
	add_to_group(group)

func _on_trigger(trigger:String):
	if trigger == show_trigger and !visible:
		modulate = Color.TRANSPARENT
		var t := create_tween()
		t.tween_property(self, "modulate", Color.WHITE, 0.65)
		show()
	if trigger == hide_trigger:
		var t := create_tween()
		t.tween_property(self, "modulate", Color.TRANSPARENT, 0.65)
		t.finished.connect(hide)
		visible = true
