class_name CrpgWorld extends Node3D

@export var avatar_scene: PackedScene
@export var avatar_root: Node3D
@export var avatar_controller: CharacterAvatarControllerSingle
@export var spawn_origin := Vector3(2.72, 0, 5.33)
@export var spawn_radius := 2.0

# Group searched for avatar spawners.
@export var avatar_spawner_group := &"avatar_spawner"

var _avatars: Dictionary[int, CharacterAvatar] = {}

func set_players(players: Dictionary[int, CharacterData]) -> void:
	for peer_id in _avatars.keys():
		if !players.has(peer_id):
			_despawn(peer_id)

	var peer_ids: Array[int] = players.keys()
	peer_ids.sort()
	for index in peer_ids.size():
		var peer_id: int = peer_ids[index]
		var character: CharacterData = players[peer_id]
		var avatar: CharacterAvatar = _avatars.get(peer_id)
		if avatar == null:
			avatar = _spawn(peer_id, character, index)
		avatar.character_data = character

func play_as(character: CharacterData) -> void:
	avatar_controller.character_data = character
	avatar_controller.locate_character()

func _spawn(peer_id: int, character: CharacterData, index: int) -> CharacterAvatar:
	var avatar := avatar_scene.instantiate() as CharacterAvatar
	avatar.name = "Avatar%d" % peer_id
	avatar.character_data = character

	var spawner := _find_valid_avatar_spawner(character)
	if spawner != null:
		avatar_root.add_child(avatar)
		avatar.global_position = spawner.global_position
		if spawner.has_method("start_intro_conversation"):
			spawner.start_intro_conversation()
	else:
		push_warning("No avatar spawner found for character '%s'; reverting to default spawn logic." % character.name)
		avatar.position = spawn_origin + _spawn_offset(index)
		avatar_root.add_child(avatar)

	_avatars[peer_id] = avatar
	return avatar

func _despawn(peer_id: int) -> void:
	var avatar: CharacterAvatar = _avatars.get(peer_id)
	if avatar:
		avatar.queue_free()
	_avatars.erase(peer_id)

func _spawn_offset(index: int) -> Vector3:
	var angle := TAU * float(index) / 6.0
	return Vector3(cos(angle), 0, sin(angle)) * spawn_radius

func _find_valid_avatar_spawner(character: CharacterData) -> Node3D:
	var candidates: Array[Node] = []
	var groups_to_search: Array[StringName] = [
		avatar_spawner_group,
		&"avatar_spawner",
		&"avatar_spawners",
		&"AvatarSpawner",
		&"avatar_spawners_group",
	]
	if get_tree() != null:
		for group_name in groups_to_search:
			for node in get_tree().get_nodes_in_group(group_name):
				if node is Node3D and not candidates.has(node):
					candidates.append(node)

	# Always scan the tree as well, so spawners are found even if their group
	# registration differs or the searched group name is not one of the known
	# aliases.
	if get_tree() != null:
		_collect_avatar_spawners(get_tree().root, candidates)
	else:
		_collect_avatar_spawners(self, candidates)
		if avatar_root != null:
			_collect_avatar_spawners(avatar_root, candidates)

	for node in candidates:
		if node is AvatarSpawner:
			var spawner := node as AvatarSpawner
			if spawner.character_names.has(character.name):
				return spawner
		elif _node_accepts_character(node, character):
			return node as Node3D
	return null

func _collect_avatar_spawners(node: Node, out: Array[Node]) -> void:
	if node is AvatarSpawner and node is Node3D:
		if not out.has(node):
			out.append(node)
	for child in node.get_children():
		_collect_avatar_spawners(child, out)

func _node_accepts_character(node: Node, character: CharacterData) -> bool:
	# Duck-typed support for spawners that expose a character-name filter.
	if node == null or character == null:
		return false
	var names: Array[String] = []
	if "character_names" in node:
		var v = node.get("character_names")
		if v is Array:
			for item in v:
				names.append(str(item))
	if names.is_empty() and "character_name" in node:
		var v2 = node.get("character_name")
		if v2 != null:
			names = [str(v2)]
	if names.is_empty():
		# A node with no filter list is not considered a valid match unless it
		# explicitly claims to accept everything. Keep parity with AvatarSpawner,
		# which requires a name match.
		return false
	return names.has(character.name)
