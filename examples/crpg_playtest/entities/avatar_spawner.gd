class_name AvatarSpawner extends Marker3D

## An avatar spawner marks a place where a player avatar may be spawned.
## It adds itself to a group so the world's spawn logic can find spawners
## without hard-coded node paths, and filters the characters it accepts by name.

@export var group := &"avatar_spawner"
@export var character_names: Array[String] = []

func _ready() -> void:
	add_to_group(group)
	for alias in [&"avatar_spawner", &"avatar_spawners", &"AvatarSpawner", &"avatar_spawners_group"]:
		add_to_group(alias)
