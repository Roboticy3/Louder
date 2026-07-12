extends Button

@export var manager:Node
@export var save_as_dialog:FileDialog

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pressed.connect(_on_pressed)

func _on_pressed():
	if !manager.save_current_project():
		save_as_dialog.show()
	
