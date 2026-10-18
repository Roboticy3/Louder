extends Node

## The engine binds a method it renamed with #[func(rename = "...")] under the
## quoted string from that attribute, quotes and all. A plain call to "add_actor"
## reports the method as missing; the name it is actually registered under is
## "\"add_actor\"". Only these two need spelling that way, and they are spelled
## this way only until the engine is fixed: go back to the plain names then.
const ADD_ACTOR := "\"add_actor\""
const FLUSH_EVENTS := "\"flush_events\""

var context := LouderContext.new()

func load_conversation_file(file_path) -> int:
	var conversation_name := context.make_name(file_path)
	var fa := FileAccess.open(file_path, FileAccess.READ)
	if FileAccess.get_open_error() != OK:
		push_error("Failed to open conversation ", file_path)
		return -1
	var text := fa.get_as_text()
	var failure: int = context.load_conversation(conversation_name, text)
	if failure != 0:
		push_error("Failed to load conversation %s: %s" % [file_path, LouderLoader.error_name(failure)])
		return -1
	return conversation_name

func load_conversation_config_file(file_path) -> int:
	var fa := FileAccess.open(file_path, FileAccess.READ)
	if FileAccess.get_open_error() != OK:
		push_error("Failed to open config ", file_path)
		return -1
	var text := fa.get_as_text()
	context.load_conversation_config(text)
	return 0

## Add an actor and put it on the first hub carrying any of `start_tags`.
##
## The tag and speaker arguments are taken untyped on purpose. They go straight
## into the engine as interned names, and that conversion needs a typed array of
## ints, which neither a GDScript literal nor Array.map will hand over.
func add_actor(conversation: int, start_tags: Array, flags: int, configured_speakers: Array, configured_listen_to: Array = []) -> int:
	var start: int = context.next_hub_from_tags(conversation, _intern(start_tags), false, -1)
	if start == -1:
		push_error("Conversation %d has no hub tagged %s" % [conversation, start_tags])
		return -1
	return context.call(ADD_ACTOR, conversation, start, flags, _intern(configured_speakers), _intern(configured_listen_to))

## Put the actor back where it already stands.
##
## Adding an actor places it without saying anything, so this is how the engine
## is asked to introduce the hub the conversation opens on.
func announce_position(conversation: int, actor: int) -> bool:
	var position: int = context.get_actor_position(conversation, actor)
	if position == -1:
		return false
	return context.force_position(conversation, actor, position, -1, 0)

func get_actor_position(conversation: int, actor: int) -> int:
	return context.get_actor_position(conversation, actor)

## The choices this actor may take where it stands, as engine choice indices.
##
## These are not positions in a list of anything the caller has built: the engine
## hands back the indices it uses on the hub itself, and it only hands back the
## ones that pass its rules. Read them with get_choice_text and hand them back
## to select_choice unchanged.
func get_available_choice_indices(conversation: int, actor: int) -> Array[int]:
	return context.get_hub_choices(conversation, context.get_actor_position(conversation, actor), actor)

func get_choice_text(conversation: int, position: int, actor: int, choice_index: int) -> String:
	return _text(context.get_hub_choice_text(conversation, position, actor, choice_index))

## The speaker of a hub, already as text. Empty when the hub has no speaker,
## which is how the engine says a hub is narration rather than something said.
func get_line_speaker(conversation: int, position: int) -> String:
	return _text(context.get_hub_speaker(conversation, position))

## The text of a hub, already as text.
func get_line_text(conversation: int, position: int) -> String:
	return _text(context.get_hub_text(conversation, position))

func resolve_name(name: int) -> String:
	return _text(name)

## Take an engine choice index from get_available_choice_indices.
func select_choice(conversation: int, actor: int, choice_index: int) -> bool:
	return context.select_choice(conversation, actor, choice_index)

func get_event_count() -> int:
	return context.get_event_count()

func get_event(index: int) -> LouderEvent:
	return context.get_event(index)

func flush_events() -> void:
	context.call(FLUSH_EVENTS)

func _intern(names: Array) -> Array[int]:
	var interned: Array[int] = []
	for name in names:
		interned.append(context.make_name(name))
	return interned

func _text(name: int) -> String:
	return "" if name == -1 else LouderNameContent.resolve(context, name)
