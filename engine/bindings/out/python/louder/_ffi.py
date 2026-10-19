# Bindings for Python, generated from the C header of the Louder engine.
#
# Do not edit. Run `python engine/bindings/generate.py` and this file comes back.
"""
The C boundary, exactly as the header describes it.

Nothing here interprets anything. A name is an integer, a tagged
union is a tag and a block of bytes, and a string is a length and a
buffer. The convenience layer above is where those become names,
variants and text; use it unless you are writing one yourself.
"""

from __future__ import annotations

import ctypes
import os

from ctypes import POINTER
from ctypes import c_bool, c_double, c_float, c_int8, c_int16, c_int32, c_int64
from ctypes import c_size_t, c_ssize_t, c_uint8, c_uint16, c_uint32, c_uint64
from typing import Any

__all__ = ["LIBRARY_NAMES", "load_library", "available"]


LIBRARY_NAMES = ("louder_rust.dll", "liblouder_rust.so", "liblouder_rust.dylib", "louder_rust.so", "louder_rust.dylib",)

# Point this at the library to skip the search.
LIBRARY_ENV = "LOUDER_LIBRARY"

_library: ctypes.CDLL | None = None


def library_candidates() -> list[str]:
    """Where to look for the engine, in the order to try it."""
    override = os.environ.get(LIBRARY_ENV)
    if override:
        return [override]

    here = os.path.dirname(os.path.abspath(__file__))
    roots = [os.path.normpath(os.path.join(here, "..", "..")), here]
    out: list[str] = []
    for root in roots:
        for name in LIBRARY_NAMES:
            out.append(os.path.join(root, name))
    return out


def load_library(path: str | None = None) -> ctypes.CDLL:
    """Load the engine, or raise with every place that was tried."""
    global _library
    if path is None and _library is not None:
        return _library

    tried = [path] if path else library_candidates()
    errors: list[str] = []
    for candidate in tried:
        try:
            found = ctypes.CDLL(candidate)
        except OSError as error:
            errors.append("  " + str(candidate) + ": " + str(error))
            continue
        _bind(found)
        if path is None:
            _library = found
        return found

    raise OSError(
        "the Louder engine could not be loaded. Build it with "
        + "cargo build -p louder_rust, then point " + LIBRARY_ENV + " at the "
        + "library or pass a path. Tried:" + chr(10) + chr(10).join(errors)
    )

# --- types ----------------------------------------------------------

EventData_Tag = c_int32
EventData_Tag_None = 0
EventData_Tag_RegularMotion = 0
EventData_Tag_Warp = 0
EventData_Tag_ForceMotion = 0
EventData_Tag_ChoiceAppearance = 0
EventData_Tag_ChannelChange = 0
EventData_Tag_AdditionalDataChange = 0
EventData_Tag_SpeakerChange = 0
EventData_Tag_TextChange = 0
EventData_Tag_Statement = 0
EventData_Tag_Condition = 0
EventData_Tag_Callback = 0
EventData_Tag_ActorAdded = 0
EventData_Tag_ActorRemoved = 0
EventData_Tag_SpeakerAdded = 0
EventData_Tag_SpeakerRemoved = 0
EventData_Tag_ConversationAdded = 0
EventData_Tag_ConversationRemoved = 0
EventData_Tag_LoadError = 0
LouderError = c_int32
LouderError_Ok = 0
LouderError_Missing = 0
LouderOption_LouderAsk_Tag = c_int32
LouderOption_LouderAsk_Tag_None_LouderAsk = 0
LouderOption_LouderAsk_Tag_Some_LouderAsk = 0
LouderOption_LouderEvent_Tag = c_int32
LouderOption_LouderEvent_Tag_None_LouderEvent = 0
LouderOption_LouderEvent_Tag_Some_LouderEvent = 0
LouderVariant_Tag = c_int32
LouderVariant_Tag_Int = 0
LouderVariant_Tag_Float = 0
LouderVariant_Tag_String = 0


# """Flag bits describing an actor."""
class ActorFlags(ctypes.Structure):
    _fields_ = [
        ('bits', c_int32),
    ]

# """A key of an actor's additional data being replaced."""
class AdditionalDataChange(ctypes.Structure):
    _fields_ = [
        ('key', c_int32),
    ]

# """A choice becoming available on a hub."""
class ChoiceAppearance(ctypes.Structure):
    _fields_ = [
        ('position', c_int32),
        ('choice_index', c_int32),
    ]

