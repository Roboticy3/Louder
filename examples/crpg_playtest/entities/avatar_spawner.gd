class_name AvatarSpawner extends Marker3D

## An avatar spawner marks a place where a player avatar may be spawned.
## It adds itself to a group so the world's spawn logic can find spawners
## without hard-coded node paths, and filters the characters it accepts by name.
## If character_names is empty, the node's name is used as a character name.

@export var group := &"avatar_spawner"
@export var character_names: Array[String] = []

# Path to a conversation that plays when a player spawns at this spawner.
@export_file_path("*.json", "*.md") var intro_conversation_path := ""

# Group used to ask the UI to start conversations.
@export var scrolling_dialogue_group := &"scrolling_dialogue"

func start_intro_conversation() -> void:
	if intro_conversation_path == "":
		return
	if not is_inside_tree():
		return
	get_tree().set_meta("pending_intro_conversation", intro_conversation_path)
	get_tree().call_deferred("call_group", scrolling_dialogue_group, &"start_conversation", intro_conversation_path)

func _ready() -> void:
	if character_names.is_empty():
		character_names.push_back(name)
	
	add_to_group(group)
	for alias in [&"avatar_spawner", &"avatar_spawners", &"AvatarSpawner", &"avatar_spawners_group"]:
		add_to_group(alias)
