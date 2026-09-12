extends Button

@export var level:PackedScene
@export var conversation:LineEdit

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	for a in OS.get_cmdline_args():
		if a == "--quickstart=editor":
			conversation.text = "res://demos/conversations/merchant.json"
			_on_pressed()
			break
	pressed.connect(_on_pressed)

func _on_pressed():
	CustomConversationLayer.conversation_path = conversation.text
	get_tree().change_scene_to_packed(level)
