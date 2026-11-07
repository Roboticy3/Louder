class_name ScrollingDialogue extends Control

@export var group := &"scrolling_dialogue"
@export var scroll_container: ScrollContainer
@export var lines: VBoxContainer
@export var choices: VBoxContainer
@export var line_scene: PackedScene
@export var choice_scene: PackedScene
@export var start_tag := &"start"
## Off by default: an actor needs no flags to make choices, and a spectator is
## shown every hub and offered none of them, which is the wrong shape for a player.
@export_flags("Spectator", "Debug") var actor_flags := 0

@export var show_rules := true

## The speaker this dialogue plays as. When empty, the local peer's selected
## character from the lobby is used.
@export var player_speaker := &""

signal player_actor_changed(actor: int)

var conversation := -1
var player_actor := -1

# The engine choice index behind each button, in the order the buttons are shown.
var _choice_indices: Array[int] = []
var _revealed_anything := false
var _last_hub_position := -1
var _last_hub_conversation := -1


func _ready() -> void:
	add_to_group(group)
	
	LouderContextGlobal.conversation_started.connect(_on_conversation_started)
	LouderContextGlobal.actor_added.connect(_on_actor_added)
	LouderContextGlobal.actor_removed.connect(_on_actor_removed)
	LouderContextGlobal.event_received.connect(_on_event)
	
	visible = false

	if get_tree().has_meta("pending_intro_conversation"):
		var pending := str(get_tree().get_meta("pending_intro_conversation"))
		if pending != "":
			call_deferred("start_conversation", pending)


## Ask the server to start `document_path` for this dialogue's speaker.
##
## This is the only request the interface makes. The server creates an actor for
## every player character as part of starting the conversation, so there is no
## separate add-an-actor step to wait for: when `conversation_started` comes back
## this dialogue's actor is already in the conversation.
func _local_speaker_name() -> String:
	if str(player_speaker) != "":
		return str(player_speaker)
	var c_data := Network.get_character_of()
	if c_data == null:
		return ""
	return str(c_data.name)

func start_conversation(document_path: String) -> void:
	LouderContextGlobal.start_conversation(document_path, get_path(), _local_speaker_name())


func _on_conversation_started(c: int, source: NodePath, source_character: String) -> void:
	# Only the dialogue that asked for this conversation claims it, and only in
	# the speaker its own controller asked for. Other player characters have
	# actors in every conversation now, so the speaker alone is not enough.
	var c_data := Network.get_character_of()
	if !c_data:
		# Fall back to the configured speaker when the lobby hasn't told us yet.
		if _local_speaker_name() == "":
			return
	var local_speaker := _local_speaker_name()
	if local_speaker == "":
		return
	if source != get_path() or str(source_character) != local_speaker:
		return

	conversation = c
	player_actor = LouderContextGlobal.get_actor_for_speaker(c, local_speaker)
	visible = player_actor != -1
	_last_hub_position = -1
	_last_hub_conversation = c
	player_actor_changed.emit(player_actor)


func _on_actor_added(c: int, a: int, speak_as: Array) -> void:
	# The actor of the conversation this dialogue started may arrive after the
	# conversation was announced, since a client learns about it by relay. A
	# peer that joins a conversation already in progress also learns about
	# its actor this way, so claim the conversation here rather than only in
	# _on_conversation_started.
	var local_speaker := _local_speaker_name()
	if local_speaker == "":
		return
	if conversation != c or player_actor != -1:
		return
	if not _speaks_as(speak_as, local_speaker):
		return

	player_actor = a
	visible = true
	player_actor_changed.emit(player_actor)


func _speaks_as(speak_as: Array, speaker: StringName) -> bool:
	for name in speak_as:
		if str(LouderContextGlobal.resolve_name(int(name))) == str(speaker):
			return true
	return false

func _on_actor_removed(c:int, a:int):
	if conversation == c and player_actor == a:
		_on_conversation_finished()
		player_actor = -1
		conversation = -1
		visible = false

