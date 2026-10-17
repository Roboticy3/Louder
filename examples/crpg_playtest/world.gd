class_name CrpgWorld extends Node3D

@export var avatar_scene: PackedScene
@export var avatar_root: Node3D
@export var avatar_controller: CharacterAvatarControllerSingle
@export var spawn_origin := Vector3(2.72, 0, 5.33)
@export var spawn_radius := 2.5

var _avatars: Dictionary = {}

func set_players(players: Dictionary) -> void:
	for peer_id in _avatars.keys():
		if !players.has(peer_id):
			_despawn(peer_id)

	var peer_ids := players.keys()
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
	var angle := TAU * float(index) / 4.0
	return Vector3(cos(angle), 0, sin(angle)) * spawn_radius
