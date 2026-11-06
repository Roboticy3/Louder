extends Control

## A log of the conversations that have taken place.
##
## Collects a full history of hub appearance events and the choices that led to
## them, so a player can scroll back through everything that happened. The
## history is kept as a list of entries (see [member entries]) that carry the
## fields later filtering and search features will need: the conversation, the
## actor/character, the speaker, the text, the choice that was made, and the
## choices that were available. Nothing is discarded beyond the display cap, so
## the log stays complete.
##
## Future organization (by conversation, by character, or by word search) is
## meant to be built on top of [member entries] without changing this class's
## shape: every entry already carries those fields, and the helper methods below
## leave room for richer filters.

class_name ConversationLog

signal entry_added(entry: Dictionary)
signal log_cleared

@export var container: VBoxContainer
@export var max_entries := 1000

## The actor whose perspective this log follows. -1 means "all actors" (server/debug).
var player_actor := -1

## Aliases for the same filter.
var player_character_actor: int:
	get:
		return player_actor
	set(value):
		player_actor = value
var selected_actor: int:
	get:
		return player_actor
	set(value):
		player_actor = value
var actor_filter: int:
	get:
		return player_actor
	set(value):
		player_actor = value

## Aliases kept for compatibility with the event log's naming and future callers.
var max_logs: int:
	get:
		return max_entries
	set(value):
		max_entries = value

var max_history: int:
	get:
		return max_entries
	set(value):
		max_entries = value

## The full collected history, in the order events arrived.
var entries: Array[Dictionary] = []

# Compatibility aliases for the same collection.
var history: Array[Dictionary]:
	get:
		return entries

var log: Array[Dictionary]:
	get:
		return entries

var items: Array[Dictionary]:
	get:
		return entries

var _next_trace_index := 0
var _last_targets: Dictionary = {}


func _ready() -> void:
	if container == null:
		container = get_node_or_null("VBoxContainer") as VBoxContainer
		if container == null:
			container = get_node_or_null("ScrollContainer/VBoxContainer") as VBoxContainer
	if container == null:
		push_error("A ConversationLog has no container to put labels in.")
		set_process(false)

	if not LouderContextGlobal.event_received.is_connected(_on_event):
		LouderContextGlobal.event_received.connect(_on_event)


func _process(_delta: float) -> void:
	# Collection now happens through the event_received signal, so peers that
	# have no engine trace of their own (clients) still get their journal.
	while container != null and container.get_child_count() > max_entries:
		var oldest := container.get_child(0)
		container.remove_child(oldest)
		oldest.queue_free()


func _on_event(event: Dictionary) -> void:
	var kind := int(event.get("kind", -1))
	# Only hub appearance events (motions) are part of the conversation log.
	if kind < 1 or kind > 3:
		return
	var c := int(event.get("conversation", -1))
	var a := int(event.get("actor", -1))
	# Be aware of the client's perspective: only the selected character's hub
	# appearances belong in this journal.
	if player_actor >= 0 and a != player_actor:
		return

	var motion: Dictionary = event.get("motion", {})
	var target := int(motion.get("target_position", -1))
	var key := "%d:%d" % [c, a]
	# Ignore events that repeat the same position for this actor.
	if target >= 0 and _last_targets.get(key, -999999) == target:
		return
	if target >= 0:
		_last_targets[key] = target

	var entry := _make_entry_from_event(event)
	entries.append(entry)
	_add_label(entry)
	entry_added.emit(entry)