#
class LouderVariant_Body(ctypes.Union):
    _fields_ = [
        ('int_', c_int32),
        ('float_', c_float),
        ('string', c_int32),
    ]

# """
# One unit of data in an actor, hub, speaker, or story state.

# Booleans are not a separate kind: they are carried as LouderVariant::Int 1 or 0, which is what makes them interoperable with the arithmetic and comparison rules of the engine guide.
# """
class LouderVariant(ctypes.Structure):
    _fields_ = [
        ('tag', LouderVariant_Tag),
        ('body', LouderVariant_Body),
    ]
    _anonymous_ = ('body',)

# """What a condition on a hub came to, when an actor landed on it."""
class ConditionOutcome(ctypes.Structure):
    _fields_ = [
        ('condition', c_int32),
        ('outcome', LouderVariant),
    ]

# """
# The dialogue rules a host can turn on and off.

# Both are on by default, because each is what makes the engine usable without every conversation having to be written around its absence. They are gathered here rather than left as separate switches so that a host reads and writes the whole configuration in one call, and so that adding a rule does not add another pair of engine calls.
# """
class DialogueRules(ctypes.Structure):
    _fields_ = [
        ('bits', c_int32),
    ]

# """
# One move of an actor from one hub to another.

# Every event a move produces carries the same motion, which is how a host reads where an actor came from and where it went.
# """
class Motion(ctypes.Structure):
    _fields_ = [
        ('source_position', c_int32),
        ('target_position', c_int32),
        ('choice_index', c_int32),
        ('background', c_bool),
    ]

# """
# A record in a document the engine could not read.

# Carries the position of the record that was at fault, or LOUDER_POSITION_NONE when the problem was with the document as a whole.
# """
class LoadFailure(ctypes.Structure):
    _fields_ = [
        ('position', c_int32),
        ('error', c_int32),
    ]

#
class EventData_Body(ctypes.Union):
    _fields_ = [
        ('regular_motion', Motion),
        ('warp', Motion),
        ('force_motion', Motion),
        ('choice_appearance', ChoiceAppearance),
        ('additional_data_change', AdditionalDataChange),
        ('speaker_change', c_int32),
        ('text_change', c_int32),
        ('statement', c_int32),
        ('condition', ConditionOutcome),
        ('callback', c_int32),
        ('speaker_added', c_int32),
        ('speaker_removed', c_int32),
        ('conversation_added', c_int32),
        ('conversation_removed', c_int32),
        ('load_error', LoadFailure),
    ]

# """
# What an LouderEvent is about.

# A tagged union cannot be read as one value by a host that has to switch on it, so this is the discriminant of an EventData as an `int`, and the payload is read through the accessors that match it. A host that does not recognise a value is looking at something a newer engine produced.
# """
class EventData(ctypes.Structure):
    _fields_ = [
        ('tag', EventData_Tag),
        ('body', EventData_Body),
    ]
    _anonymous_ = ('body',)

# """Flag bits describing a hub."""
class HubFlags(ctypes.Structure):
    _fields_ = [
        ('bits', c_int32),
    ]

# """
# A question the engine has asked the host and is waiting on.

# Answering an ask moves it to the back of the cache, so indexes shift; read the cache again after each answer.
# """
class LouderAsk(ctypes.Structure):
    _fields_ = [
        ('conversation', c_int32),
        ('channel', c_int32),
        ('actor', c_int32),
        ('motion', Motion),
        ('ask', c_int32),
    ]

# """
# One entry in the event queue.

# The queue is polled from index 0 upwards and emptied with LouderContext::flush_events(super::context::LouderContext).
# """
class LouderEvent(ctypes.Structure):
    _fields_ = [
        ('conversation', c_int32),
        ('channel', c_int32),
        ('actor', c_int32),
        ('data', EventData),
    ]

#
class LouderOption_LouderAsk_Body(ctypes.Union):
    _fields_ = [
        ('some', LouderAsk),
    ]

# """Custom Option type exposed to the C FFI"""
class LouderOption_LouderAsk(ctypes.Structure):
    _fields_ = [
        ('tag', LouderOption_LouderAsk_Tag),
        ('body', LouderOption_LouderAsk_Body),
    ]
    _anonymous_ = ('body',)

#
class LouderOption_LouderEvent_Body(ctypes.Union):
    _fields_ = [
        ('some', LouderEvent),
    ]

