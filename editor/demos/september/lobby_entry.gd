extends HBoxContainer

@export var group := &"lobby_entry"
@export var mode:Label
@export var username:Label
@export var character:Label

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	add_to_group(group)
	
func set_mode(to:String) -> void:
	mode.text = to + ":"

func set_username(to:String) -> void:
	username.text = to

func set_character(to:String) -> void:
	character.text = " (playing as " + to + ")"

func get_username() -> String:
	return username.text
