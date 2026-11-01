class_name DialogueSession extends RefCounted

## The multiplayer layer of a conversation.
##
## A peer is a network peer playing a character. The character says which voice
## the peer's actor speaks as, and the engine turns that voice into what the
## peer may see and do. This class keeps the peers of a conversation and the
## actors that stand for them, and reports what the engine did in terms of
## peers rather than actors, so that everything above it can talk about
## players instead of about engine ids.
##
## It draws nothing and knows nothing about the interface: the signals below
## are the whole contract, and every question is answered by asking the
## engine. That is what lets the same session be driven by a test with no
## interface in it at all.
##
## The engine is run by one peer, the authority. It is the authority that
## creates the actors, and every other peer is told about them and keeps a
## copy of its own, so that the copy can answer the questions the peer asks
## about its own character.

## A peer joined the conversation, with the actor that stands for it.
signal peer_joined(peer_id: int, actor: int)

## A line was revealed to a peer: what was said, in whose voice, whether it
## happened behind the peer's back, and how the peer came to be hearing it.
signal line_revealed(peer_id: int, speaker: String, text: String, background: bool, kind: int)

## The choices a peer may take, in the order they are to be shown.
signal choices_revealed(peer_id: int, texts: PackedStringArray)

## The engine's hook for game state. The peer is told which callback was
## raised rather than being left to work out that it missed one.
signal callback_raised(callback: String)

## How a peer came to be hearing a line. These are the engine's own numbers:
## the peer chose, the peer was carried along by somebody else's choice, and
## the peer was put somewhere.
const MOVE_CHOSE := 1
const MOVE_WARPED := 2
const MOVE_PLACED := 3

## The engine's blanks, which are all minus one.
const NO_NAME := -1
const NO_ACTOR := -1

## The conversation this session is about.
var conversation := -1

## The tag the conversation opens on.
var start_tag := &"start"

# peer id -> CharacterData
var _peers: Dictionary = {}
# peer id -> actor id
var _actors: Dictionary = {}
# actor id -> peer id
var _peer_of_actor: Dictionary = {}


func _init() -> void:
	# The session listens to the multiplayer layer rather than to the engine
	# directly, so that the same session works on the peer that runs the engine
	# and on the peers that were told what it did.
	LouderContextGlobal.actor_added.connect(_on_actor_added)
	LouderContextGlobal.motion.connect(_on_motion)
	LouderContextGlobal.callback_raised.connect(_on_callback_raised)


## Put a peer into a conversation, speaking as the voice its character names.
##
## Returns the actor that stands for the peer. On the peer that runs the
## engine that is the actor itself; on every other peer it is the blank,
## because the actor is created by the authority and arrives later, as a copy.
func join(conversation_id: int, peer_id: int, character: CharacterData) -> int:
	conversation = conversation_id
	_peers[peer_id] = character

	if !_is_authority():
		return NO_ACTOR

	# A started conversation already has an actor for every player character, so
	# joining is looking the peer's actor up rather than adding a second one.
	# The connect call is only for conversations that were loaded without being
	# started, which is how the engine-level tests drive one.
	var speaker := get_speaker(peer_id)
	var actor: int = LouderContextGlobal.get_actor_for_speaker(conversation_id, speaker)
	if actor == NO_ACTOR:
		actor = LouderContextGlobal.connect_actor(
			conversation_id, [start_tag], character.get_actor_flags(), character.get_speaker_names())
	_bind(peer_id, actor)
	peer_joined.emit(peer_id, actor)
	return actor


## The actor standing for a peer, or the blank when the peer has none.
func get_actor(peer_id: int) -> int:
	return _actors.get(peer_id, NO_ACTOR)


## The voice a peer's actor speaks as, or an empty string when the peer speaks
## as nobody.
func get_speaker(peer_id: int) -> String:
	var character: CharacterData = _peers.get(peer_id)
	if character == null:
		return ""
	var speakers := character.get_speaker_names()
	return "" if speakers.is_empty() else String(speakers[0])


## Where a peer's actor is standing.
func get_position(peer_id: int) -> int:
	var actor := get_actor(peer_id)
	if actor == NO_ACTOR:
		return -1
	return LouderContextGlobal.get_actor_position(conversation, actor)