# """Custom Option type exposed to the C FFI"""
class LouderOption_LouderEvent(ctypes.Structure):
    _fields_ = [
        ('tag', LouderOption_LouderEvent_Tag),
        ('body', LouderOption_LouderEvent_Body),
    ]
    _anonymous_ = ('body',)

# """Flag bits describing a speaker."""
class SpeakerFlags(ctypes.Structure):
    _fields_ = [
        ('bits', c_int32),
    ]


class LouderContext(ctypes.Structure):
    """The opaque handle to the engine. Obtained from crate::init_louder and released w..."""
    pass

LoadError = c_int32
LouderActor = c_int32
LouderCallback = c_int32
LouderExpression = c_int32
LouderName = c_int32
LouderPosition = c_int32



# --- constants ------------------------------------------------------

ActorFlags_SPECTATOR = ActorFlags.bits
ActorFlags_DEBUG = ActorFlags.bits
DialogueRules_SPOKE = DialogueRules.bits
DialogueRules_SINGLEPLAYER = DialogueRules.bits
HubFlags_AUTO = HubFlags.bits
HubFlags_ASK = HubFlags.bits
SpeakerFlags_PUBLIC = SpeakerFlags.bits
LOUDER_ACTOR_NONE = -1
LOUDER_LOAD_ALREADY_LOADED = 4
LOUDER_LOAD_MALFORMED_JSON = 1
LOUDER_LOAD_NOT_AN_OBJECT = 3
LOUDER_LOAD_NOT_A_LIST = 2
LOUDER_LOAD_OK = 0
LOUDER_NAME_NONE = -1


# --- entry points ---------------------------------------------------

