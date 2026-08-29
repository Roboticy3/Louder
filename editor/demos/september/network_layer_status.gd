extends CanvasLayer

const MESSAGE_LIFETIME := 4.0
const TICK := 0.2

@onready var message_box:VBoxContainer = $MarginContainer/VBoxContainer
@onready var timer:Timer = $Timer

var posted:Array[Dictionary] = []

func _ready() -> void:
	timer.wait_time = TICK
	timer.timeout.connect(_on_timer_timeout)
	timer.start()

func post(text:String) -> void:
	var label := Label.new()
	label.text = text
	message_box.add_child(label)
	posted.append({"label": label, "age": 0.0})

func _on_timer_timeout() -> void:
	for i in range(posted.size() - 1, -1, -1):
		posted[i]["age"] += TICK
		if posted[i]["age"] >= MESSAGE_LIFETIME:
			var label:Label = posted[i]["label"]
			posted.remove_at(i)
			message_box.remove_child(label)
			label.queue_free()
