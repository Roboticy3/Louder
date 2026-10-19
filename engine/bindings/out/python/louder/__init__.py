# Bindings for Python, generated from the C header of the Louder engine.
#
# Do not edit. Run `python engine/bindings/generate.py` and this file comes back.
"""
The Louder engine, from Python.

A `Louder` owns one engine context and everything that comes with it:
the name table, every conversation, and the event queue. It is released
when it is closed, or when it goes out of scope as a context manager.

    with Louder() as engine:
        engine.load_config(document)
        engine.load_conversation('chat', document)
        actor = engine.add_actor('chat', 0, speak_as=['A'])
        engine.choose('chat', actor, 0)
        for event in engine.drain_events():
            print(event.kind, event.value)

Strings cross the boundary as bytes and are decoded here as UTF-8, which
is the encoding the engine reads JSON in. Anything else is handed over
untouched, as `bytes`.
"""

from __future__ import annotations

import ctypes

from dataclasses import dataclass
from enum import IntEnum, IntFlag
from typing import Any, Iterator, Sequence

from . import _ffi
from ._ffi import (
    LOUDER_ACTOR_NONE,
    LOUDER_LOAD_ALREADY_LOADED,
    LOUDER_LOAD_MALFORMED_JSON,
    LOUDER_LOAD_NOT_AN_OBJECT,
    LOUDER_LOAD_NOT_A_LIST,
    LOUDER_LOAD_OK,
    LOUDER_NAME_NONE,
)

__all__ = [
    'Louder',
    'LouderFailure',
    'LouderError',
    'LoadError',
    'Event',
    'Ask',
    'Motion',
    'ChoiceAppearance',
    'AdditionalDataChange',
    'ConditionOutcome',
    'LoadFailure',
    'NAME_NONE',
    'ACTOR_NONE',
    'BLANK_POSITION',
    'EventData_Tag',
    'LouderError',
    'LouderOption_LouderAsk_Tag',
    'LouderOption_LouderEvent_Tag',
    'LouderVariant_Tag',
    'ActorFlags',
    'DialogueRules',
    'HubFlags',
    'SpeakerFlags',
]


# --- vocabularies -------------------------------------------------

class EventData_Tag(IntEnum):
    """
    What an LouderEvent is about.

    A tagged union cannot be read as one value by a host that has to switch on it, so this is the discriminant of an EventData as an `int`, and the payload is read through the accessors that match it. A host that does not recognise a value is looking at something a newer engine produced.
    """
    NONE_ = 0
    RegularMotion = 1
    Warp = 2
    ForceMotion = 3
    ChoiceAppearance = 4
    ChannelChange = 5
    AdditionalDataChange = 6
    SpeakerChange = 7
    TextChange = 8
    Statement = 9
    Condition = 10
    Callback = 11
    ActorAdded = 12
    ActorRemoved = 13
    SpeakerAdded = 14
    SpeakerRemoved = 15
    ConversationAdded = 16
    ConversationRemoved = 17
    LoadError = 18


class LouderError(IntEnum):
    """
    An error code returned by most operations.

    - LouderError::Ok: no error. - LouderError::Missing: the requested object does not exist.
    """
    Ok = 0
    Missing = 1


class LouderOption_LouderAsk_Tag(IntEnum):
    """Custom Option type exposed to the C FFI"""
    None_LouderAsk = 0
    Some_LouderAsk = 1


class LouderOption_LouderEvent_Tag(IntEnum):
    """Custom Option type exposed to the C FFI"""
    None_LouderEvent = 0
    Some_LouderEvent = 1


class LouderVariant_Tag(IntEnum):
    """
    One unit of data in an actor, hub, speaker, or story state.

    Booleans are not a separate kind: they are carried as LouderVariant::Int 1 or 0, which is what makes them interoperable with the arithmetic and comparison rules of the engine guide.
    """
    Int = 0
    Float = 1
    String = 2


class ActorFlags(IntFlag):
    """The flags of a actor."""
    SPECTATOR = 1
    DEBUG = 2


class DialogueRules(IntFlag):
    """The flags of a DialogueRules."""
    SPOKE = 1
    SINGLEPLAYER = 2


class HubFlags(IntFlag):
    """The flags of a hub."""
    AUTO = 1
    ASK = 2


class SpeakerFlags(IntFlag):
    """The flags of a speaker."""
    PUBLIC = 1


