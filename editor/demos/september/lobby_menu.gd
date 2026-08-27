extends CanvasLayer

@export var group := &"lobby"
@export var lobby_entry_group := &"lobby_entry"
@export var lobby_entry_point:Control
const LOBBY_ENTRY := preload("res://demos/september/lobby_entry.tscn")

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	add_to_group(group)
	for c in lobby_entry_point.get_children():
		if c.is_in_group(lobby_entry_group):
			c.queue_free()
	hide()

func _on_client_connect():
	var le = LOBBY_ENTRY.instantiate()
	lobby_entry_point.add_child(le)

func _on_client_disconnect():
	pass

func _input(event: InputEvent) -> void:
	if event.is_action("lobby_menu"):
		visible = event.is_pressed()
