class_name DialogueLine extends VBoxContainer

@export var speaker_label: Label
@export var text_label: Label

func setup(speaker: String, text: String, background: bool) -> void:
	# A line with no speaker is narration rather than something said. A hub
	# without a speaker of its own still has one: the voice of whoever walked
	# onto it.
	speaker_label.text = speaker
	speaker_label.visible = !speaker.is_empty()
	text_label.text = text
	modulate = Color(1, 1, 1, 0.55) if background else Color.WHITE