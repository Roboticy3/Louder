extends BaseButton

@export var mutual_exclusive:Array[Control]
@export var mutual_inclusive:Array[Control]
@export var inverted:bool

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	toggled.connect(_on_toggled)
	toggled.emit.call_deferred(button_pressed)

func _on_toggled(to:bool) -> void:
	for m in mutual_exclusive:
		if is_instance_valid(m):
			m.visible = !to
	for m in mutual_inclusive:
		if is_instance_valid(m):
			m.visible = to
