class_name OnScreenDebugger extends RichTextLabel

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("graph_debug"):
		visible = !visible

@export var line_duration := 1.0
@export var max_messages := 10
@onready var message_count := 1 + text.count("\n")

func _ready() -> void:
	var timer := Timer.new()
	timer.wait_time = line_duration
	timer.autostart = true
	timer.one_shot = false
	timer.timeout.connect(_on_line_timeout)
	add_child(timer)

func _on_line_timeout():
	var w := text.find("\n")
	if w != -1:
		message_count -= 1
	if w < text.length() - 1:
		text = text.substr(w + 1)

func add_message(s:String):
	while message_count >= max_messages && text.find("\n"):
		_on_line_timeout()
	text += "\n" + s
	message_count += 1
