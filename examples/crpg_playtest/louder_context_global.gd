extends Node

## The autoload that holds the engine, runs it, and tells everyone what it did.
##
## A LouderContext is a conversation in progress, and running one is a thing only
## one peer may do: two peers running the same actor against the same choices
## come apart the moment the engine decides anything. So this singleton is
## server-authoritative. The server loads conversations, owns the actors and
## takes their choices. Every other peer only recieves events.
##
## The engine names things with integers it hands out as it interns them, so an id
## on one peer means the same as that id on another only if both peers read the
## same documents in the same order. A conversation is therefore loaded on every
## peer instead of being shipped across, and actors are added in the order the
## server hands them out, which is what keeps them numbered the same everywhere.
##
## What a listener needs arrives as a signal, containing data from events in the engine
##
## A lone peer with no lobby is a server. Godot still has its offline multiplayer
## peer in place, and that peer answers as id 1, so the playtest and the headless
## tests go down the same path a hosted session does.

signal conversation_started(conversation: int, source: NodePath, source_character:String)
signal actor_added(conversation: int, actor: int, speak_as: Array)
signal actor_removed(conversation: int, actor: int)
## A move, as the engine announced it.
##
## The speaker is the voice the hub was heard in for that move, which is not
## always the voice the hub carries: a hub without a speaker of its own speaks
## in the voice of whoever walked onto it. It travels as text because that is
## what a line needs, and because a name only means something on the peer that
## interned it.
##
## The kind is what sort of move it was: the peer chose, the peer was carried
## along by somebody else's choice, or the peer was put somewhere. A peer that
## was carried along has not chosen anything, which is what tells one apart
## from the other.
signal motion(conversation: int, actor: int, target: int, background: int, speaker: String, kind: int)
signal event_received(event: Dictionary)
signal callback_raised(callback: String)
signal interrupt_raised(rule: String, identifier: String, conversation: int, actor: int, hub: int)
signal ask_raised(rule: String, ask: String, conversation: int, actor: int, hub: int)
signal identifier_to_resolve(rule: String, identifier: String, conversation: int, actor: int, hub: int)

const SERVER_ID := 1

## The speaker tag this demo uses to mark the characters a player may control.
## The tag is not special to the engine; it is read out of the speaker config and
## every speaker carrying it gets an actor when a conversation starts.
const PLAYER_CHARACTER_TAG := "player_character"

# Where a player character's actor enters a conversation.
const PLAYER_CHARACTER_START_TAGS := ["start"]

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

# LouderContext.get_hub_choice_rule returns the engine's ChoiceRule discriminant.
const _CHOICE_RULE_NAMES := ["standard", "skill", "spoke", "debug", "invisible", "held"]

## The engine. Every question about a conversation is answered out of it, and on
## every peer it is that peer's own context rather than the server's.
var context := LouderContext.new()

## The speakers the loaded config tagged as player characters, in config order.
## Starting a conversation creates one actor for each of them, so that a
## conversation always holds the same actors regardless of which interfaces are
## open or which peers have asked to join.
var player_character_speakers: Array[String] = []


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
##
## Every peer loads it, not just the server: the config decides the order the
## engine interns speaker names, and that order has to be the same everywhere
## for the ids in relayed events to mean the same thing.
func load_conversation_config_file(file_path) -> int:
	var file := FileAccess.open(file_path, FileAccess.READ)
	if FileAccess.get_open_error() != OK:
		push_error("Failed to open config %s." % file_path)
		return -1

	var text := file.get_as_text()
	var failure: int = context.load_conversation_config(text)
	if failure != LOAD_OK:
		push_error("Failed to load config %s: %s" % [file_path, LouderLoader.error_name(failure)])
		return -1

	player_character_speakers = _read_player_character_speakers(text)
	return 0


## The speakers carrying `player_character` in a config document, in file order.
##
## The config is read here rather than asked of the engine because the engine
## has no query for "every speaker with a tag": it can only answer that question
## once a speaker is named.
func _read_player_character_speakers(text: String) -> Array[String]:
	var speakers: Array[String] = []
	var records = JSON.parse_string(text)
	if not records is Array:
		return speakers

	for record in records:
		if not record is Dictionary:
			continue
		if str(record.get("type", "")) != "speaker":
			continue
		var tags := str(record.get("tags", "")).replace(",", " ").replace("\t", " ").replace("\n", " ").split(" ", false)
		if PLAYER_CHARACTER_TAG not in tags:
			continue
		var speaker := str(record.get("name", ""))
		if speaker != "":
			speakers.append(speaker)
	return speakers