func _on_event(event: Dictionary):
	var kind := int(event.get("kind", -1))
	if kind not in [1, 2, 3]:
		return
	var c := int(event.get("conversation", -1))
	var a := int(event.get("actor", -1))
	if c != conversation or a != player_actor:
		return
	var motion_dict: Dictionary = event.get("motion", {})
	var target := int(motion_dict.get("target_position", -1))
	# Ignore events that repeat the same position for this conversation.
	if target >= 0 and c == _last_hub_conversation and target == _last_hub_position:
		return
	if target >= 0:
		_last_hub_position = target
		_last_hub_conversation = c
	var speaker := str(motion_dict.get("speaker_text", ""))
	var text := str(motion_dict.get("text", ""))
	var is_background: bool = bool(motion_dict.get("background", false))
	_on_line_revealed(speaker, text, is_background)

	var choice_indices: Array = motion_dict.get("choice_indices", [])
	var revealed_choices: Array = motion_dict.get("choices", [])
	_choice_indices = []
	for idx in choice_indices:
		_choice_indices.append(int(idx))
	_on_choices_revealed(revealed_choices)

	if revealed_choices.is_empty() and _revealed_anything:
		_on_conversation_finished()

## Which hub the player is standing on. Not named get_position: Control already
## has one of those, and it answers a different question.
func get_hub_position() -> int:
	if conversation == -1 or player_actor == -1:
		return -1
	return LouderContextGlobal.get_actor_position(conversation, player_actor)

## Take the choice behind row `row`.
##
## Rows are where a button sits on screen, which is not the engine's choice
## index: the engine only offers some of the choices on a hub and its indices
## skip the rest, so the row has to be looked up rather than passed on.
func choose(row: int) -> void:
	var local_speaker := _local_speaker_name()
	if local_speaker == "":
		return
	if conversation == -1 or player_actor == -1:
		return
	if row < 0 or row >= _choice_indices.size():
		push_error("There is no dialogue choice in row %d." % row)
		return

	var choice_index := _choice_indices[row]
	if !LouderContextGlobal.select_choice(conversation, local_speaker, choice_index):
		push_error("The engine refused choice %d at hub %d." % [choice_index, get_hub_position()])
		return

func _refresh_choices() -> void:
	print("position: ", LouderContextGlobal.context.get_actor_position(conversation, player_actor))
	_choice_indices = LouderContextGlobal.get_available_choice_indices(conversation, player_actor)
	
	var revealed_choices: Array[Dictionary] = []
	for index in _choice_indices.size():
		var choice_index := _choice_indices[index]
		revealed_choices.append({
			"rule": LouderContextGlobal.get_choice_rule(conversation, get_hub_position(), player_actor, choice_index),
			"text": LouderContextGlobal.get_choice_text(conversation, get_hub_position(), player_actor, choice_index),
		})

	_on_choices_revealed(revealed_choices)

	# One actor means nobody else is going to move the flow along, so a hub that
	# offers nothing is the end of the conversation. More than one actor will need
	# a better test than that.
	if revealed_choices.is_empty() and _revealed_anything:
		_on_conversation_finished()

var scrolling_to_bottom := false
func _on_line_revealed(speaker: String, text: String, background: bool) -> void:
	var line := line_scene.instantiate() as DialogueLine
	line.setup(speaker, text, background)
	lines.add_child(line)

func _on_choices_revealed(revealed_choices: Array) -> void:
	for child in choices.get_children():
		choices.remove_child(child)
		child.queue_free()

	for row in revealed_choices.size():
		var choice_data: Dictionary = revealed_choices[row]
		var text := str(choice_data.get("text", ""))
		var rule :=str(choice_data.get("rule", ""))
		if rule == "spoke" or rule == "skill":
			text = "CONTINUE."
		if show_rules:
			text = "(%s): %s" % [choice_data.get("rule", "RULE MISSING!"), text]
		var choice := choice_scene.instantiate() as DialogueChoice
		choice.setup(row, text)
		choice.chosen.connect(_on_choice_chosen)
		choices.add_child(choice)
	
	if revealed_choices.is_empty():
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