# Each name is bound the first time it is used, so importing the types
# above does not require the engine to be present at all.
_PROTOTYPES: dict[str, tuple[Any, tuple[Any, ...]]] = {
    'add_actor': (c_int32, (POINTER(LouderContext), c_int32, c_int32, ActorFlags, POINTER(c_int32), c_int32, POINTER(c_int32), c_int32, )),
    'answer_ask': (LouderError, (POINTER(LouderContext), c_size_t, c_int32, )),
    'clear_config': (c_int32, (POINTER(LouderContext), )),
    'declare_name': (c_int32, (POINTER(LouderContext), POINTER(c_uint8), c_size_t, )),
    'deinit_louder': (LouderError, (POINTER(LouderContext), )),
    'flush_events': (None, (POINTER(LouderContext), )),
    'force_position': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, c_int32, c_int32, )),
    'force_warp': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, c_int32, c_int32, )),
    'get_actor_additional': (LouderVariant, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'get_actor_channel': (c_int32, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_actor_flags': (ActorFlags, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_actor_position': (c_int32, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_actor_priority': (c_int32, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_ask': (LouderOption_LouderAsk, (POINTER(LouderContext), c_size_t, )),
    'get_dialogue_rules': (DialogueRules, (POINTER(LouderContext), )),
    'get_event': (LouderOption_LouderEvent, (POINTER(LouderContext), c_size_t, )),
    'get_hub_additional': (LouderVariant, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'get_hub_callbacks': (c_int32, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_hub_channel': (c_int32, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_hub_choice_count': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'get_hub_choice_link': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, c_int32, )),
    'get_hub_choice_text': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, c_int32, )),
    'get_hub_expressions': (c_int32, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_hub_flags': (HubFlags, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_hub_priority': (c_int32, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_hub_speaker': (c_int32, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_hub_text': (c_int32, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_named_id': (c_int32, (POINTER(LouderContext), POINTER(c_uint8), c_size_t, )),
    'get_next_speaker_listen_to': (c_int32, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_speaker_additional': (LouderVariant, (POINTER(LouderContext), c_int32, c_int32, )),
    'get_speaker_flags': (SpeakerFlags, (POINTER(LouderContext), c_int32, )),
    'get_story_state': (LouderVariant, (POINTER(LouderContext), c_int32, c_int32, )),
    'init_louder': (POINTER(LouderContext), ()),
    'load_conversation_config': (c_int32, (POINTER(LouderContext), POINTER(c_uint8), c_size_t, )),
    'load_conversation_json': (c_int32, (POINTER(LouderContext), c_int32, POINTER(c_uint8), c_size_t, )),
    'make_event_data_actor_added': (EventData, ()),
    'make_event_data_actor_removed': (EventData, ()),
    'make_event_data_additional_data_change': (EventData, (AdditionalDataChange, )),
    'make_event_data_callback': (EventData, (c_int32, )),
    'make_event_data_channel_change': (EventData, ()),
    'make_event_data_choice_appearance': (EventData, (ChoiceAppearance, )),
    'make_event_data_condition': (EventData, (ConditionOutcome, )),
    'make_event_data_conversation_added': (EventData, (c_int32, )),
    'make_event_data_conversation_removed': (EventData, (c_int32, )),
    'make_event_data_force_motion': (EventData, (Motion, )),
    'make_event_data_load_error': (EventData, (LoadFailure, )),
    'make_event_data_none': (EventData, ()),
    'make_event_data_regular_motion': (EventData, (Motion, )),
    'make_event_data_speaker_added': (EventData, (c_int32, )),
    'make_event_data_speaker_change': (EventData, (c_int32, )),
    'make_event_data_speaker_removed': (EventData, (c_int32, )),
    'make_event_data_statement': (EventData, (c_int32, )),
    'make_event_data_text_change': (EventData, (c_int32, )),
    'make_event_data_warp': (EventData, (Motion, )),
    'make_some_ask': (LouderOption_LouderAsk, (LouderAsk, )),
    'make_some_event': (LouderOption_LouderEvent, (LouderEvent, )),
    'make_variant_bool': (LouderVariant, (c_bool, )),
    'make_variant_float': (LouderVariant, (c_float, )),
    'make_variant_int': (LouderVariant, (c_int32, )),
    'make_variant_string': (LouderVariant, (c_int32, )),
    'make_variant_zero': (LouderVariant, ()),
    'next_actor_from_flags': (c_int32, (POINTER(LouderContext), c_int32, ActorFlags, c_int32, c_int32, )),
    'next_actor_in_channel': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'next_actor_in_channel_and_position': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, c_int32, )),
    'next_actor_listening': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'next_actor_listening_base': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'next_actor_listening_to': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'next_actor_speak_as': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'next_actor_speaking_as': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'next_hub_by_channel': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'next_hub_by_priority': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, c_int32, )),
    'next_hub_from_flags': (c_int32, (POINTER(LouderContext), c_int32, HubFlags, c_int32, c_int32, )),
    'next_hub_from_tags': (c_int32, (POINTER(LouderContext), c_int32, POINTER(c_int32), c_int32, c_int32, c_int32, )),
    'next_hub_heard_by': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'next_hub_recipient': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'next_hub_spoken_by': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'next_hub_tag': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'push_ask': (LouderError, (POINTER(LouderContext), POINTER(LouderAsk), )),
    'read_name': (c_size_t, (POINTER(LouderContext), c_int32, POINTER(c_uint8), c_size_t, )),
    'read_snapshot_json': (c_int32, (POINTER(LouderContext), c_int32, POINTER(c_uint8), c_size_t, )),
    'read_state_json': (c_int32, (POINTER(LouderContext), c_int32, POINTER(c_uint8), c_size_t, )),
    'remove_actor': (c_int32, (POINTER(LouderContext), c_int32, c_int32, )),
    'remove_conversation': (c_int32, (POINTER(LouderContext), c_int32, )),
    'select_choice': (c_int32, (POINTER(LouderContext), c_int32, c_int32, c_int32, )),
    'set_dialogue_rules': (c_int32, (POINTER(LouderContext), DialogueRules, )),
    'set_story_state': (None, (POINTER(LouderContext), c_int32, c_int32, LouderVariant, )),
    'test_linkage': (c_int32, ()),
    'test_push_event': (LouderError, (POINTER(LouderContext), POINTER(LouderEvent), )),
}

def _bind(found: ctypes.CDLL) -> None:
    for name, (restype, argtypes) in _PROTOTYPES.items():
        entry = getattr(found, name)
        entry.restype = restype
        entry.argtypes = argtypes
        globals()[name] = entry


def __getattr__(name: str) -> Any:
    """Bind an entry point the first time it is asked for."""
    if name in _PROTOTYPES:
        entry = getattr(load_library(), name)
        restype, argtypes = _PROTOTYPES[name]
        entry.restype = restype
        entry.argtypes = argtypes
        globals()[name] = entry
        return entry
    raise AttributeError(
        'module ' + __name__ + ' has no attribute ' + repr(name)
    )


def available() -> bool:
    """Whether the engine can be loaded at all."""
    try:
        load_library()
    except OSError:
        return False
    return True