# -- starting and joining ----------------------------------------------

## Ask the server to start a conversation.
##
## The player's interface calls this, in practice through
## `ScrollingDialogue.start_conversation`. Nothing else is needed afterwards:
## the server creates an actor for every player character as part of starting
## the conversation, so the caller's actor is already there when the
## conversation is announced.
func start_conversation(document_path: String, source: NodePath, source_character: String) -> void:
	if _is_authority():
		start_conversation_rpc(document_path, source, source_character)
	else:
		start_conversation_rpc.rpc_id(SERVER_ID, document_path, source, source_character)


## Start a conversation document and announce it, `source` being the node that
## asked, which is how the nodes that wanted it tell each other apart.
##
## Called over RPC, so that the peer asking for a conversation is never the peer
## that runs it. call_local is what lets a peer ask for itself: with call_remote
## a peer addressed by its own id is refused outright, and a lone peer with no
## lobby is addressing itself.
@rpc("any_peer", "call_local", "reliable")
func start_conversation_rpc(document_path: String, source: NodePath, source_character:String) -> void:
	# Only the host processes dialogue logic: a client asking for a conversation
	# is a request, and the host is the one that creates it and adds its actors.
	if !multiplayer.is_server():
		return

	# The id is settled by whoever answers the call and handed to the others
	# rather than made again on each peer, so that this is one conversation and
	# not one per peer's interning order.
	var conversation: int = context.make_name(document_path)
	if !_load_conversation(conversation, document_path):
		return
	_report_load_failures()

	# Tell the other peers which document to load before any actor copies are
	# relayed to them, so their contexts know the conversation first.
	_started.rpc(conversation, document_path, source, source_character)

	# Every player character gets an actor now, whether or not a UI has joined.
	# This is the whole of a UI's setup: it asked for the conversation and its
	# actor exists before the conversation is announced to it.
	_add_player_character_actors(conversation, source_character)

	conversation_started.emit(conversation, source, source_character)
	# Announce the actors and their opening hubs afterwards. By then the UI has
	# been told which conversation it asked for, so it can claim its actor.
	_drain()


## Give every player character in the config an actor on the conversation's
## opening hub, adding only the ones that are missing.
##
## Adding an actor already puts it on the hub, so the only one that needs to be
## announced is the actor of the peer that asked: that announcement is what puts
## the opening line on its screen. Announcing every actor would replay every one
## of their opening motions and leave the requesting UI holding several copies of
## the same line.
func _add_player_character_actors(conversation: int, source_character: String) -> void:
	var start: int = context.next_hub_from_tags(conversation, _intern(PLAYER_CHARACTER_START_TAGS), false, BLANK_POSITION)
	if start == BLANK_POSITION:
		push_error("Conversation %d has no hub tagged %s." % [conversation, PLAYER_CHARACTER_START_TAGS])
		return

	for speaker in player_character_speakers:
		if get_actor_for_speaker(conversation, speaker) != NO_ACTOR:
			continue
		_add_actor(conversation, start, _actor_flags_for(speaker), [speaker], [])

	if source_character == "":
		return
	var source_actor := get_actor_for_speaker(conversation, source_character)
	if source_actor != NO_ACTOR:
		announce_position(conversation, source_actor)


## The actor flags for a player character, taken from its character data when
## the catalog knows it. The speaker flags in the config are about visibility,
## not about how an actor takes part, so they are not used here.
func _actor_flags_for(speaker: String) -> int:
	for character in Network.get_characters():
		if str(character.name) == speaker:
			return character.get_actor_flags()
	return 0



## Put an actor into a conversation on the hub carrying any of `start_tags`, and
## announce both.
##
## This is the low-level way in, for tools, tests, and actors that are not
## player characters. A conversation started through [method start_conversation]
## already holds an actor for every player character, so an interface never
## needs this: its actor is there before the conversation is announced.
##
## The tags and the speakers cross the wire as themselves rather than as interned
## ids, since the peer that answers the call is the one holding the ids.
@rpc("any_peer", "call_local", "reliable")
func connect_actor(conversation: int, start_tags: Array, flags: int, configured_speakers: Array, configured_listen_to: Array = []) -> int:
	if !multiplayer.is_server():
		return NO_ACTOR
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


## The actor speaking as `speaker` in a conversation, or NO_ACTOR.
##
## The server keeps actor ids to itself: the interface knows which speaker it
## plays and asks for the actor by name. The same lookup runs on every peer,
## where the relayed copies make the answer agree.
func get_actor_for_speaker(conversation: int, speaker: String) -> int:
	if speaker == "":
		return NO_ACTOR
	return context.next_actor_speaking_as(conversation, context.make_name(StringName(speaker)), NO_ACTOR)


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


