class_name HubResource extends Resource

@export var conversation_id: int
@export var speaker: String
@export var text_content: String
@export var dc_enabled: bool
@export var dc_skill: String
@export var dc_level: int
@export var expressions: PackedStringArray
@export var start_enabled: bool
@export var choices: PackedStringArray
@export var position_offset: Vector2
@export var metadata:Dictionary[String, Variant]

func _to_string() -> String:
	return """HubResource(
		speaker: {}
		text_content: {}
		dc_enabled: {}
		dc_skill: {}
		dc_level: {}
		start_enabled: {}
		choice count: {}
	)""".format([
		speaker,
		text_content,
		dc_enabled,
		dc_skill,
		dc_level,
		start_enabled,
		choices.size()
	], "{}")
