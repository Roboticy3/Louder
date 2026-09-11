extends CanvasItem


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	ClientDataLayer.character_selected.connect(_on_character_selected)

func _on_character_selected(authority:int, _character:String):
	if authority != multiplayer.multiplayer_peer.get_unique_id():
		return
	hide()

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("lobby_character_select"):
		show()
