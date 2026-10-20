extends Node

## The autoload that holds the engine, runs it, and tells everyone what it did.
##
## A LouderContext is a conversation in progress, and running one is a thing only
## one peer may do: two peers running the same actor against the same choices
## come apart the moment the engine decides anything. So this singleton is
## server-authoritative. The server loads conversations, owns the actors and
## takes their choices. Every other peer loads the same documents, keeps a copy
## of each actor so its own context can answer questions about it, and is told
## what the server decided.
##
## The engine names things with integers it hands out as it interns them, so an id
## on one peer means the same as that id on another only if both peers read the
## same documents in the same order. A conversation is therefore loaded on every
## peer instead of being shipped across, and actors are added in the order the
## server hands them out, which is what keeps them numbered the same everywhere.
##
## What a listener needs arrives as a signal, and every signal is local: a peer
## gets its own copy of them whether it is the server or was told by one.
##
## A lone peer with no lobby is a server. Godot still has its offline multiplayer
## peer in place, and that peer answers as id 1, so the playtest and the headless
## tests go down the same path a hosted session does.

signal conversation_started(conversation: int, source: NodePath)
signal actor_added(conversation: int, actor: int)
signal actor_removed(conversation: int, actor: int)
signal motion(conversation: int, actor: int, target: int, background: int)
signal callback_raised(callback: String)

const SERVER_ID := 1

# The engine writes plain integers across the boundary, and its blanks are all
# minus one, so the caller of a Louder function gets them back as ints.
const NO_NAME := -1
const NO_ACTOR := -1
const BLANK_POSITION := -1
const LOAD_OK := 0
# A document loaded twice is the document that is already there, not a failure:
# the same conversation is announced to every peer and to the server itself.
const LOAD_ALREADY_LOADED := 4

# LouderEvent.get_kind() values this singleton reads. The engine numbers every
# event it can emit, so the ones not listed here are simply not matched.
const _EVENT_REGULAR_MOTION := 1
const _EVENT_WARP := 2
const _EVENT_FORCE_MOTION := 3
const _EVENT_CALLBACK := 11
const _EVENT_ACTOR_ADDED := 12
const _EVENT_ACTOR_REMOVED := 13
const _EVENT_LOAD_ERROR := 18

## The engine. Every question about a conversation is answered out of it, and on
## every peer it is that peer's own context rather than the server's.
var context := LouderContext.new()


func _process(_delta: float) -> void:
	# Only the server has anything to read here. Every other peer is told what
	# the engine did, and flushes the events its own copies of actors queue up.
	if _is_authority():
		_drain()


# -- loading -----------------------------------------------------------

## Load a conversation document and answer with the id the engine filed it
## under, or NO_NAME if it did not load.
##
## Loading is not a step in the flow: a peer that is told a conversation started
## loads it here like anyone else, which is what keeps the two interning the same
## names in the same order.
func load_conversation_file(file_path) -> int:
	var conversation: int = context.make_name(file_path)
	if !_load_conversation(conversation, file_path):
		return NO_NAME
	return conversation


## Load the speaker config, which the engine holds once for every conversation
## rather than once per document.
func load_conversation_config_file(file_path) -> int:
	var file := FileAccess.open(file_path, FileAccess.READ)
	if FileAccess.get_open_error() != OK:
		push_error("Failed to open config %s." % file_path)
		return -1

	var failure: int = context.load_conversation_config(file.get_as_text())
	if failure != LOAD_OK:
		push_error("Failed to load config %s: %s" % [file_path, LouderLoader.error_name(failure)])
		return -1
	return 0


# -- starting and joining ----------------------------------------------

## Start a conversation document and announce it, `source` being the node that
## asked, which is how the nodes that wanted it tell each other apart.
##
## Called over RPC, so that the peer asking for a conversation is never the peer
## that runs it. call_local is what lets a peer ask for itself: with call_remote
## a peer addressed by its own id is refused outright, and a lone peer with no
## lobby is addressing itself.
@rpc("any_peer", "call_local", "reliable")
func start_conversation_rpc(document_path: String, source: NodePath) -> void:
	# The id is settled by whoever answers the call and handed to the others
	# rather than made again on each peer, so that this is one conversation and
	# not one per peer's interning order.
	var conversation: int = context.make_name(document_path)
	_started(conversation, document_path, source)
	_started.rpc(conversation, document_path, source)


## Put an actor into a conversation on the hub carrying any of `start_tags`, and
## announce both.
##
## The tags and the speakers cross the wire as themselves rather than as interned
## ids, since the peer that answers the call is the one holding the ids.
@rpc("any_peer", "call_local", "reliable")
func connect_actor(conversation: int, start_tags: Array, flags: int, configured_speakers: Array, configured_listen_to: Array = []) -> int:
	var start: int = context.next_hub_from_tags(conversation, _intern(start_tags), false, BLANK_POSITION)
	if start == BLANK_POSITION:
		push_error("Conversation %d has no hub tagged %s." % [conversation, start_tags])
		return NO_ACTOR

	var actor: int = _add_actor(conversation, start, flags, configured_speakers, configured_listen_to)
	if actor == NO_ACTOR:
		return NO_ACTOR

	# Adding an actor drops it on the start hub without saying anything, and that
	# hub is the first thing anyone wants to see.
	announce_position(conversation, actor)
	_drain()
	return actor


