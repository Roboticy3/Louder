extends CanvasLayer

@export var group := &"lobby"
@export var lobby_entry_group := &"lobby_entry"

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	add_to_group(group)
	hide()

func _input(event: InputEvent) -> void:
	if event.is_action("lobby_menu"):
		visible = event.is_pressed()
