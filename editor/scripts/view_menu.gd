extends MenuButton

const SPEAKER_COLORS := 0
const SPEAKER_COLORS_GROUP := &"speaker_colors_menu"

func _ready():
	text = "View"
	var popup := get_popup()
	popup.add_check_item("Speaker Colors", SPEAKER_COLORS)
	popup.id_pressed.connect(_on_id_pressed)

func _on_id_pressed(id:int):
	match id:
		SPEAKER_COLORS:
			_toggle_speaker_colors()

func _toggle_speaker_colors():
	var nodes := get_tree().get_nodes_in_group(SPEAKER_COLORS_GROUP)
	var show := false
	for node in nodes:
		if node is CanvasItem and node.visible:
			show = true
			break
	show = !show
	for node in nodes:
		if node is CanvasItem:
			node.visible = show
	get_popup().set_item_checked(SPEAKER_COLORS, show)