## Take an actor out of a conversation.
@rpc("any_peer", "call_local", "reliable")
func remove_actor(conversation: int, actor: int) -> void:
	if !context.remove_actor(conversation, actor):
		return
	_drain()


## Put the actor back where it already stands.
##
## Adding an actor places it without saying anything, so this is how the engine
## is asked to introduce the hub the conversation opens on.
func announce_position(conversation: int, actor: int) -> bool:
	var position: int = context.get_actor_position(conversation, actor)
	if position == BLANK_POSITION:
		return false
	return context.force_position(conversation, actor, position, NO_NAME, 0)


# -- asking about a conversation ----------------------------------------

## Where an actor stands, as a hub index.
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
	return resolve_name(context.get_hub_choice_text(conversation, position, actor, choice_index))


## The speaker of a hub, already as text. Empty when the hub has no speaker,
## which is how the engine says a hub is narration rather than something said.
func get_line_speaker(conversation: int, position: int) -> String:
	return resolve_name(context.get_hub_speaker(conversation, position))


## The text of a hub, already as text.
func get_line_text(conversation: int, position: int) -> String:
	return resolve_name(context.get_hub_text(conversation, position))


## What an interned name stands for, or an empty string when there is none.
func resolve_name(name: int) -> String:
	return "" if name == NO_NAME else LouderNameContent.resolve(context, name)


## Take an engine choice index from get_available_choice_indices.
##
## Only the server runs the engine, so a peer asks for the choice rather than
## taking it, and what it is told is that the request went out, not that the
## engine took it. A choice that was taken arrives as the motion it produces; a
## choice the engine refused arrives as nothing at all.
func select_choice(conversation: int, actor: int, choice_index: int) -> bool:
	if _is_authority():
		return _take_choice(conversation, actor, choice_index)
	_take_choice.rpc_id(SERVER_ID, conversation, actor, choice_index)
	return true


@rpc("any_peer", "call_remote", "reliable")
func _take_choice(conversation: int, actor: int, choice_index: int) -> bool:
	if !context.select_choice(conversation, actor, choice_index):
		push_error("The engine refused choice %d at hub %d." % [choice_index, context.get_actor_position(conversation, actor)])
		return false
	_drain()
	return true


# -- what the engine did ------------------------------------------------

## Read the engine's queue out in one go and pass every event on.
##
## Everything is read before anything is emitted, so a listener is free to call
## back into this singleton without the drain it is interrupting swallowing the
## answer. The order is the order the engine queued them in, which is what puts
## an actor being added ahead of the motion that follows it.
func _drain() -> void:
	var queued: Array[LouderEvent] = []
	var index := 0
	while index < context.get_event_count():
		var event: LouderEvent = context.get_event(index)
		if event == null:
			break
		queued.append(event)
		index += 1
	context.flush_events()

	for event in queued:
		match event.get_kind():
			_EVENT_REGULAR_MOTION, _EVENT_WARP, _EVENT_FORCE_MOTION:
				var travel: Motion = event.get_motion()
				_pass_motion(event.get_conversation(), event.get_actor(),
					travel.get_target_position(), travel.is_background())
			_EVENT_CALLBACK:
				_pass_callback(event.get_name())
			_EVENT_ACTOR_ADDED:
				_pass_actor_added(event.get_conversation(), event.get_actor())
			_EVENT_ACTOR_REMOVED:
				_pass_actor_removed(event.get_conversation(), event.get_actor())
			_:
				pass


@rpc("authority", "call_remote", "reliable")
func _started(conversation: int, document_path: String, source: NodePath) -> void:
	if !_load_conversation(conversation, document_path):
		return
	# Every peer read the document itself, so every peer is the one that knows
	# which records in it could not be read.
	_report_load_failures()
	conversation_started.emit(conversation, source)


## Copy of an actor added on the server, as this peer's own actors are added.
##
## The copy answers what the original would answer on a peer that is not running
## the engine: which speakers it speaks as, and which choices it may take where
## it stands. Ids are the engine's own and are only meaningful to a peer holding
## the same actors in the same order, so the id the copy came out as is checked
## against the one it was told to be rather than assumed.
@rpc("authority", "call_remote", "reliable")
func _actor_added(conversation: int, actor: int) -> void:
	var original: Actor = context.get_actor(conversation, actor)
	if original == null:
		push_error("No actor %d in conversation %d to copy." % [actor, conversation])
		return

	# speak_as and listen_to are already ids and go across as they are, which is
	# also why this cannot go through _add_actor: that interns, and they are.
	var copy: int = context.add_actor(conversation, original.get_position(), original.get_flags(),
		original.speak_as(), original.listen_to())

	# A copy queues the same event the original did, and nothing on this peer is
	# reading it: this peer was told, it did not work it out.
	context.flush_events()

	if copy != actor:
		push_error("Copy of actor %d in conversation %d came out as %d. Peers only agree on actor ids while every peer adds actors in the same order." % [actor, conversation, copy])
		return

	actor_added.emit(conversation, actor)