## The engine's choice indices a peer may take where it stands.
##
## These are indices on the hub itself and they skip the choices the peer may
## not take, so they are not the rows a peer is shown. Read them with
## [method get_choice_texts], which lines the texts up with them.
func get_choice_indices(peer_id: int) -> Array[int]:
	var actor := get_actor(peer_id)
	if actor == NO_ACTOR:
		return []
	return LouderContextGlobal.get_available_choice_indices(conversation, actor)


## The text of each choice a peer may take, in the order the peer is shown
## them.
func get_choice_texts(peer_id: int) -> PackedStringArray:
	var actor := get_actor(peer_id)
	if actor == NO_ACTOR:
		return PackedStringArray()

	var position := LouderContextGlobal.get_actor_position(conversation, actor)
	var texts := PackedStringArray()
	for index in get_choice_indices(peer_id):
		texts.append(LouderContextGlobal.get_choice_text(conversation, position, actor, index))
	return texts


## Take the choice shown in row `row` of what the peer was last offered.
##
## Rows are positions in the list the peer was shown rather than engine choice
## indices, so the row is looked up here and the engine is handed the index it
## knows.
func select_choice(peer_id: int, row: int) -> bool:
	var actor := get_actor(peer_id)
	if actor == NO_ACTOR:
		return false

	var indices := get_choice_indices(peer_id)
	if row < 0 or row >= indices.size():
		push_error("Peer %d has no dialogue choice in row %d." % [peer_id, row])
		return false

	return LouderContextGlobal.select_choice(conversation, get_speaker(peer_id), indices[row])


## The engine's choice index behind a row, which is what the engine is handed.
func get_choice_index(peer_id: int, row: int) -> int:
	var indices := get_choice_indices(peer_id)
	if row < 0 or row >= indices.size():
		return -1
	return indices[row]


# -- what the engine did ------------------------------------------------

## An actor was added somewhere, which may be this session's peer.
func _on_actor_added(conversation_id: int, actor: int, _speak_as: Array) -> void:
	if conversation_id != conversation:
		return

	# The authority creates the actors and every other peer copies them, so a
	# peer that was not the one asking is told about an actor it has never seen.
	# Which peer it stands for is found by the voice it speaks as, since that is
	# what the character named and every peer speaks as exactly one voice.
	var spoken_as := _speaker_of_actor(conversation_id, actor)
	for peer_id in _peers:
		if _actors.has(peer_id):
			continue
		if get_speaker(peer_id) == spoken_as:
			_bind(peer_id, actor)
			peer_joined.emit(peer_id, actor)
			return


## A peer moved, which is a line for whoever was moved.
func _on_motion(conversation_id: int, actor: int, target: int, background: int, speaker: String, kind: int) -> void:
	if conversation_id != conversation:
		return

	var peer_id: int = _peer_of_actor.get(actor, NO_ACTOR)
	if peer_id == NO_ACTOR:
		return

	var text := LouderContextGlobal.get_line_text(conversation_id, target)
	line_revealed.emit(peer_id, speaker, text, background != 0, kind)
	_announce_choices(peer_id)


## The engine raised a callback, which every peer is told about.
func _on_callback_raised(callback: String) -> void:
	callback_raised.emit(callback)


## Tell a peer what it may choose where it stands.
##
## Emitted whether or not there are choices, since an empty list is how a peer
## is told that there is nothing to pick here.
func _announce_choices(peer_id: int) -> void:
	choices_revealed.emit(peer_id, get_choice_texts(peer_id))


# -- helpers -----------------------------------------------------------

func _bind(peer_id: int, actor: int) -> void:
	_actors[peer_id] = actor
	_peer_of_actor[actor] = peer_id


## The voice an actor speaks as, as text.
func _speaker_of_actor(conversation_id: int, actor: int) -> String:
	var entry: Actor = LouderContextGlobal.context.get_actor(conversation_id, actor)
	if entry == null:
		return ""
	var speak_as := entry.speak_as()
	if speak_as.is_empty():
		return ""
	return LouderContextGlobal.resolve_name(speak_as[0])


## Whether this peer is the one running the engine.
##
## A lone peer with no lobby is still the authority: Godot's offline peer
## answers as the server, so the playtest and the headless tests go down the
## same path a hosted session does.
func _is_authority() -> bool:
	return LouderContextGlobal.multiplayer.is_server()