func _make_entry_from_event(event: Dictionary) -> Dictionary:
	var conversation := int(event.get("conversation", -1))
	var actor := int(event.get("actor", -1))
	var channel := int(event.get("channel", -1))
	var kind := int(event.get("kind", -1))

	var motion: Dictionary = event.get("motion", {})
	var source_position := int(motion.get("source_position", -1))
	var target_position := int(motion.get("target_position", -1))
	var choice_index := int(motion.get("choice_index", -1))
	var background := bool(motion.get("background", false))

	var speaker := str(motion.get("speaker_text", ""))
	if speaker == "":
		speaker = str(motion.get("speaker", ""))
	var text := str(motion.get("text", ""))
	if text == "" and target_position >= 0:
		text = LouderContextGlobal.get_line_text(conversation, target_position)

	var choice_text := ""
	if source_position >= 0 and choice_index >= 0:
		choice_text = LouderContextGlobal.get_choice_text(conversation, source_position, actor, choice_index)

	var available_choices: Array = motion.get("choices", [])
	if available_choices.is_empty() and target_position >= 0 and actor >= 0:
		var indices := LouderContextGlobal.get_available_choice_indices(conversation, actor)
		for idx in indices:
			available_choices.append({
				"index": idx,
				"text": LouderContextGlobal.get_choice_text(conversation, target_position, actor, idx),
				"rule": LouderContextGlobal.get_choice_rule(conversation, target_position, actor, idx),
			})

	return {
		"conversation": conversation,
		"conversation_id": conversation,
		"conversation_name": LouderContextGlobal.resolve_name(conversation) if conversation >= 0 else "",
		"channel": channel,
		"actor": actor,
		"character": speaker,
		"character_name": speaker,
		"speaker_name": speaker,
		"kind": kind,
		"background": background,
		"speaker": speaker,
		"text": text,
		"source_position": source_position,
		"target_position": target_position,
		"choice_index": choice_index,
		"choice_text": choice_text,
		"choice": choice_text,
		"selected_choice": choice_text,
		"choices": available_choices,
	}


func _add_label(entry: Dictionary) -> void:
	if container == null:
		return
	var label := Label.new()
	var speaker := str(entry.get("speaker", ""))
	var text := str(entry.get("text", ""))
	label.text = "speaker: %s\ntext: %s" % [speaker, text]
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	container.add_child(label)


func _clear() -> void:
	if container != null:
		for child in container.get_children():
			container.remove_child(child)
			child.queue_free()
	entries.clear()
	_last_targets.clear()
	log_cleared.emit()


# -- manual recording --------------------------------------------------

func add_entry(entry: Dictionary) -> void:
	entries.append(entry.duplicate(true))
	_add_label(entries.back())
	entry_added.emit(entries.back())
	while container != null and container.get_child_count() > max_entries:
		var oldest := container.get_child(0)
		container.remove_child(oldest)
		oldest.queue_free()

func record(entry: Dictionary) -> void:
	add_entry(entry)

func append_entry(entry: Dictionary) -> void:
	add_entry(entry)

func append(entry: Dictionary) -> void:
	add_entry(entry)


# -- accessors ---------------------------------------------------------

func get_entries() -> Array[Dictionary]:
	return entries

func get_history() -> Array[Dictionary]:
	return entries

func get_log() -> Array[Dictionary]:
	return entries

func get_items() -> Array[Dictionary]:
	return entries


# -- clearing ----------------------------------------------------------

func clear() -> void:
	_clear()
	_next_trace_index = 0

func clear_entries() -> void:
	clear()

func clear_history() -> void:
	clear()

func clear_log() -> void:
	clear()

func reset() -> void:
	clear()


# -- future organization helpers --------------------------------------

func entries_for_conversation(conversation) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for entry in entries:
		if entry.get("conversation") == conversation or str(entry.get("conversation_name", "")) == str(conversation):
			result.append(entry)
	return result

func by_conversation(conversation) -> Array[Dictionary]:
	return entries_for_conversation(conversation)

func for_conversation(conversation) -> Array[Dictionary]:
	return entries_for_conversation(conversation)

func filter_by_conversation(conversation) -> Array[Dictionary]:
	return entries_for_conversation(conversation)


func entries_for_character(character) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	for entry in entries:
		if entry.get("character") == character or entry.get("speaker") == character or str(entry.get("character", "")) == str(character):
			result.append(entry)
	return result

func by_character(character) -> Array[Dictionary]:
	return entries_for_character(character)

func for_character(character) -> Array[Dictionary]:
	return entries_for_character(character)

func filter_by_character(character) -> Array[Dictionary]:
	return entries_for_character(character)


func search_entries(query: String) -> Array[Dictionary]:
	var result: Array[Dictionary] = []
	var needle := query.to_lower()
	for entry in entries:
		var haystack := ("%s %s %s %s" % [
			str(entry.get("speaker", "")),
			str(entry.get("text", "")),
			str(entry.get("choice_text", "")),
			str(entry.get("conversation_name", "")),
		]).to_lower()
		if haystack.contains(needle):
			result.append(entry)
	return result

func search(query: String) -> Array[Dictionary]:
	return search_entries(query)

func find(query: String) -> Array[Dictionary]:
	return search_entries(query)

func find_entries(query: String) -> Array[Dictionary]:
	return search_entries(query)
