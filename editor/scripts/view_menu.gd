extends MenuButton

const SPEAKER_COLORS := 0
const SPEAKER_COLORS_GROUP := &"speaker_colors"

func _ready():
	text = "View"
	var popup := get_popup()
	popup.add_item("Speaker Colors", SPEAKER_COLORS)
	popup.id_pressed.connect(_on_id_pressed)

func _on_id_pressed(id:int):
	match id:
		SPEAKER_COLORS:
			_toggle_speaker_colors()

func _toggle_speaker_colors():
	var nodes := get_tree().get_nodes_in_group(SPEAKER_COLORS_GROUP)
	var any_visible := false
	for node in nodes:
		if node.get("visible"):
			any_visible = true
			break
	for node in nodes:
		node.set("visible", !any_visible)
