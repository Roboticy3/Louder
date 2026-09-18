use bitflags::bitflags;

use crate::louder::callback::LouderCallback;

use super::actor::LouderActor;
use super::object::LouderName;
use super::hub::LouderPosition;

bitflags! {
  /// Flag bits describing an event.
  ///
  /// An event with 0 flags is invalid and shouldn't appear.
  #[repr(transparent)]
  pub struct EventFlags: i32 {
    /// The hub at motion.target appeared. 
    const HUB_APPEARANCE = 1 << 0;
    /// The hub at motion.target appeared in the background.
    const BACKGROUND_HUB_APPEARANCE = 1 << 1;
    /// The choice at hub motion.target, index motion.choice_index appeared.
    const CHOICE_APPEARANCE = 1 << 2;
    /// The hub at motion.target has a callback that needs to be evaluated.
    /// If the hub had the `HubFlags::ASK` flag set, an ask event is emitted instead.
    const CALLBACK = 1 << 3;
    /// The hub at motion.target contains a statement that has been evaluated by the engine.
    const STATEMENT = 1 << 4;
    /// The hub at motion.target contains a condition that has been evaluated by the engine.
    const CONDITION = 1 << 5;
    /// The hub at motion.target changed channels.
    const CHANNEL = 1 << 6;
    /// The motion is a warp.
    const WARP = 1 << 7;
    /// The motion is a forced position update.
    const FORCE_POSITION = 1 << 8;
    /// Additional data changed on motion.actor
    const ACTOR_ADDITIONAL_DATA_CHANGE = 1 << 9;
    /// actor_added_or_removed was added.
    const ACTOR_ADDED = 1 << 10;
    /// actor_added_or_removed was removed.
    const ACTOR_REMOVED = 1 << 11;
    /// conversation_added_or_removed was added.
    const CONVERSATION_ADDED = 1 << 12;
    /// conversation_added_or_removed was removed.
    const CONVERSATION_REMOVED = 1 << 13;
    /// speaker_added_or_removed was added.
    const SPEAKER_ADDED = 1 << 14;
    /// speaker_added_or_removed was removed.
    const SPEAKER_REMOVED = 1 << 15;
    /// A duplicate hub was found while loading a conversation
    const DUPLICATE_HUB_LOADED = 1 << 16;
  }
}

/// Key passed to [`crate::get_event_name`].
#[repr(i32)]
pub enum EventName {
  /// Get the conversation associated with this event.
  Conversation = 0,
  /// Get the channel name if the event indicates a channel change.
  Channel,
  /// Get the key of the data changed on an actor if the event is
  /// `EventFlags::ACTOR_ADDITIONAL_DATA_CHANGE`.
  ActorAdditionalDataChanged,
    /// Get the text of this event if it is a hub appearance.
  Text,
  /// Get the text of the choice of this event if it is a choice appearance.
  ChoiceText,
  /// Get the callback of this event if it is a hub appearance and the event
  /// indicates a callback.
  Callback,
  /// Get the statement of this event if it is a hub appearance and the event
  /// indicates a statement.
  Statement,
  /// Get the condition of this event if it is a hub appearance and the event
  /// indicates a condition evaluation.
  Condition,
}

/// Key passed to [`crate::get_event_position`].
#[repr(i32)]
pub enum EventPosition {
  /// Get the source position associated with this event's motion.
  Source = 0,
  /// Get the target position associated with this event's motion.
  Target = 1,
  /// Get the position linked to by the choice of this event.
  ChoiceLinked = 2,
}

/// Key passed to [`crate::get_event_int`].
#[repr(i32)]
pub enum EventInt {
  /// Get the index of the choice selected on the source position by the actor of
  /// this event's motion.
  SourceChoiceSelected = 0,
  /// Get the choice index of this event if it is a choice appearance. `-1`
  /// otherwise.
  ChoiceIndex = 1,
}

/// Key passed to [`crate::get_ask_name`].
#[repr(i32)]
pub enum AskName {
  /// Get the conversation associated with this ask.
  Conversation = 0,
  /// Get the channel name of this ask if it indicates a channel change.
  Channel = 1,
}

/// Key passed to [`crate::get_ask_int`].
#[repr(i32)]
pub enum AskInt {
  /// Get the actor associated with this ask's motion, or just the actor.
  Actor = 0,
  /// Get the source position associated with this ask's motion.
  SourcePosition = 1,
  /// Get the index of the choice selected on the source position by the actor of
  /// this ask's motion.
  SourceChoiceSelected = 2,
  /// Get the position the callback originates from.
  CallbackPosition = 3,
}

pub struct Motion {
  actor: LouderActor,
  source_position: LouderName,
  target_position: LouderName,
  choice_index: i32,
}

pub struct LouderEvent {
  flags: EventFlags,
  conversation: LouderName,
  channel: LouderName,
  motion: Motion,
  actor_added_or_removed: LouderActor,
  speaker_added_or_removed: LouderName,
  conversation_added_or_removed: LouderName,
  duplicate_hub_loaded: LouderPosition,
}

pub struct LouderAsk {
  flags: EventFlags,
  conversation: LouderName,
  position: LouderPosition,
  channel: LouderName,
  actor: LouderActor,
  ask: LouderCallback,
}