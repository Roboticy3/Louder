class_name ScrollingDialogue extends Control

@export var scroll_container: ScrollContainer
@export var lines: VBoxContainer
@export var choices: VBoxContainer
@export var line_scene: PackedScene
@export var choice_scene: PackedScene
@export_file("*.json") var document_path := "res://conversations/jail.json"
@export_file("*.json") var config_path := "res://configs/config.json"
@export var start_tag := &"start"
@export var player_speaker := &"Benjamin"
## Off by default: an actor needs no flags to make choices, and a spectator is
## shown every hub and offered none of them, which is the wrong shape for a player.
@export_flags("Spectator", "Debug") var actor_flags := 0

# LouderEvent.get_kind() values this script reads. The engine numbers these for
# every event it can emit, so the ones not listed here are simply not matched.
const EVENT_MOTION := 1
const EVENT_WARP := 2
const EVENT_FORCED_MOTION := 3
const EVENT_CALLBACK := 11

var conversation := -1
var player_actor := -1

# The engine choice index behind each button, in the order the buttons are shown.
var _choice_indices: Array[int] = []
var _revealed_anything := false


func _ready() -> void:
	LouderContextGlobal.load_conversation_config_file(config_path)
	conversation = LouderContextGlobal.load_conversation_file(document_path)
	if conversation == -1:
		push_error("No conversation at %s." % document_path)
		return

	player_actor = LouderContextGlobal.add_actor(conversation, [start_tag], actor_flags, [player_speaker])
	if player_actor == -1:
		push_error("No player actor in the conversation at %s." % document_path)
		return

	LouderContextGlobal.announce_position(conversation, player_actor)
	_refresh()

## Which hub the player is standing on. Not named get_position: Control already
## has one of those, and it answers a different question.
func get_hub_position() -> int:
	return LouderContextGlobal.get_actor_position(conversation, player_actor)

## Take the choice behind row `row`.
##
## Rows are where a button sits on screen, which is not the engine's choice
## index: the engine only offers some of the choices on a hub and its indices
## skip the rest, so the row has to be looked up rather than passed on.
func choose(row: int) -> void:
	if conversation == -1 or player_actor == -1:
		return
	if row < 0 or row >= _choice_indices.size():
		push_error("There is no dialogue choice in row %d." % row)
		return

	var choice_index := _choice_indices[row]
	if !LouderContextGlobal.select_choice(conversation, player_actor, choice_index):
		push_error("The engine refused choice %d at hub %d." % [choice_index, get_hub_position()])
		return

	_refresh()

## Read everything the engine has queued and then redraw from where it left off.
func _refresh() -> void:
	_drain_events()
	_refresh_choices()

func _drain_events() -> void:
	# The whole queue is read out before the UI is touched, so a listener that
	# calls back into the engine cannot have the drain it is interrupting eat the
	# answer.
	var positions := PackedInt32Array()
	var backgrounds := PackedByteArray()
	var callbacks := PackedInt32Array()

	var index := 0
	while index < LouderContextGlobal.get_event_count():
		var event: LouderEvent = LouderContextGlobal.get_event(index)
		if event.get_actor() == player_actor:
			match event.get_kind():
				EVENT_MOTION, EVENT_WARP, EVENT_FORCED_MOTION:
					var motion := event.get_motion()
					positions.append(motion.get_target_position())
					backgrounds.append(1 if motion.is_background() else 0)
				EVENT_CALLBACK:
					callbacks.append(event.get_name())
		index += 1

	LouderContextGlobal.flush_events()

	for revealed in positions.size():
		_revealed_anything = true
		_on_line_revealed(
			LouderContextGlobal.get_line_speaker(conversation, positions[revealed]),
			LouderContextGlobal.get_line_text(conversation, positions[revealed]),
			backgrounds[revealed] == 1)

	for callback in callbacks:
		_on_callback_raised(LouderContextGlobal.resolve_name(callback))

func _refresh_choices() -> void:
	_choice_indices = LouderContextGlobal.get_available_choice_indices(conversation, player_actor)

	var texts := PackedStringArray()
	for index in _choice_indices.size():
		texts.append(LouderContextGlobal.get_choice_text(conversation, get_hub_position(), player_actor, _choice_indices[index]))

	_on_choices_revealed(texts)

	# One actor means nobody else is going to move the flow along, so a hub that
	# offers nothing is the end of the conversation. More than one actor will need
	# a better test than that.
	if texts.is_empty() and _revealed_anything:
		_on_conversation_finished()

var scrolling_to_bottom := false
func _on_line_revealed(speaker: String, text: String, background: bool) -> void:
	var line := line_scene.instantiate() as DialogueLine
	line.setup(speaker, text, background)
	lines.add_child(line)

func _on_choices_revealed(texts: PackedStringArray) -> void:
	for child in choices.get_children():
		choices.remove_child(child)
		child.queue_free()

	for row in texts.size():
		var choice := choice_scene.instantiate() as DialogueChoice
		choice.setup(row, texts[row])
		choice.chosen.connect(_on_choice_chosen)
		choices.add_child(choice)
	
	if texts.is_empty():
		choices.custom_minimum_size.y = 0.0
	else:
		choices.custom_minimum_size.y = scroll_container.size.y * .3
	scrolling_to_bottom = true

func _on_choice_chosen(row: int) -> void:
	choose(row)

func _on_callback_raised(callback: String) -> void:
	# Callbacks are the engine's hook for game state. Nothing acts on them yet, so
	# they are printed to keep the playtest visible.
	print("Dialogue callback: %s" % callback)

func _on_conversation_finished() -> void:
	print("Dialogue conversation finished.")

func _process(delta: float) -> void:
	if scrolling_to_bottom:
		var bar := scroll_container.get_v_scroll_bar() 
		bar.value += 300 * delta
		if bar.value > bar.max_value - 10:
			scrolling_to_bottom = false
