class_name CharacterData extends Resource

const ACTOR_FLAG_DEBUG := 0x2

@export var name: String
@export_multiline var description: String
@export var is_debug := false

static func make_debug() -> CharacterData:
	var character := CharacterData.new()
	character.name = "Debug"
	character.description = "A debug actor. It sees every hub and every choice, speaks to nobody, and has no speaker in dialogue."
	character.is_debug = true
	return character

func get_id() -> StringName:
	return &"debug" if is_debug else StringName(name)

func get_actor_flags() -> int:
	return ACTOR_FLAG_DEBUG if is_debug else 0

func get_speaker_names() -> Array[StringName]:
	var speakers: Array[StringName] = []
	if !is_debug:
		speakers.append(StringName(name))
	return speakers