class LoadError(IntEnum):
    """Why a document was refused. Zero means it was not refused."""
    Ok = 0
    MalformedJson = 1
    NotAList = 2
    NotAnObject = 3
    AlreadyLoaded = 4


class LouderActor(IntEnum):
    """Why a document was refused. Zero means it was not refused."""
    ActorNone = -1
    LoadOk = 0
    LoadMalformedJson = 1
    LoadNotAList = 2
    LoadNotAnObject = 3
    LoadAlreadyLoaded = 4


class LouderCallback(IntEnum):
    """Why a document was refused. Zero means it was not refused."""
    Ok = 0
    MalformedJson = 1
    NotAList = 2
    NotAnObject = 3
    AlreadyLoaded = 4


class LouderName(IntEnum):
    """Why a document was refused. Zero means it was not refused."""
    NameNone = -1
    LoadOk = 0
    LoadMalformedJson = 1
    LoadNotAList = 2
    LoadNotAnObject = 3
    LoadAlreadyLoaded = 4


LOUDER_ACTOR_NONE = -1
LOUDER_LOAD_ALREADY_LOADED = 4
LOUDER_LOAD_MALFORMED_JSON = 1
LOUDER_LOAD_NOT_AN_OBJECT = 3
LOUDER_LOAD_NOT_A_LIST = 2
LOUDER_LOAD_OK = 0
LOUDER_NAME_NONE = -1

# --- records -----------------------------------------------------

@dataclass
class ActorFlags:
    """Flag bits describing an actor."""
    bits: int


@dataclass
class AdditionalDataChange:
    """A key of an actor's additional data being replaced."""
    key: int


@dataclass
class ChoiceAppearance:
    """A choice becoming available on a hub."""
    position: int
    choice_index: int


@dataclass
class ConditionOutcome:
    """What a condition on a hub came to, when an actor landed on it."""
    condition: int
    outcome: Any


@dataclass
class DialogueRules:
    """
    The dialogue rules a host can turn on and off.

    Both are on by default, because each is what makes the engine usable without every conversation having to be written around its absence. They are gathered here rather than left as separate switches so that a host reads and writes the whole configuration in one call, and so that adding a rule does not add another pair of engine calls.
    """
    bits: int


@dataclass
class HubFlags:
    """Flag bits describing a hub."""
    bits: int


@dataclass
class LoadFailure:
    """
    A record in a document the engine could not read.

    Carries the position of the record that was at fault, or LOUDER_POSITION_NONE when the problem was with the document as a whole.
    """
    position: int
    error: int


@dataclass
class Motion:
    """
    One move of an actor from one hub to another.

    Every event a move produces carries the same motion, which is how a host reads where an actor came from and where it went.
    """
    source_position: int
    target_position: int
    choice_index: int
    background: int


@dataclass
class SpeakerFlags:
    """Flag bits describing a speaker."""
    bits: int



# --- failures ----------------------------------------------------


class LouderFailure(Exception):
    """Something went wrong that the caller has to know about."""


class LoadFailed(LouderFailure):
    """A document the engine refused, and why."""

    def __init__(self, code: int) -> None:
        self.code = LoadError(code)
        super().__init__(self.code.name)


# --- kinds -------------------------------------------------------

EventData_KINDS: dict[int, str] = {
    0: 'None',
    1: 'RegularMotion',
    2: 'Warp',
    3: 'ForceMotion',
    4: 'ChoiceAppearance',
    5: 'ChannelChange',
    6: 'AdditionalDataChange',
    7: 'SpeakerChange',
    8: 'TextChange',
    9: 'Statement',
    10: 'Condition',
    11: 'Callback',
    12: 'ActorAdded',
    13: 'ActorRemoved',
    14: 'SpeakerAdded',
    15: 'SpeakerRemoved',
    16: 'ConversationAdded',
    17: 'ConversationRemoved',
    18: 'LoadError',
}

LouderOption_LouderAsk_KINDS: dict[int, str] = {
    0: 'None_LouderAsk',
    1: 'Some_LouderAsk',
}

LouderOption_LouderEvent_KINDS: dict[int, str] = {
    0: 'None_LouderEvent',
    1: 'Some_LouderEvent',
}

LouderVariant_KINDS: dict[int, str] = {
    0: 'Int',
    1: 'Float',
    2: 'String',
}



NAME_NONE = LOUDER_NAME_NONE
"""The id of nothing: what a failed lookup, or an unset name, comes back as."""

