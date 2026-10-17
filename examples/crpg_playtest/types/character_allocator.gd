class_name CharacterAllocator extends RefCounted

signal changed

enum Result {
	SELECTED,
	UNKNOWN_CHARACTER,
	TAKEN,
}

const NO_PEER := 0

var _characters: Array[CharacterData] = []
var _character_of_peer: Dictionary = {}
var _peer_of_character: Dictionary = {}

func setup(characters: Array[CharacterData]) -> void:
	_characters = characters
	reset()

func get_characters() -> Array[CharacterData]:
	return _characters

func get_character(character_id: StringName) -> CharacterData:
	for character in _characters:
		if character.get_id() == character_id:
			return character
	return null

func get_peer_of(character_id: StringName) -> int:
	return _peer_of_character.get(character_id, NO_PEER)

func get_character_of(peer_id: int) -> StringName:
	return _character_of_peer.get(peer_id, &"")

func select(peer_id: int, character_id: StringName) -> Result:
	if get_character(character_id) == null:
		return Result.UNKNOWN_CHARACTER

	var holder := get_peer_of(character_id)
	if holder != NO_PEER && holder != peer_id:
		return Result.TAKEN

	_release(peer_id)
	_character_of_peer[peer_id] = character_id
	_peer_of_character[character_id] = peer_id
	changed.emit()
	return Result.SELECTED

func deselect(peer_id: int) -> Result:
	_release(peer_id)
	changed.emit()
	return Result.SELECTED

func release_peer(peer_id: int) -> void:
	if _release(peer_id):
		changed.emit()

func reset() -> void:
	_character_of_peer.clear()
	_peer_of_character.clear()
	changed.emit()

func to_state() -> Dictionary:
	return _character_of_peer.duplicate()

func _release(peer_id: int) -> bool:
	var character_id := get_character_of(peer_id)
	if character_id == &"":
		return false

	_peer_of_character.erase(character_id)
	_character_of_peer.erase(peer_id)
	return true