# API

The set of functions revealed to the user.

# Names
int get_named_id(char* name, int name_length)
Get the id of a string. If it didn't exist before, this function returns -1.

int declare_name(char* name, int name_length)
Assign the id of a string.

int read_name(int id, char* name, int max_length)
From an id, read a name. If no name is associated, write 0 bytes.

When this api refers to a "name", it's referring to an integer in this system.

Conversations are id'd by names containing the file they were loaded from.

# Events
int poll_event(int index)
 - Get an event by index. Returns 0 id the index is invalid.
EventType get_event_type(int index)
 - Get an event type by index. Returns 0 for "NONE" if index is invalid.
int get_event_int(int index, EventInt key)
 - CONVERSATION: Get a conversation by event index. Returns -1 for "no conversation" if index is invalid or no conversation is associated with this event.
 - ACTOR: Get an actor associated with this event's motion or just the actor. Returns -1 if index is invalid or no actor is associated with this event.
 - SOURCE_POSITION: Get the source position associated with this event's motion. Returns -1 if index is invalid or no motion is associated.
 - SOURCE_CHOICE_SELECTED: Get the index of the choice selected on the source position by the actor of this event's motion. Returns -1 if index is invalid, no motion is associated, or the motion associated is a warp with no emulation.
 - TARGET_POSITION: Get the target position associated with this event's motion. Returns -1 if the index is invalid or no motion is associated.
 - IS_BACKGROUND: Get whether an event is in the background. Defaults to 0 or "false" if index is invalid.
 - CHOICE_INDEX: Get the choice index of this event. Returns -1 if index is invalid of if no choice is associated with this event.
 - CHOICE_LINKED_POSITION: Get the position linked to by the choice of thie event. Returns -1 if index is invalid of if no choice is associated with this event.
 - IS_WARP: Returns 1 if this event is a warp. 0 if otherwise or invalid index.
 - IS_FORCE: Returns 1 if event has a motion resulting from a force position update. 0 if otherwise or invalid index.
 - CHANNEL: Get the channel name of this event if the event indicates a channel change. -1 otherwise or if invalid index.
 - PRIORITY: Get the priority of this event if the event indicates a hub appearance. -1 otherwise of if invalid index.
 - CONDITION_RESULT: Get the result of a condition
int read_event_string(int index, EventString key, char* buffer, int max_size)
Writes 0 bytes if index is invalid or if no `key` object is associated with this event.
 - TEXT: Get the text of this event if it's a hub appearance event.
 - CHOICE_TEXT: Get the text of the choice of this event if it's a choice appearance event.
 - CALLBACK: Get the callback of this event if it's a hub appearance event and the event indicates a callback.
 - STATMENT: Get the statement of this event if it's a hub appearance event and the event indicates a state change.
 - CONDITION: Get the condition of this event if it's a hub appearance event and the event indicates a condition evaluation.
 - ACTOR_ADDED: Whether or not this event is an actor being added or not.
 - ACTOR_REMOVED: Whether or not this event is an actor being removed or not.
VariantType get_event_additional_variant_type(int index, int key)
 - Returns NONE, INTEGER, FLOAT, or STRING depending on the type of data at the name `key` in the additional data of this event. Returns NONE if key or index are invalid.
int get_event_additional_int(int index, int key)
 - Get integer from additional data in this event. NONE, STRING map to 0 and FLOAT is truncated.
float get_event_additional_float(int index, int key)
 - Get float from additional data in this event. NONE, STRING map to 0.0 and INT is extended to decimal.
int read_event_additional_string(int index, int key, char* buffer, int max_size)
 - Get string from additional data in this event. NONE maps to "". INT and FLOAT map to their printed variants.
int flush_events()
 - Flush all events from the queue and return the number of events flushed.

Intended usage:
```c
for (int i = 0; poll_conversation_event(i); i++) {
  //and so on
}
flush_conversation_events();
```

Generally, api layers should split these events into individual events for the different types of events described in the [requirements](../index.md).

# Accessors
int get_actor_channel(int conversation, int actor)
 - Get the channel of an actor.
int get_actor_position(int conversation, int actor)
 - Get the position of an actor.
int read_actors_in_channel(int conversation, int channel, int* actors, int max_actors)
 - Find actors currently in the given conversation and channel.
 - Returns number of actors read into buffer.
int read_actors_in_channel_and_position(int conversation, int channel, int position, int* actors, int max_actors)
 - Find actors currently in the given conversation and channel and position.
 - Returns number of actors read into buffer.
int get_tags

position, tags, speakers,
text content, choices, expressions, callbacks, flags, recipients, channels, priority, and user-defined properties.

Finally, the engine exposes an interface to see the following:

Constant accessors. Channels are optional and default to "main".
 - From an actor and conversation, get the current channel and position
 - From a actor and conversation, get choices currently appearing to this actor
 - From a channel, get current actors in channel
 - From a channel and position, get current actors in channel at that position
 - From a hub position and conversation, get any hub property, including homogeneous properties
 - From an actor and conversation, get any actor property including additional properties
 - From a speaker, get any speaker property including additional properties
 - From a conversation, get any Variant from the story state
 - From tags, get hub positions
 - From flags, get hub positions
 - From channels, get hub positions
 - From priority constraints, get hub positions
 - From a speaker, get actors speaking as that speaker
 - From a speaker, get actors listening to that speaker
 - Get spectators
 - Get debug actors

Getters that return multiple of something should generally be in the form of
an iterator, taking a blank object as the starting argument and returning a
blank object when all objects have been iterated through. Since objects in the
conversation, actors, and so on are generally registered via integers, there is
always a canonical ordering with which to iterate through these sets without
having to expose complex collections to the user.

Modifiers
 - Load a conversation from a file created from the editor
 - Unload a conversation
 - Load actors and speakers from a speaker config created from the editor and add them to a conversation
 - Add an actor with speak as, listen to config to a conversation
 - Add a spectator to a conversation
 - Add a debug actor to a conversation
 - Remove an debug, spectator, or regular actor.
 - Select a choice from an actor, choice index and conversation
 - Force a warp from an actor, target, optional target choice index, and conversation
 - Force a change of position from an actor, position, and conversation.
 - Force a change of position from a channel, position, and conversation.
 - Force a channel change with an actor, channel, and conversation
 - Change the story state with a variable, variant and conversation