ACTOR_NONE = LOUDER_ACTOR_NONE
"""The id of no actor, which is what an actor outside the flow reads as."""

BLANK_POSITION = -1
"""The position of no hub, and where a cursor over hubs starts."""


@dataclass
class Event:
    """One thing that happened, with the names in it already decoded."""

    conversation: str
    """The conversation it happened in."""

    channel: str
    """The channel it happened on."""

    actor: int
    """The actor it happened to, or ACTOR_NONE."""

    kind: str
    """Which of the engine kinds of event this is."""

    value: Any
    """Whatever that kind carries, or None where it carries nothing."""


@dataclass
class Ask:
    """A question the engine has asked the host and is waiting on."""

    conversation: str
    channel: str
    actor: int
    ask: str
    """The name of the ask, which is what the host acts on."""


_NAME_BUFFER = 4096
_JSON_BUFFER = 1 << 16


def _bytes(value: Any) -> bytes:
    if isinstance(value, str):
        return value.encode("utf-8")
    if isinstance(value, (bytes, bytearray, memoryview)):
        return bytes(value)
    raise TypeError(type(value).__name__ + " is neither text nor bytes")


def _with(engine: "Louder", value: Any) -> tuple[Any, int]:
    """A buffer and a length, which is how every string crosses the boundary.

    The buffer is an array of the byte type the prototypes declare rather than
    a string buffer: `ctypes` will not quietly pass one where the other is
    wanted, which is the right way round.
    """
    raw = _bytes(value)
    buffer = (ctypes.c_uint8 * max(len(raw), 1))()
    ctypes.memmove(buffer, raw, len(raw))
    return buffer, len(raw)


def _take(call: Any, *args: Any, size: int = _NAME_BUFFER) -> bytes:
    """Call something that fills a buffer, and return what it wrote.

    The engine truncates rather than refusing, and says how much fitted, so the
    only way to be sure of the whole answer is to ask again with more room. Two
    calls is the normal case; the loop is what makes a long document safe.
    """
    while True:
        buffer = (ctypes.c_uint8 * size)()
        written = call(*args, buffer, size - 1)
        if written < 0:
            raise LouderFailure(call.__name__ + " reported " + str(written))
        if written < size - 1:
            return bytes(memoryview(buffer).cast("B")[:written])
        size *= 4


def _name_array(values: Sequence[int]) -> Any:
    """A list of name ids as the array the engine reads."""
    array = (ctypes.c_int32 * max(len(values), 1))()
    for index, value in enumerate(values):
        array[index] = int(value)
    return array


def _bits(type_name: str, value: int) -> Any:
    """A flags value as the struct the engine takes.

    The convenience layer has the flags as an `IntFlag`, which is what a caller
    wants to write; the boundary has them as a struct wrapping the bits. This
    is the one crossing between the two, and it is the only place it happens.
    """
    built = getattr(_ffi, type_name)()
    built.bits = int(value)
    return built


