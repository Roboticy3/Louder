class_name CharacterCatalog extends RefCounted

const CHARACTER_DATA_DIRECTORY := "res://character_data"
const CHARACTER_DATA_EXTENSION := "tres"

static func load_characters() -> Array[CharacterData]:
	var characters: Array[CharacterData] = []
	for path in _find_character_data_paths():
		var character := load(path) as CharacterData
		if character:
			characters.append(character)

	characters.sort_custom(_compare_by_name)
	characters.append(CharacterData.make_debug())
	return characters

static func _compare_by_name(a: CharacterData, b: CharacterData) -> bool:
	return a.name.to_lower() < b.name.to_lower()

static func _find_character_data_paths() -> PackedStringArray:
	var paths := PackedStringArray()
	var directory := DirAccess.open(CHARACTER_DATA_DIRECTORY)
	if directory == null:
		push_error("Could not open the character data directory %s" % CHARACTER_DATA_DIRECTORY)
		return paths

	for file_name in directory.get_files():
		if file_name.get_extension() == CHARACTER_DATA_EXTENSION:
			paths.append(CHARACTER_DATA_DIRECTORY.path_join(file_name))
	return paths