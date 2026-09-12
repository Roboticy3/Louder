extends CanvasItem

@export var background:Texture2D
@export var characters:Array[CharacterSelectItemData]
@export var config:PSConfig

@export var background_rect:TextureRect
@export var slot:Control
@export var item_packed:PackedScene

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	background_rect.texture = background
	
	var make_temp_characters := characters.is_empty()
	if make_temp_characters:
		
		for n in config.speakers_by_name:
			var n_data:Dictionary = config.speakers_by_name[n]
			if n_data.get(config.SPK_OWND, false):
				var c := CharacterSelectItemData.new()
				c.name = n_data.get(config.SPK_NAME, "")
				c.description = n_data.get(config.SPK_DESC, "")
				characters.append(c)
	
	for c in characters:
		var instance := item_packed.instantiate()
		instance.set("data", c)
		slot.add_child(instance)
	
	if make_temp_characters:
		characters.clear()
	
	ClientDataLayer.character_selected.connect(_on_character_selected)

func _on_character_selected(authority:int, _character:String):
	if authority != multiplayer.multiplayer_peer.get_unique_id():
		return
	hide()

func _unhandled_input(event: InputEvent) -> void:
	if event.is_action_pressed("lobby_character_select"):
		show()