@rpc("authority", "call_remote", "reliable")
func _actor_removed(conversation: int, actor: int) -> void:
	# The copy goes when the actor it stands in for goes, so this peer's context
	# is left holding only the actors that are still in the conversation. Nothing
	# is said about a copy that was never made: a peer that joined late never had
	# one, and that is not a failure.
	context.remove_actor(conversation, actor)
	context.flush_events()
	actor_removed.emit(conversation, actor)


@rpc("authority", "call_remote", "reliable")
func _callback(callback: int) -> void:
	callback_raised.emit(resolve_name(callback))


@rpc("authority", "call_remote", "reliable")
func _motion(conversation: int, actor: int, target: int, background: int) -> void:
	# Only a peer that is not running the engine gets here, so the actor is this
	# peer's copy of one the server owns, and the copy is still standing where it
	# stood when it was copied. Putting it where the server put it is what keeps
	# the choices this peer offers the choices of the hub the actor is on.
	_follow_motion(conversation, actor, target)
	motion.emit(conversation, actor, target, background)


func _pass_motion(conversation: int, actor: int, target: int, background: bool) -> void:
	# The flag travels as an int: a listener reads it as `!= 0`, which is what it
	# has to do to read one that crossed a network boundary anyway.
	var background_flag := 1 if background else 0
	motion.emit(conversation, actor, target, background_flag)
	_motion.rpc(conversation, actor, target, background_flag)


func _pass_actor_added(conversation: int, actor: int) -> void:
	actor_added.emit(conversation, actor)
	_actor_added.rpc(conversation, actor)


func _pass_actor_removed(conversation: int, actor: int) -> void:
	actor_removed.emit(conversation, actor)
	_actor_removed.rpc(conversation, actor)


## The engine's callbacks are its hook for game state, and the game state is the
## server's. A peer is told which callback was raised rather than being left to
## work out that it missed one.
func _pass_callback(callback: int) -> void:
	callback_raised.emit(resolve_name(callback))
	_callback.rpc(callback)


## Move this peer's copy of an actor to where the server moved it.
##
## force_position is the only thing in the engine that puts an actor somewhere,
## and it queues a motion for having done it, so the queue is emptied again
## right after: this peer is keeping up, not deciding.
func _follow_motion(conversation: int, actor: int, target: int) -> void:
	if !context.force_position(conversation, actor, target, NO_NAME, 0):
		push_error("Copy of actor %d in conversation %d could not follow to hub %d." % [actor, conversation, target])
	context.flush_events()


## Say what in the queue was a document that could not be read.
##
## One unreadable record costs the caller that record and not the conversation,
## so the engine reports it through the queue instead of by failing the load.
## Swallowing those events would leave a document quietly missing hubs. It is
## read on the peer that loaded the document, which is the only peer that knows
## which one it was.
func _report_load_failures() -> void:
	var index := 0
	while index < context.get_event_count():
		var event: LouderEvent = context.get_event(index)
		if event != null and event.get_kind() == _EVENT_LOAD_ERROR:
			var failure: LoadFailure = event.get_load_failure()
			if failure != null:
				push_error("A loaded conversation has an unreadable record at hub %d: %s." % [
					failure.get_position(), LouderLoader.error_name(failure.get_error())])
		index += 1
	context.flush_events()


# -- the engine's own vocabulary ----------------------------------------

## Whether this peer is the one running the engine.
##
## With no lobby there is still a multiplayer peer, the offline one, and it
## answers as the server: a lone process is a server that happens to have nobody
## to talk to.
func _is_authority() -> bool:
	return multiplayer.is_server()


func _load_conversation(conversation: int, document_path: String) -> bool:
	var file := FileAccess.open(document_path, FileAccess.READ)
	if FileAccess.get_open_error() != OK:
		push_error("Failed to open conversation %s." % document_path)
		return false

	var failure: int = context.load_conversation(conversation, file.get_as_text())
	if failure != LOAD_OK and failure != LOAD_ALREADY_LOADED:
		push_error("Failed to load conversation %s: %s" % [document_path, LouderLoader.error_name(failure)])
		return false
	return true


func _add_actor(conversation: int, start: int, flags: int, configured_speakers: Array, configured_listen_to: Array = []) -> int:
	return context.add_actor(conversation, start, flags, _intern(configured_speakers), _intern(configured_listen_to))


## Hand a list of names to the engine as the interned ids it wants.
##
## The lists are taken untyped on purpose: they go straight into the engine,
## which wants a typed array of ints, and neither a GDScript literal nor
## Array.map will hand one over.
func _intern(names: Array) -> Array[int]:
	var interned: Array[int] = []
	for name in names:
		interned.append(context.make_name(name))
	return interned
