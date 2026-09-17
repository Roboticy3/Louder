use bitflags::bitflags;

bitflags! {
  /// Flag bits describing an event.
  ///
  /// An event with 0 flags is invalid and shouldn't appear.
  #[repr(transparent)]
  pub struct EventFlags: i32 {
    /// The main text of a hub is shown.
    const HUB_APPEARANCE = 1 << 0;
    /// A hub appears outside of the focused conversation.
    const BACKGROUND_HUB_APPEARANCE = 1 << 1;
    /// A choice is shown on a hub.
    const CHOICE_APPEARANCE = 1 << 2;
    /// The event is a callback.
    const CALLBACK = 1 << 3;
    /// The event is a statement.
    const STATEMENT = 1 << 4;
    /// The event evaluates a condition.
    const CONDITION = 1 << 5;
    /// The event indicates a channel change.
    const CHANNEL = 1 << 6;
    /// The event is a warp.
    const WARP = 1 << 7;
    /// The event was caused by forcing a position update.
    const FORCE_POSITION = 1 << 8;
    /// Additional data changed on an actor.
    const ACTOR_ADDITIONAL_DATA_CHANGE = 1 << 9;
    /// An actor was added.
    const ACTOR_ADDED = 1 << 10;
    /// An actor was removed.
    const ACTOR_REMOVED = 1 << 11;
    /// A conversation was added.
    const CONVERSATION_ADDED = 1 << 12;
    /// A conversation was removed.
    const CONVERSATION_REMOVED = 1 << 13;
    /// A speaker was added.
    const SPEAKER_ADDED = 1 << 14;
    /// A speaker was removed.
    const SPEAKER_REMOVED = 1 << 15;
    /// A duplicate id was found while loading.
    const DUPLICATE_ID_LOADED = 1 << 16;
  }
}

/// Key passed to [`crate::get_event_name`].
#[repr(i32)]
pub enum EventName {
  /// Get the conversation associated with this event.
  Conversation = 0,
  /// Get the channel name if the event indicates a channel change.
  Channel = 1,
  /// Get the key of the data changed on an actor if the event is
  /// `EventFlags::ACTOR_ADDITIONAL_DATA_CHANGE`.
  ActorAdditionalDataChanged = 2,
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

/// Key passed to [`crate::read_event_string`].
#[repr(i32)]
pub enum EventString {
  /// Get the text of this event if it is a hub appearance.
  Text = 0,
  /// Get the text of the choice of this event if it is a choice appearance.
  ChoiceText = 1,
  /// Get the callback of this event if it is a hub appearance and the event
  /// indicates a callback.
  Callback = 2,
  /// Get the statement of this event if it is a hub appearance and the event
  /// indicates a statement.
  Statement = 3,
  /// Get the condition of this event if it is a hub appearance and the event
  /// indicates a condition evaluation.
  Condition = 4,
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