class Louder:
    """One engine context.

    Owns the name table, every conversation loaded into it, and the event queue.
    Closing it releases all of that at once.
    """

    __slots__ = ("_handle", "_closed")

    def __init__(self, library: Any = None, path: str | None = None) -> None:
        self._handle = (library or _ffi.load_library(path)).init_louder()
        if not self._handle:
            raise LouderFailure("the engine could not allocate a context")
        self._closed = False

    def close(self) -> None:
        if not self._closed:
            self._closed = True
            _ffi.deinit_louder(self._handle)

    def __enter__(self) -> "Louder":
        return self

    def __exit__(self, *_: Any) -> None:
        self.close()

    def __del__(self) -> None:
        try:
            self.close()
        except Exception:
            pass

    @property
    def handle(self) -> Any:
        """The raw context pointer, for anything this layer does not wrap."""
        return self._handle

    # --- names -----------------------------------------------------

    def lookup(self, text: str | bytes) -> int:
        """The id of some text, or NAME_NONE if it was never declared."""
        return _ffi.get_named_id(self._handle, *_with(self, text))

    def declare(self, text: str | int) -> int:
        """
        The id of some text, declaring it if this is the first time.

        An id is passed straight back, so a value that came out of one of
        these calls can go into any other without being looked up again.
        """
        if isinstance(text, int):
            return text
        existing = self.lookup(text)
        if existing != NAME_NONE:
            return existing
        return _ffi.declare_name(self._handle, *_with(self, text))

    def name(self, text: str | int) -> int:
        """An alias for declare, for when the text really is a name."""
        return self.declare(text)

    def text(self, name: int) -> str:
        """The text behind an id, decoded as UTF-8."""
        return _take(_ffi.read_name, self._handle, int(name)).decode('utf-8', 'replace')

    def optional_text(self, name: int) -> str | None:
        """The text behind an id, or None when the id names nothing."""
        if int(name) == NAME_NONE:
            return None
        return self.text(name)


    # --- context ----------------------------------------------------

    def rules(self) -> DialogueRules:
        """The dialogue rules in effect."""
        return DialogueRules(_ffi.get_dialogue_rules(self._handle).bits)

    def set_rules(self, rules: int) -> None:
        """Replace the dialogue rules. They take effect on the next move."""
        _ffi.set_dialogue_rules(self._handle, _bits('DialogueRules', rules))


    # --- documents -------------------------------------------------

    def load_conversation(self, conversation: str | int, document: str | bytes) -> LoadError:
        """Load a conversation from a JSON document."""
        name = self.declare(conversation)
        return LoadError(_ffi.load_conversation_json(self._handle, name, *_with(self, document)))

    def load_config(self, document: str | bytes) -> LoadError:
        """Load the speaker config, which is a singleton."""
        return LoadError(_ffi.load_conversation_config(self._handle, *_with(self, document)))

    def clear_config(self) -> bool:
        """Remove every speaker. Conversations are left alone."""
        return _ffi.clear_config(self._handle) == 0

    def remove_conversation(self, conversation: str | int) -> bool:
        """Remove a conversation and the actors in it."""
        return _ffi.remove_conversation(self._handle, self.declare(conversation)) == 0


    # --- hubs ------------------------------------------------------

    def hub_text(self, conversation: str | int, position: int) -> str:
        """The text of a hub."""
        return self.text(_ffi.get_hub_text(self._handle, self.declare(conversation), position))

    def hub_speaker(self, conversation: str | int, position: int) -> str:
        """The speaker of a hub."""
        return self.text(_ffi.get_hub_speaker(self._handle, self.declare(conversation), position))

    def hub_channel(self, conversation: str | int, position: int) -> str:
        """The channel of a hub."""
        return self.text(_ffi.get_hub_channel(self._handle, self.declare(conversation), position))

    def hub_expressions(self, conversation: str | int, position: int) -> str:
        """The expressions of a hub."""
        return self.text(_ffi.get_hub_expressions(self._handle, self.declare(conversation), position))

    def hub_callbacks(self, conversation: str | int, position: int) -> str:
        """The callbacks of a hub."""
        return self.text(_ffi.get_hub_callbacks(self._handle, self.declare(conversation), position))

    def hub_priority(self, conversation: str | int, position: int) -> int:
        """The priority of a hub."""
        return self.text(_ffi.get_hub_priority(self._handle, self.declare(conversation), position))

    def hub_flags(self, conversation: str | int, position: int) -> HubFlags:
        """The flags on a hub."""
        return HubFlags(_ffi.get_hub_flags(self._handle, self.declare(conversation), position).bits)

    def hub_additional(self, conversation: str | int, position: int, key: str | int) -> Any:
        """Additional data on a hub, under a key."""
        return self._variant(
            _ffi.get_hub_additional(
                self._handle, self.declare(conversation), position, self.declare(key)
            )
        )

    def hub_choice_count(self, conversation: str | int, position: int, actor: int) -> int:
        """How many choices are available to an actor on a hub."""
        return _ffi.get_hub_choice_count(self._handle, self.declare(conversation), position, actor)

    def hub_choice_text(self, conversation: str | int, position: int, actor: int, index: int) -> str:
        """The text of one choice, which is the text of the hub it leads to."""
        return self.text(
            _ffi.get_hub_choice_text(self._handle, self.declare(conversation), position, actor, index)
        )

    def hub_choice_link(self, conversation: str | int, position: int, actor: int, index: int) -> int:
        """Where one choice leads."""
        return _ffi.get_hub_choice_link(self._handle, self.declare(conversation), position, actor, index)


    # --- actors -----------------------------------------------------

    def add_actor(
        self,
        conversation: str | int,
        position: int,
        *,
        speak_as: Sequence[str | int] = (),
        listen_to: Sequence[str | int] = (),
        flags: int = 0,
    ) -> int:
        """Put an actor into a conversation and return its id."""
        return _ffi.add_actor(
            self._handle,
            self.declare(conversation),
            position,
            _bits('ActorFlags', flags),
            _name_array([self.declare(s) for s in speak_as]),
            len(speak_as),
            _name_array([self.declare(s) for s in listen_to]),
            len(listen_to),
        )

    def remove_actor(self, conversation: str | int, actor: int) -> bool:
        """Take an actor out of a conversation."""
        return _ffi.remove_actor(self._handle, self.declare(conversation), actor) == 0

    def actor_position(self, conversation: str | int, actor: int) -> int:
        """Where an actor is standing."""
        return _ffi.get_actor_position(self._handle, self.declare(conversation), actor)

    def actor_channel(self, conversation: str | int, actor: int) -> str:
        """Which channel an actor is on."""
        return self.text(_ffi.get_actor_channel(self._handle, self.declare(conversation), actor))

    def actor_flags(self, conversation: str | int, actor: int) -> ActorFlags:
        """The flags on an actor."""
        return ActorFlags(_ffi.get_actor_flags(self._handle, self.declare(conversation), actor).bits)

    def actor_priority(self, conversation: str | int, actor: int) -> int:
        """The priority on an actor."""
        return _ffi.get_actor_priority(self._handle, self.declare(conversation), actor)

    def actor_additional(self, conversation: str | int, actor: int, key: str | int) -> Any:
        """Additional data on an actor, under a key."""
        return self._variant(
            _ffi.get_actor_additional(
                self._handle, self.declare(conversation), actor, self.declare(key)
            )
        )

    def choose(self, conversation: str | int, actor: int, index: int) -> bool:
        """Take a choice on the hub an actor is standing on."""
        return _ffi.select_choice(self._handle, self.declare(conversation), actor, index) == 0

    def warp(
        self,
        conversation: str | int,
        actor: int,
        target: int,
        *,
        channel: str | int | None = None,
        priority: int = 0,
    ) -> bool:
        """Move an actor as though it had walked there."""
        return _ffi.force_warp(
            self._handle,
            self.declare(conversation),
            actor,
            target,
            NAME_NONE if channel is None else self.declare(channel),
            priority,
        ) == 0

    def teleport(
        self,
        conversation: str | int,
        actor: int,
        target: int,
        *,
        channel: str | int | None = None,
        priority: int = 0,
    ) -> bool:
        """Move an actor straight there."""
        return _ffi.force_position(
            self._handle,
            self.declare(conversation),
            actor,
            target,
            NAME_NONE if channel is None else self.declare(channel),
            priority,
        ) == 0


    # --- story state ----------------------------------------------

    def get(self, conversation: str | int, key: str | int) -> Any:
        """A story state value, or a zero variant if it was never written."""
        return self._variant(
            _ffi.get_story_state(self._handle, self.declare(conversation), self.declare(key))
        )

    def set(self, conversation: str | int, key: str | int, value: Any) -> None:
        """Write a story state value."""
        _ffi.set_story_state(
            self._handle, self.declare(conversation), self.declare(key), self._make_variant(value)
        )

    def state_json(self, conversation: str | int) -> str:
        """The whole story state as a JSON document."""
        return _take(_ffi.read_state_json, self._handle, self.declare(conversation)).decode('utf-8')

    def snapshot_json(self, conversation: str | int) -> str:
        """The story state and every actor, as a JSON document."""
        return _take(_ffi.read_snapshot_json, self._handle, self.declare(conversation)).decode('utf-8')

    # --- variants ----------------------------------------------------

    def variant_int(self, value: int) -> Any:
        """Build the variant the engine reads back as an integer."""
        return _ffi.make_variant_int(int(value))

    def variant_float(self, value: float) -> Any:
        """Build the variant the engine reads back as a float."""
        return _ffi.make_variant_float(float(value))

    def variant_bool(self, value: bool) -> Any:
        """Build the variant the engine reads back as an integer."""
        return _ffi.make_variant_bool(bool(value))

    def variant_name(self, text: str | int) -> Any:
        """Build the variant the engine reads back as a name."""
        return _ffi.make_variant_string(self.declare(text))

    def _make_variant(self, value: Any) -> Any:
        """Build the variant standing for an ordinary Python value."""
        if isinstance(value, bool):
            return self.variant_bool(value)
        if isinstance(value, int):
            return self.variant_int(value)
        if isinstance(value, float):
            return self.variant_float(value)
        if isinstance(value, str):
            return self.variant_name(value)
        raise TypeError(
            type(value).__name__ + ' is not a value the engine can hold'
        )

    def _variant(self, variant: Any) -> Any:
        """Read a variant as the Python value it stands for."""
        kind = variant.tag
        if kind == LouderVariant_Tag.Int:
            return variant.int_
        if kind == LouderVariant_Tag.Float:
            return variant.float_
        return self.text(variant.string)


    # --- iteration ---------------------------------------------------

    # Every one of these is a cursor in the C header: a `start`, the thing
    # it returns, and a sentinel for the end of the walk. Here they are
    # generators, so the walk reads as a `for` and the caller cannot forget
    # to advance.

    def iter_actor_from_flags(self, conversation: str | int, include: ActorFlags, exact: int = 0) -> Iterator[int]:
        """
        Iterate the actors matching certain flags.

        With `exact` nonzero the flags must match exactly; otherwise every bit in `include` must be present. Pass LOUDER_ACTOR_NONE as `start` to begin.
        """
        start: Any = ACTOR_NONE
        while True:
            found = _ffi.next_actor_from_flags(self._handle, self.declare(conversation), include, exact, start)
            if found == ACTOR_NONE:
                return
            yield found
            start = found

    def iter_actor_in_channel(self, conversation: str | int, channel: str | int) -> Iterator[int]:
        """
        Iterate the actors in a channel.

        Pass LOUDER_ACTOR_NONE as `start` to begin. Actors outside the flow have no channel and are never returned.
        """
        start: Any = ACTOR_NONE
        while True:
            found = _ffi.next_actor_in_channel(self._handle, self.declare(conversation), self.declare(channel), start)
            if found == ACTOR_NONE:
                return
            yield found
            start = found

    def iter_actor_in_channel_and_position(self, conversation: str | int, channel: str | int, position: int) -> Iterator[int]:
        """
        Iterate the actors in a channel at a specific position.

        Pass LOUDER_ACTOR_NONE as `start` to begin.
        """
        start: Any = ACTOR_NONE
        while True:
            found = _ffi.next_actor_in_channel_and_position(self._handle, self.declare(conversation), self.declare(channel), position, start)
            if found == ACTOR_NONE:
                return
            yield found
            start = found

    def iter_actor_listening(self, conversation: str | int, actor: int) -> Iterator[int]:
        """
        Iterate the speakers an actor listens to.

        Includes the actor's `listen_to` speakers plus the listening speakers of the speakers it speaks as. Pass LOUDER_NAME_NONE as `start` to begin.
        """
        start: Any = NAME_NONE
        while True:
            found = _ffi.next_actor_listening(self._handle, self.declare(conversation), actor, start)
            if found == NAME_NONE:
                return
            yield found
            start = found

    def iter_actor_listening_base(self, conversation: str | int, actor: int) -> Iterator[int]:
        """
        Iterate the base speakers an actor listens to.

        These are the actor's own `listen_to` speakers, without the ones reachable through who it speaks as. See next_actor_listening for the wider set.
        """
        start: Any = NAME_NONE
        while True:
            found = _ffi.next_actor_listening_base(self._handle, self.declare(conversation), actor, start)
            if found == NAME_NONE:
                return
            yield found
            start = found

    def iter_actor_listening_to(self, conversation: str | int, speaker: str | int) -> Iterator[int]:
        """
        Iterate the actors listening to a speaker.

        This covers `listen_to` plus the listening actors derived from the listening speakers of the speakers an actor speaks as. Pass LOUDER_ACTOR_NONE as `start` to begin.
        """
        start: Any = ACTOR_NONE
        while True:
            found = _ffi.next_actor_listening_to(self._handle, self.declare(conversation), self.declare(speaker), start)
            if found == ACTOR_NONE:
                return
            yield found
            start = found

    def iter_actor_speak_as(self, conversation: str | int, actor: int) -> Iterator[int]:
        """
        Iterate the speakers an actor speaks as.

        Pass LOUDER_NAME_NONE as `start` to begin, then pass the previous result until LOUDER_NAME_NONE comes back.
        """
        start: Any = NAME_NONE
        while True:
            found = _ffi.next_actor_speak_as(self._handle, self.declare(conversation), actor, start)
            if found == NAME_NONE:
                return
            yield found
            start = found

    def iter_actor_speaking_as(self, conversation: str | int, speaker: str | int) -> Iterator[int]:
        """
        Iterate the actors speaking as a speaker.

        Pass LOUDER_ACTOR_NONE as `start` to begin.
        """
        start: Any = ACTOR_NONE
        while True:
            found = _ffi.next_actor_speaking_as(self._handle, self.declare(conversation), self.declare(speaker), start)
            if found == ACTOR_NONE:
                return
            yield found
            start = found

    def iter_hub_by_channel(self, conversation: str | int, channel: str | int) -> Iterator[int]:
        """
        Iterate the hubs in a channel.

        Hubs that name no channel belong to the default channel, so searching for it also finds them. Pass BLANK_POSITION as `start` to begin.
        """
        start: Any = BLANK_POSITION
        while True:
            found = _ffi.next_hub_by_channel(self._handle, self.declare(conversation), self.declare(channel), start)
            if found == BLANK_POSITION:
                return
            yield found
            start = found

    def iter_hub_by_priority(self, conversation: str | int, priority_min: int, priority_max: int) -> Iterator[int]:
        """
        Iterate the hubs with a priority within `[priority_min, priority_max]`.

        The bounds are inclusive. Pass BLANK_POSITION as `start` to begin.
        """
        start: Any = BLANK_POSITION
        while True:
            found = _ffi.next_hub_by_priority(self._handle, self.declare(conversation), priority_min, priority_max, start)
            if found == BLANK_POSITION:
                return
            yield found
            start = found

    def iter_hub_from_flags(self, conversation: str | int, include: HubFlags, exact: int = 0) -> Iterator[int]:
        """
        Iterate the hubs matching certain flags.

        With `exact` nonzero the flags must match exactly; otherwise every bit in `include` must be present. Pass BLANK_POSITION as `start` to begin.
        """
        start: Any = BLANK_POSITION
        while True:
            found = _ffi.next_hub_from_flags(self._handle, self.declare(conversation), include, exact, start)
            if found == BLANK_POSITION:
                return
            yield found
            start = found

    def iter_hub_from_tags(self, conversation: str | int, include: Any, include_count: int, exact: int = 0) -> Iterator[int]:
        """
        Iterate the hubs containing all, or exactly, the given tags.

        With `exact` nonzero the hub must carry exactly the listed tags; otherwise carrying extra tags is fine. Pass BLANK_POSITION as `start` to begin.
        """
        start: Any = BLANK_POSITION
        while True:
            found = _ffi.next_hub_from_tags(self._handle, self.declare(conversation), include, include_count, exact, start)
            if found == BLANK_POSITION:
                return
            yield found
            start = found

    def iter_hub_heard_by(self, conversation: str | int, speaker: str | int) -> Iterator[int]:
        """
        Iterate the hubs heard by a speaker.

        This searches the speaker's own listening speakers as well as the speaker itself, which are the hubs that may appear to an actor speaking as this speaker. Pass BLANK_POSITION as `start` to begin.
        """
        start: Any = BLANK_POSITION
        while True:
            found = _ffi.next_hub_heard_by(self._handle, self.declare(conversation), self.declare(speaker), start)
            if found == BLANK_POSITION:
                return
            yield found
            start = found

    def iter_hub_recipient(self, conversation: str | int, position: int) -> Iterator[int]:
        """
        Iterate the recipients of a hub.

        Returns LOUDER_NAME_NONE when the hub has no recipients, which means the hub is not restricted. Pass LOUDER_NAME_NONE as `start` to begin.
        """
        start: Any = NAME_NONE
        while True:
            found = _ffi.next_hub_recipient(self._handle, self.declare(conversation), position, start)
            if found == NAME_NONE:
                return
            yield found
            start = found

    def iter_hub_spoken_by(self, conversation: str | int, speaker: str | int) -> Iterator[int]:
        """
        Iterate the hubs spoken by a speaker.

        Pass BLANK_POSITION as `start` to begin.
        """
        start: Any = BLANK_POSITION
        while True:
            found = _ffi.next_hub_spoken_by(self._handle, self.declare(conversation), self.declare(speaker), start)
            if found == BLANK_POSITION:
                return
            yield found
            start = found

    def iter_hub_tag(self, conversation: str | int, position: int) -> Iterator[int]:
        """
        Iterate the tags of a position.

        Pass LOUDER_NAME_NONE as `start` to begin.
        """
        start: Any = NAME_NONE
        while True:
            found = _ffi.next_hub_tag(self._handle, self.declare(conversation), position, start)
            if found == NAME_NONE:
                return
            yield found
            start = found


    # --- events -----------------------------------------------------

    def events(self) -> Iterator[Event]:
        """Every queued event, oldest first, leaving the queue alone."""
        index = 0
        while True:
            found = _ffi.get_event(self._handle, index)
            if found.tag != LouderOption_LouderEvent_Tag.Some_LouderEvent:
                return
            yield self._event(found.some)
            index += 1

    def drain_events(self) -> list[Event]:
        """Every queued event, oldest first, and empty the queue."""
        out = list(self.events())
        _ffi.flush_events(self._handle)
        return out

    def _event(self, raw: Any) -> Event:
        """A raw event, with its names decoded and its tag read."""
        kind = EventData_KINDS.get(raw.data.tag, str(raw.data.tag))
        return Event(
            conversation=self.text(raw.conversation),
            channel=self.text(raw.channel),
            actor=raw.actor,
            kind=kind,
            value=self._event_value(kind, raw.data),
        )

    def _event_value(self, kind: str, data: Any) -> Any:
        """The arm the tag names, read as an ordinary Python value."""
        if kind == 'RegularMotion':
            arm = data.regular_motion
            return Motion(
                source_position=arm.source_position,
                target_position=arm.target_position,
                choice_index=arm.choice_index,
                background=bool(arm.background),
            )
        if kind == 'Warp':
            arm = data.warp
            return Motion(
                source_position=arm.source_position,
                target_position=arm.target_position,
                choice_index=arm.choice_index,
                background=bool(arm.background),
            )
        if kind == 'ForceMotion':
            arm = data.force_motion
            return Motion(
                source_position=arm.source_position,
                target_position=arm.target_position,
                choice_index=arm.choice_index,
                background=bool(arm.background),
            )
        if kind == 'ChoiceAppearance':
            arm = data.choice_appearance
            return ChoiceAppearance(
                position=arm.position,
                choice_index=arm.choice_index,
            )
        if kind == 'AdditionalDataChange':
            return self.text(data.additional_data_change.key)
        if kind == 'SpeakerChange':
            return self.text(data.speaker_change)
        if kind == 'TextChange':
            return self.text(data.text_change)
        if kind == 'Statement':
            return self.text(data.statement)
        if kind == 'Condition':
            return (
                self.text(data.condition.condition),
                self._variant(data.condition.outcome),
            )
        if kind == 'Callback':
            return self.text(data.callback)
        if kind == 'SpeakerAdded':
            return self.text(data.speaker_added)
        if kind == 'SpeakerRemoved':
            return self.text(data.speaker_removed)
        if kind == 'ConversationAdded':
            return self.text(data.conversation_added)
        if kind == 'ConversationRemoved':
            return self.text(data.conversation_removed)
        if kind == 'LoadError':
            return LoadFailure(
                position=data.load_error.position,
                error=LoadError(data.load_error.error),
            )
        return None

    # --- asks -------------------------------------------------------

    def asks(self) -> Iterator[Ask]:
        """Every pending ask. The order is not stable."""
        index = 0
        while True:
            found = _ffi.get_ask(self._handle, index)
            if found.tag != LouderOption_LouderAsk_Tag.Some_LouderAsk:
                return
            raw = found.some
            yield Ask(
                conversation=self.text(raw.conversation),
                channel=self.text(raw.channel),
                actor=raw.actor,
                ask=self.text(raw.ask),
            )
            index += 1

    def answer(self, index: int, verdict: int) -> bool:
        """Answer the ask at an index. Answering shifts the ones after it."""
        return _ffi.answer_ask(self._handle, index, verdict) == 0


    # --- everything the generator did not wrap ------------------

    def call(self, name: str, *args: Any) -> Any:
        """Call any entry point by name, with the signatures of the raw layer."""
        entry = getattr(_ffi, name)
        return entry(self._handle, *args) if args else entry()

    def push_event(self, event: Any) -> LouderError:
        """Put an event on the queue, for testing."""
        return LouderError(_ffi.test_push_event(self._handle, event))

    def push_ask(self, ask: Any) -> LouderError:
        """Put an ask in the cache, for testing."""
        return LouderError(_ffi.push_ask(self._handle, ask))


