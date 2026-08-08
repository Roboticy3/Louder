extends CheckBox

@export var mutual_exclusive:Array[Control]
@export var inverted:bool

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	toggled.connect(_on_toggled)
	_on_toggled(button_pressed)

func _on_toggled(to:bool) -> void:
	for m in mutual_exclusive:
		if is_instance_valid(m):
			m.visible = to if inverted else !to