## The rule that made a choice available, as the engine's lower-case rule name.
func get_choice_rule(conversation: int, position: int, actor: int, choice_index: int) -> String:
	return context.get_hub_choice_rule_name(conversation, position, actor, choice_index)


## The speaker of a hub, already as text, as the actor in hand would hear it.
##
## A hub with no speaker of its own is anonymous: it speaks in the voice of
## whoever is standing on it, so the answer depends on the actor as much as on
## the hub. Empty when even that voice is missing, which is how the engine says
## a hub is narration rather than something said.
func get_line_speaker(conversation: int, position: int, actor: int) -> String:
	return resolve_name(context.get_hub_speaker_for_actor(conversation, position, actor))


## The text of a hub, already as text.
func get_line_text(conversation: int, position: int) -> String:
	return resolve_name(context.get_hub_text(conversation, position))


## What an interned name stands for, or an empty string when there is none.
func resolve_name(name: int) -> String:
	return "" if name == NO_NAME else LouderNameContent.resolve(context, name)


## Take an engine choice index from get_available_choice_indices, as the speaker
## whose interface is asking.
##
## Only the server runs the engine, so a peer asks for the choice rather than
## taking it, and what it is told is that the request went out, not that the
## engine took it. A choice that was taken arrives as the motion it produces; a
## choice the engine refused arrives as nothing at all.
##
## The peer sends the speaker it plays, not an actor id. Actor ids are the
## server's bookkeeping and differ nowhere only because every peer adds actors
## in the same order; the speaker is what the interface actually knows.
func select_choice(conversation: int, speaker: String, choice_index: int) -> bool:
	if _is_authority():
		return _take_choice(conversation, speaker, choice_index)
	_take_choice.rpc_id(SERVER_ID, conversation, speaker, choice_index)
	return true


@rpc("any_peer", "call_remote", "reliable")
func _take_choice(conversation: int, speaker: String, choice_index: int) -> bool:
	var actor := get_actor_for_speaker(conversation, speaker)
	if actor == NO_ACTOR:
		push_error("No actor speaks as '%s' in conversation %d." % [speaker, conversation])
		return false
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
				_pass_motion(event)
			_EVENT_CALLBACK:
				_pass_callback(event.get_name())
			_EVENT_ACTOR_ADDED:
				_pass_actor_added(event.get_conversation(), event.get_actor())
			_EVENT_ACTOR_REMOVED:
				_pass_actor_removed(event.get_conversation(), event.get_actor())
			_:
				pass

	# The engine interrupts evaluation to ask the host for an identifier's value.
	# Resolve every outstanding ask with the placeholder "0" and tell listeners
	# which rule raised it and where it came from.
	while true:
		var pending_ask := context.get_ask(0)
		if pending_ask == null:
			break
		var ask_rule := pending_ask.get_rule_name()
		var ask_identifier := resolve_name(pending_ask.get_ask())
		var ask_conversation := pending_ask.get_conversation()
		var ask_actor := pending_ask.get_actor()
		var ask_hub := pending_ask.get_motion().get_target_position()
		if not context.answer_ask(0, 0):
			break
		interrupt_raised.emit(ask_rule, ask_identifier, ask_conversation, ask_actor, ask_hub)
		ask_raised.emit(ask_rule, ask_identifier, ask_conversation, ask_actor, ask_hub)
		identifier_to_resolve.emit(ask_rule, ask_identifier, ask_conversation, ask_actor, ask_hub)
		_interrupt.rpc(ask_rule, ask_identifier, ask_conversation, ask_actor, ask_hub)


@rpc("authority", "call_remote", "reliable")
func _started(conversation: int, document_path: String, source: NodePath, source_character:String) -> void:
	if !_load_conversation(conversation, document_path):
		return
	_report_load_failures()
	conversation_started.emit(conversation, source, source_character)


## Relay an actor added on the server.
##
## The copy is added here, in the order the server handed actors out, so that the
## peer's own context can answer the UI's questions and so that later choices
## asked for by speaker resolve to the same actor.
@rpc("authority", "call_remote", "reliable")
func _actor_added(conversation: int, actor: int, position: int, flags: int, speak_as: Array[int], listen_to: Array[int]) -> void:
	if context.get_actor(conversation, actor) == null:
		context.add_actor(conversation, position, flags, speak_as, listen_to)
		# The copy's own actor-added event is bookkeeping; the event that
		# matters has already been relayed from the server.
		context.flush_events()
	actor_added.emit(conversation, actor, speak_as)


