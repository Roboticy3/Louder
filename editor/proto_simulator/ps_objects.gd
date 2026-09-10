class_name PSObjects

enum BodyProperties {
	branch = 0,
	branch_changed,
	position,
	speaker,
	text,
	success,
	dominated,
}

enum HubProperties {
	speaker = 0,
	text,
	private,
	recipient,
	auto,
	start,
	once,
	choices,
	branch,
	callbacks,
	conditions,
	statements,
	conversation_id,
}

enum ChoiceProperties {
	from_choice = 0,
	from_id,
	to_id
}

enum SpeakerProperties {
	name = 0,
	color,
	relationships,
}
