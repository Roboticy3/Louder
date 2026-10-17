class_name DialogueSession extends RefCounted

signal line_revealed(speaker: String, text: String, background: bool)
signal choices_revealed(texts: PackedStringArray)
signal callback_raised(callback: String)
signal conversation_finished

# The engine writes plain integers across the boundary, and its blanks are all
# minus one, so the caller of a Louder function gets them back as ints.
const NO_NAME := -1
const BLANK_POSITION := -1
const NO_ACTOR := -1
const LOAD_OK := 0

# LouderEvent.get_kind() discriminants that a singleplayer log cares about.
const _EVENT_REGULAR_MOTION := 1
const _EVENT_WARP := 2
const _EVENT_FORCE_MOTION := 3
const _EVENT_CALLBACK := 11
const _EVENT_LOAD_ERROR := 18

var _context := LouderContext.new()
var _conversation := NO_NAME
var _actor := NO_ACTOR
# Left untyped on purpose: the engine hands back a plain array and a typed member
# would refuse it at runtime over a distinction nothing here depends on.
var _choice_indices: Array = []
var _revealed_anything := false

## Load a conversation and place the player on its first hub tagged `start_tag`.
##
## Returns an empty string on success and a sentence explaining itself otherwise,
## which the caller is free to show as-is.
func start(document_path: String, start_tag: StringName, speaker: StringName) -> String:
	var document := FileAccess.get_file_as_string(document_path)
	if document.is_empty():
		return "Could not read %s." % document_path

	_conversation = _context.make_name(StringName(document_path.get_file().get_basename()))
	var start := _context.make_name(start_tag)

	var load_error: int = _context.load_conversation(_conversation, document)
	if load_error != LOAD_OK:
		return "Could not load %s: %s." % [document_path, LouderLoader.error_name(load_error)]

	var start_position: int = _context.next_hub_from_tags(_conversation, [start], false, BLANK_POSITION)
	if start_position == BLANK_POSITION:
		return "%s has no hub tagged %s." % [document_path, start_tag]

	_report_load_failures(document_path)

	# Actors are announced, so the queue has to start empty or the log opens with
	# an actor that never said anything.
	_context."flush_events"()
	_actor = _context.add_actor(_conversation, start_position, 0, [_context.make_name(speaker)], [])
	if _actor == NO_ACTOR:
		return "Could not add an actor to %s." % document_path

	# Adding an actor drops it on the start hub silently. Forcing it there as well
	# is what makes the engine announce the hub, and that announcement is the only
	# path into this class's signals, so the log and the choices come from the same
	# place no matter how the flow began.
	_context.force_position(_conversation, _actor, start_position, NO_NAME, 0)
	_drain()
	return ""

## Take the choice shown in row `row` of the last choices_revealed.
##
## Rows are positions in the visible list rather than engine choice indices: the
## engine only shows some of the choices on a hub, and the indices it uses skip
## the rest.
func select_choice(row: int) -> bool:
	if row < 0 or row >= _choice_indices.size():
		push_error("There is no dialogue choice in row %d." % row)
		return false

	if !_context.select_choice(_conversation, _actor, _choice_indices[row]):
		push_error("The engine refused choice %d at hub %d." % [_choice_indices[row], get_position()])
		return false

	_drain()
	return true

func get_position() -> int:
	return _context.get_actor_position(_conversation, _actor)

func _drain() -> void:
	var positions := PackedInt32Array()
	var backgrounds := PackedByteArray()
	var callbacks := PackedInt32Array()

	var index := 0
	while index < _context.get_event_count():
		var event: LouderEvent = _context.get_event(index)
		if event == null:
			break
		if event.get_actor() == _actor:
			match event.get_kind():
				_EVENT_REGULAR_MOTION, _EVENT_WARP, _EVENT_FORCE_MOTION:
					var motion: Motion = event.get_motion()
					positions.append(motion.get_target_position())
					backgrounds.append(1 if motion.is_background() else 0)
				_EVENT_CALLBACK:
					callbacks.append(event.get_name())
		index += 1

	# Everything is read out of the queue before a single signal goes out, so a
	# listener is free to call back into the session without the drain it is
	# interrupting swallowing the answer.
	_context.flush_events()

	for revealed in positions.size():
		var position := positions[revealed]
		_revealed_anything = true
		line_revealed.emit(_speaker_of(position), _text_of(position), backgrounds[revealed] == 1)

	for callback in callbacks:
		callback_raised.emit(_resolve(callback))

	_refresh_choices()

func _refresh_choices() -> void:
	var position := get_position()
	_choice_indices = _context.get_hub_choices(_conversation, position, _actor)

	var texts := PackedStringArray()
	for index in _choice_indices.size():
		texts.append(_resolve(_context.get_hub_choice_text(_conversation, position, _actor, _choice_indices[index])))

	# Emitted whether or not there are choices, since an empty list is how the
	# log says "nothing to pick here".
	choices_revealed.emit(texts)

	# With one actor there is nobody else to move the flow along, so a hub with no
	# choices available is the end of it. Multiplayer will need a better test.
	if texts.is_empty() and _revealed_anything:
		conversation_finished.emit()

func _speaker_of(position: int) -> String:
	return _resolve(_context.get_hub_speaker(_conversation, position))

func _text_of(position: int) -> String:
	return _resolve(_context.get_hub_text(_conversation, position))

func _resolve(name: int) -> String:
	return "" if name == NO_NAME else LouderNameContent.resolve(_context, name)

## One unreadable record costs the caller that record and not the conversation,
## so the engine reports it through the queue instead of by failing. Swallowing
## those events would leave a document quietly missing hubs.
func _report_load_failures(document_path: String) -> void:
	var index := 0
	while index < _context.get_event_count():
		var event: LouderEvent = _context.get_event(index)
		if event != null and event.get_kind() == _EVENT_LOAD_ERROR:
			var failure: LoadFailure = event.get_load_failure()
			push_error("%s has an unreadable record at hub %d: %s." % [
				document_path, failure.get_position(), LouderLoader.error_name(failure.get_error())])
		index += 1