@rpc("authority", "call_remote", "reliable")
func _actor_removed(conversation: int, actor: int) -> void:
	if context.get_actor(conversation, actor) != null:
		context.remove_actor(conversation, actor)
		context.flush_events()
	actor_removed.emit(conversation, actor)


@rpc("authority", "call_remote", "reliable")
func _callback(callback: int) -> void:
	callback_raised.emit(resolve_name(callback))


@rpc("authority", "call_remote", "reliable")
func _interrupt(rule: String, identifier: String, conversation: int, actor: int, hub: int) -> void:
	interrupt_raised.emit(rule, identifier, conversation, actor, hub)
	ask_raised.emit(rule, identifier, conversation, actor, hub)
	identifier_to_resolve.emit(rule, identifier, conversation, actor, hub)


@rpc("authority", "call_remote", "reliable")
func _motion(event_dict: Dictionary) -> void:
	# The event arrived as a whole, so unpack it here rather than as separate
	# parameters. The rule that produced it is in event_dict["kind"].
	var conversation := int(event_dict["conversation"])
	var actor := int(event_dict["actor"])
	var motion_dict: Dictionary = event_dict["motion"]
	var target := int(motion_dict["target_position"])
	var background := bool(motion_dict["background"])
	var speaker := resolve_name(int(motion_dict["speaker"]))
	var kind := int(event_dict["kind"])

	# Keep this peer's copy where the server put it. The move is replayed as a
	# forced move rather than by taking the choice again, so the copy follows
	# the server exactly without this peer running the engine.
	if context.get_actor(conversation, actor) != null:
		if kind == _EVENT_WARP:
			context.force_warp(conversation, actor, target, NO_NAME, 0)
		else:
			context.force_position(conversation, actor, target, NO_NAME, 0)
		context.flush_events()

	motion.emit(conversation, actor, target, 1 if background else 0, speaker, kind)
	event_received.emit(event_dict)


func _pass_motion(event: LouderEvent) -> void:
	var event_dict := event.to_dictionary()
	var motion_dict: Dictionary = event_dict["motion"]
	var background := bool(motion_dict["background"])
	var speaker := resolve_name(int(motion_dict["speaker"]))
	var text := resolve_name(context.get_hub_text(int(event_dict["conversation"]), int(motion_dict["target_position"])))
	motion_dict["speaker_text"] = speaker
	motion_dict["text"] = text
	var choice_indices := get_available_choice_indices(int(event_dict["conversation"]), int(event_dict["actor"]))
	var choices: Array[Dictionary] = []
	for idx in choice_indices:
		choices.append({
			"rule": get_choice_rule(int(event_dict["conversation"]), int(motion_dict["target_position"]), int(event_dict["actor"]), idx),
			"text": get_choice_text(int(event_dict["conversation"]), int(motion_dict["target_position"]), int(event_dict["actor"]), idx),
		})
	motion_dict["choice_indices"] = choice_indices
	motion_dict["choices"] = choices
	var kind := int(event_dict["kind"])
	var background_flag := 1 if background else 0
	motion.emit(int(event_dict["conversation"]), int(event_dict["actor"]), int(motion_dict["target_position"]), background_flag, speaker, kind)
	event_received.emit(event_dict)
	_motion.rpc(event_dict)


func _pass_actor_added(conversation: int, actor: int) -> void:
	var original: Actor = context.get_actor(conversation, actor)
	actor_added.emit(conversation, actor, original.speak_as())
	_actor_added.rpc(conversation, actor, original.get_position(), original.get_flags(),
		original.speak_as(), original.listen_to())


func _pass_actor_removed(conversation: int, actor: int) -> void:
	actor_removed.emit(conversation, actor)
	_actor_removed.rpc(conversation, actor)


## The engine's callbacks are its hook for game state, and the game state is the
## server's. A peer is told which callback was raised rather than being left to
## work out that it missed one.
func _pass_callback(callback: int) -> void:
	callback_raised.emit(resolve_name(callback))
	_callback.rpc(callback)


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

	var text := file.get_as_text()
	var failure: int
	var extension := document_path.get_extension().to_lower()
	if extension == "json" or text.strip_edges().begins_with("[") or text.strip_edges().begins_with("{"):
		failure = context.load_conversation(conversation, text)
	else:
		failure = context.load_conversation_text(conversation, text)
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
