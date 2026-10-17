class_name DialogueLine extends VBoxContainer

@export var speaker_label: Label
@export var text_label: Label

func setup(speaker: String, text: String, background: bool) -> void:
	# A hub without a speaker is narration rather than something said, and the
	# engine reports those as an unnamed speaker.
	speaker_label.text = speaker
	speaker_label.visible = !speaker.is_empty()
	text_label.text = text
	modulate = Color(1, 1, 1, 0.55) if background else Color.WHITE