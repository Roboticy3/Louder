extends BaseButton


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pressed.connect(Settings.menu.show)
