extends CanvasItem

@export var background:Texture2D
@export var characters:Array[CharacterSelectItemData]

@export var background_rect:TextureRect
@export var slot:Control
@export var item_packed:PackedScene

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	background_rect.texture = background
	
	for c in characters:
		var instance := item_packed.instantiate()
		instance.set("data", c)
		slot.add_child(instance)
	
	ClientDataLayer.character_selected.connect(_on_character_selected)

func _on_character_selected(authority:int, _character:String):
	if authority != multiplayer.multiplayer_peer.get_unique_id():
		return
	hide()

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("lobby_character_select"):
		show()
