pub mod object;

pub use actor::*;
pub use conversation::*;
pub use object::*;
pub use simulator::*;
pub use speaker::*;

/// An error code returned by most operations.
///
/// - [`LOUDER_ERROR_NONE`]: no error.
/// - [`LOUDER_ERROR_NOT_FOUND`]: the requested object does not exist.
pub type LouderError = i32;

/// An error code returned by load operations.
///
/// Standard JSON syntax errors are returned, as well as errors relating to the
/// shape of the loaded document.
pub type LoadError = i32;

/// Flag bits describing an event.
///
/// See the `EVENT_FLAG_*` constants. An event with 0 flags is invalid and
/// shouldn't appear.
pub type EventFlags = i32;

/// Key passed to [`crate::get_event_name`].
pub type EventName = i32;

/// Key passed to [`crate::get_event_position`].
pub type EventPosition = i32;

/// Key passed to [`crate::get_event_int`].
pub type EventInt = i32;

/// Key passed to [`crate::read_event_string`].
pub type EventString = i32;

/// Key passed to [`crate::get_ask_name`].
pub type AskName = i32;

/// Key passed to [`crate::get_ask_int`].
pub type AskInt = i32;

/// No error.
pub const LOUDER_ERROR_NONE: LouderError = 0;
/// The requested object does not exist.
pub const LOUDER_ERROR_NOT_FOUND: LouderError = -1;

/// The main text of a hub is shown.
pub const EVENT_FLAG_HUB_APPEARANCE: EventFlags = 1 << 0;
/// A hub appears outside of the focused conversation.
pub const EVENT_FLAG_BACKGROUND_HUB_APPEARANCE: EventFlags = 1 << 1;
/// A choice is shown on a hub.
pub const EVENT_FLAG_CHOICE_APPEARANCE: EventFlags = 1 << 2;
/// The event is a callback.
pub const EVENT_FLAG_CALLBACK: EventFlags = 1 << 3;
/// The event is a statement.
pub const EVENT_FLAG_STATEMENT: EventFlags = 1 << 4;
/// The event evaluates a condition.
pub const EVENT_FLAG_CONDITION: EventFlags = 1 << 5;
/// The event indicates a channel change.
pub const EVENT_FLAG_CHANNEL: EventFlags = 1 << 6;
/// The event is a warp.
pub const EVENT_FLAG_WARP: EventFlags = 1 << 7;
/// The event was caused by forcing a position update.
pub const EVENT_FLAG_FORCE_POSITION: EventFlags = 1 << 8;
/// Additional data changed on an actor.
pub const EVENT_FLAG_ACTOR_ADDITIONAL_DATA_CHANGE: EventFlags = 1 << 9;
/// An actor was added.
pub const EVENT_FLAG_ACTOR_ADDED: EventFlags = 1 << 10;
/// An actor was removed.
pub const EVENT_FLAG_ACTOR_REMOVED: EventFlags = 1 << 11;
/// A conversation was added.
pub const EVENT_FLAG_CONVERSATION_ADDED: EventFlags = 1 << 12;
/// A conversation was removed.
pub const EVENT_FLAG_CONVERSATION_REMOVED: EventFlags = 1 << 13;
/// A speaker was added.
pub const EVENT_FLAG_SPEAKER_ADDED: EventFlags = 1 << 14;
/// A speaker was removed.
pub const EVENT_FLAG_SPEAKER_REMOVED: EventFlags = 1 << 15;
/// A duplicate id was found while loading.
pub const EVENT_FLAG_DUPLICATE_ID_LOADED: EventFlags = 1 << 16;

/// Get the conversation associated with this event.
pub const EVENT_NAME_CONVERSATION: EventName = 0;
/// Get the channel name if the event indicates a channel change.
pub const EVENT_NAME_CHANNEL: EventName = 1;
/// Get the key of the data changed on an actor if the event is
/// `EVENT_FLAG_ACTOR_ADDITIONAL_DATA_CHANGE`.
pub const EVENT_NAME_ACTOR_ADDITIONAL_DATA_CHANGED: EventName = 2;

/// Get the source position associated with this event's motion.
pub const EVENT_POSITION_SOURCE: EventPosition = 0;
/// Get the target position associated with this event's motion.
pub const EVENT_POSITION_TARGET: EventPosition = 1;
/// Get the position linked to by the choice of this event.
pub const EVENT_POSITION_CHOICE_LINKED: EventPosition = 2;

/// Get the index of the choice selected on the source position by the actor of
/// this event's motion.
pub const EVENT_INT_SOURCE_CHOICE_SELECTED: EventInt = 0;
/// Get the choice index of this event if it is a choice appearance. `-1`
/// otherwise.
pub const EVENT_INT_CHOICE_INDEX: EventInt = 1;

/// Get the text of this event if it is a hub appearance.
pub const EVENT_STRING_TEXT: EventString = 0;
/// Get the text of the choice of this event if it is a choice appearance.
pub const EVENT_STRING_CHOICE_TEXT: EventString = 1;
/// Get the callback of this event if it is a hub appearance and the event
/// indicates a callback.
pub const EVENT_STRING_CALLBACK: EventString = 2;
/// Get the statement of this event if it is a hub appearance and the event
/// indicates a statement.
pub const EVENT_STRING_STATEMENT: EventString = 3;
/// Get the condition of this event if it is a hub appearance and the event
/// indicates a condition evaluation.
pub const EVENT_STRING_CONDITION: EventString = 4;

/// Get the conversation associated with this ask.
pub const ASK_NAME_CONVERSATION: AskName = 0;
/// Get the channel name of this ask if it indicates a channel change.
pub const ASK_NAME_CHANNEL: AskName = 1;

/// Get the actor associated with this ask's motion, or just the actor.
pub const ASK_INT_ACTOR: AskInt = 0;
/// Get the source position associated with this ask's motion.
pub const ASK_INT_SOURCE_POSITION: AskInt = 1;
/// Get the index of the choice selected on the source position by the actor of
/// this ask's motion.
pub const ASK_INT_SOURCE_CHOICE_SELECTED: AskInt = 2;
/// Get the position the callback originates from.
pub const ASK_INT_CALLBACK_POSITION: AskInt = 3;

/// The opaque handle to the engine.
///
/// Obtained from [`crate::init_louder`] and released with
/// [`crate::deinit_louder`]. The contents are internal to the engine and not
/// part of the public API.
pub struct LouderContext {
  _private: (),
}