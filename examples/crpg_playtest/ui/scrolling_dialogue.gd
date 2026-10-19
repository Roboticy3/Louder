class_name ScrollingDialogue extends Control

@export var group := &"scrolling_dialogue"
@export var scroll_container: ScrollContainer
@export var lines: VBoxContainer
@export var choices: VBoxContainer
@export var line_scene: PackedScene
@export var choice_scene: PackedScene
@export var start_tag := &"start"
@export var player_speaker := &"Benjamin"
## Off by default: an actor needs no flags to make choices, and a spectator is
## shown every hub and offered none of them, which is the wrong shape for a player.
@export_flags("Spectator", "Debug") var actor_flags := 0

var conversation := -1
var player_actor := -1

# The engine choice index behind each button, in the order the buttons are shown.
var _choice_indices: Array[int] = []
var _revealed_anything := false


func _ready() -> void:
	add_to_group(group)
	
	LouderContextGlobal.conversation_started.connect(_on_conversation_started)
	LouderContextGlobal.actor_added.connect(_on_actor_added)
	LouderContextGlobal.actor_removed.connect(_on_actor_removed)
	LouderContextGlobal.motion.connect(_on_motion)

func _on_conversation_started(c:int, source:NodePath):
	if source == get_path():
		LouderContextGlobal.connect_actor.rpc_id(1, c, ["start"], 1, [player_speaker], [])

func _on_actor_added(c:int, a:int):
	var speaker_name:int = LouderContextGlobal.context.make_name(player_speaker)
	var speak_as:Array[int] = LouderContextGlobal.context.get_actor(c, a).speak_as()
	if speaker_name in speak_as:
		player_actor = a
		conversation = c
		visible = true

func _on_actor_removed(c:int, a:int):
	if conversation == c and player_actor == a:
		_on_conversation_finished()

func _on_motion(c:int, a:int, target:int, background:int):
	if c == conversation and a == player_actor:
		var speaker_name:int = LouderContextGlobal.context.get_hub_speaker(c, target)
		var text_name:int = LouderContextGlobal.context.get_hub_text(c, target)
		var is_background:bool = background != 0
		_on_line_revealed(LouderContextGlobal.context.make_string_name(speaker_name), LouderContextGlobal.context.make_string_name(text_name), is_background)
		_refresh_choices()

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

func _refresh_choices() -> void:
	print("position: ", LouderContextGlobal.context.get_actor_position(conversation, player_actor))
	_choice_indices = LouderContextGlobal.get_available_choice_indices(conversation, player_actor)
	
	var hub := LouderContextGlobal.context.get_hub(conversation, 6)
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
