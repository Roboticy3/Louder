extends Control

#when pressed, wait for a key input to bind it to an action.
@export var activate:BaseButton
@export var action_label:Label
@export var bind_label:Label
@export var alias:String

var action: StringName
var listening := false

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	action = name if alias.is_empty() else StringName(alias)
	action_label.text = action
	bind_label.text = make_bind_text()
	activate.text = "Set"
	activate.pressed.connect(_on_activate_pressed)

func make_bind_text() -> String:
	return Settings.get_bind_text(action)

func _on_activate_pressed() -> void:
	listening = true
	activate.text = "Press..."
	bind_label.text = ""

func _input(event: InputEvent) -> void:
	if not listening:
		return
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_ESCAPE:
			cancel_listening()
			return
		apply_binding(event)
	elif event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_LEFT:
			cancel_listening()
			return
		apply_binding(event)
	elif event is InputEventJoypadButton and event.pressed:
		apply_binding(event)

func apply_binding(event: InputEvent) -> void:
	Settings.bind_action(action, event)
	cancel_listening()

func cancel_listening() -> void:
	listening = false
	activate.text = "Set"
	bind_label.text = make_bind_text()
