extends BaseButton

@export var character:String

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	ClientDataLayer.data_error.connect(_on_data_error)
	ClientDataLayer.data_updated.connect(_on_data_updated)
	
	toggled.connect(_on_toggled)

func _on_toggled(toggled_on:bool):
	if toggled_on:
		ClientDataLayer.update_client_data.rpc_id(1, {
			ClientDataLayer.ClientProperties.character: character
		})
	elif button_group and button_group.get_pressed_button() == null:
		ClientDataLayer.update_client_data.rpc_id(1, {
			ClientDataLayer.ClientProperties.character: ""
		})

func _on_data_error(error: ClientDataLayer.DataError):
	match error:
		ClientDataLayer.DataError.INVALID_CHARACTER:
			button_pressed = false
		ClientDataLayer.DataError.CHARACTER_ALREADY_IN_USE:
			button_pressed = false
		_:
			pass

func _on_data_updated(_authority, _data):
	disabled = !button_pressed and !ClientDataLayer.is_character_available(character)
