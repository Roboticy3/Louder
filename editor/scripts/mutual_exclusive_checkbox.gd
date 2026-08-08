extends CheckBox

@export var mutual_exclusive:Array[Control]

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	toggled.connect(_on_toggled)

func _on_toggled(to:bool) -> void:
	for m in mutual_exclusive:
		if is_instance_valid(m):
			m.visible = !to
