# API

The set of functions revealed to the user.

# Names
LouderName get_named_id(LouderContext* ctx, const char* name, int name_length)
Get the id of a string. If it didn't exist before, this function returns -1.

LouderName declare_name(LouderContext* ctx, const char* name, int name_length)
Assign the id of a string.

int read_name(LouderContext* ctx, LouderName name, char* name, int max_length)
From an id, read a name. If no name is associated, write 0 bytes.

When this api refers to a "name", it's referring to an integer in this system.

Conversations are id'd by names containing the file they were loaded from.

# Generic Errors
Basic errors that are reused in multiple context
LouderError
0: no error
-1: does not exist

# Events
LouderError is_event(LouderContext* ctx, int index)
 - Get an event by index. Returns 0 if the index is invalid.
EventFlags get_event_flags(LouderContext* ctx, int index)
 - Get an event type flags by index. Returns 0 for "no flags" if invalid. Events with 0 flags are invalid and shouldn't appear.
 - HUB_APPEARANCE
 - BACKGROUND_HUB_APPEARANCE
 - CHOICE_APPEARANCE
 - CALLBACK
 - STATEMENT
 - CONDITION
 - CHANNEL
 - WARP
 - FORCE_POSITION: not mentioned in reqs, but a consequence of the ability to force a position update
 - ACTOR_ADDITIONAL_DATA_CHANGE
 - ACTOR_ADDED
 - ACTOR_REMOVED
 - CONVERSATION_ADDED
 - CONVERSATION_REMOVED
 - SPEAKER_ADDED
 - SPEAKER_REMOVED
 - DUPLICATE_ID_LOADED
LouderName get_event_name(LouderContext* ctx, int index, EventName key)
 - CONVERSATION: Get a conversation by event index associated with this event.
 - CHANNEL: Get the channel name of this event if the event indicates a channel change.
 - ACTOR_ADDITIONAL_DATA_CHANGED: Get the key of the data changed on an actor if ACTOR_ADDITIONAL_DATA_CHANGE
LouderActor get_event_actor(LouderContext* ctx, int index)
 - -1 if not available.
LouderPosition get_event_position(LouderContext* ctx, int index, EventPosition key)
 - -1 if not available.
 - SOURCE_POSITION: Get the source position associated with this event's motion.
 - TARGET_POSITION: Get the target position associated with this event's motion.
 - CHOICE_LINKED_POSITION: Get the position linked to by the choice of this event.
int get_event_int(LouderContext* ctx, int index, EventInt key)
 - -1 if not available.
 - SOURCE_CHOICE_SELECTED: Get the index of the choice selected on the source position by the actor of this event's motion.
 - CHOICE_INDEX: Get the choice index of this event if CHOICE_APPEARANCE. -1 otherwise.
int read_event_string(LouderContext* ctx, int index, EventString key, char* buffer, int max_size)
 - Reads 0 bytes if index is invalid or if no `key` object is associated with this event.
 - TEXT: Get the text of this event if it's a hub appearance event.
 - CHOICE_TEXT: Get the text of the choice of this event if it's a choice appearance event.
 - CALLBACK: Get the callback of this event if it's a hub appearance event and the event indicates a callback.
 - STATEMENT: Get the statement of this event if it's a hub appearance event and the event indicates a state change.
 - CONDITION: Get the condition of this event if it's a hub appearance event and the event indicates a condition evaluation.
int get_event_priority(LouderContext* ctx, int index)
 - Get the priority of this event if the event indicates a hub appearance. 0 otherwise or if invalid index.

VariantType get_event_condition_variant_type(LouderContext* ctx, int index)
int get_event_condition_int(LouderContext* ctx, int index)
float get_event_condition_float(LouderContext* ctx, int index)
int read_event_condition_string(LouderContext* ctx, int index, char* buffer, int max_size)

VariantType get_event_additional_variant_type(LouderContext* ctx, int index, LouderName key)
int get_event_additional_int(LouderContext* ctx, int index, LouderName key)
float get_event_additional_float(LouderContext* ctx, int index, LouderName key)
int read_event_additional_string(LouderContext* ctx, int index, LouderName key, char* buffer, int max_size)

int flush_events(LouderContext* ctx)
 - Flush all events from the queue and return the number of events flushed.

Intended usage:
```c
int i = 0;
while (is_event(ctx, i)) {
  // poll event properties...
  i++;
}
flush_events(ctx);
```

Generally, language wrappers should split these events into individual events for the different types of events described in the requirements.

int is_ask(LouderContext* ctx, int index)
LouderName get_ask_name(LouderContext* ctx, int index, AskName key)
 - CONVERSATION: Get a conversation by event index associated with this event.
 - CHANNEL: Get the channel name of this event if the event indicates a channel change.
int get_ask_int(LouderContext* ctx, int index, AskInt key)
 - Returns -1 if not available or invalid index.
 - ACTOR: Get an actor associated with this event's motion or just the actor.
 - SOURCE_POSITION: Get the source position associated with this event's motion.
 - SOURCE_CHOICE_SELECTED: Get the index of the choice selected on the source position by the actor of this event's motion.
 - CALLBACK_POSITION: Get the position the callback originates from
int read_ask_callback(LouderContext* ctx, int index, char* buffer, int max_length)
int answer_ask(LouderContext* ctx, int index, int verdict)

# Accessors
"read" methods are in buffer form (int read_...(... type* buffer, int max_count) = # of items written to buffer) taking "max_" size arguments and mutable pointer args.
Iterator methods are written in the form next_... and take a start argument which should default to -1

SpeakerFlags get_speaker_flags(LouderContext* ctx, LouderName speaker)
LouderName next_speaker_listening(LouderContext* ctx, LouderName speaker, LouderName start)
VariantType get_speaker_additional_type(LouderContext* ctx, LouderName conversation, LouderName speaker, LouderName key)
int get_speaker_additional_int(LouderContext* ctx, LouderName conversation, LouderName speaker, LouderName key)
float get_speaker_additional_float(LouderContext* ctx, LouderName conversation, LouderName speaker, LouderName key)
int read_speaker_additional_string(LouderContext* ctx, LouderName conversation, LouderName speaker, LouderName key, char *buffer, int max_size)

LouderName get_actor_channel(LouderContext* ctx, LouderName conversation, LouderActor actor)
LouderPosition get_actor_position(LouderContext* ctx, LouderName conversation, LouderActor actor)
LouderName next_actor_speak_as(LouderContext* ctx, LouderName conversation, LouderActor actor, LouderName start)
LouderName next_actor_listening_base(LouderContext* ctx, LouderName conversation, LouderActor actor, LouderName start)
LouderName next_actor_listening(LouderContext* ctx, LouderName conversation, LouderActor actor, LouderName start)
 - listen_to + speak as listening speakers
ActorFlags get_actor_flags(LouderContext* ctx, LouderName conversation, LouderActor actor)
int get_actor_priority(LouderContext* ctx, LouderName conversation, LouderActor actor)
VariantType get_actor_additional_type(LouderContext* ctx, LouderName conversation, LouderActor actor, LouderName key)
int get_actor_additional_int(LouderContext* ctx, LouderName conversation, LouderActor actor, LouderName key)
float get_actor_additional_float(LouderContext* ctx, LouderName conversation, LouderActor actor, LouderName key)
int read_actor_additional_string(LouderContext* ctx, LouderName conversation, LouderActor actor, LouderName key, char *buffer, int max_size)

LouderActor next_actor_in_channel(LouderContext* ctx, LouderName conversation, LouderName channel, LouderActor start)

LouderActor next_actor_in_channel_and_position(LouderContext* ctx, LouderName conversation, LouderName channel, LouderPosition position, LouderActor start)
LouderActor next_actor_speaking_as(LouderContext* ctx, LouderName conversation, LouderName speaker, LouderActor start)
LouderActor next_actor_listening(LouderContext* ctx, LouderName conversation, LouderName speaker, LouderActor start)
 - Includes listen_to and listening actors derived from listening speakers in speak_as.
LouderActor next_actor_from_flags(LouderContext* ctx, LouderName conversation, ActorFlags include, int exact, LouderActor start)

LouderName next_hub_tag(LouderContext* ctx, LouderName conversation, LouderPosition position, LouderName start)
LouderName get_hub_speaker(LouderContext* ctx, LouderName conversation, LouderPosition position)
int read_hub_text(LouderContext* ctx, LouderName conversation, LouderPosition position, char* text, int max_length)
int get_hub_choice_count(LouderContext* ctx, LouderName conversation, LouderPosition position, LouderActor actor_perspective)
int read_hub_choice_text(LouderContext* ctx, LouderName conversation, LouderPosition position, LouderActor actor_perspective, int choice_index, char* text, int max_length)
int get_hub_choice_link(LouderContext* ctx, LouderName conversation, LouderPosition position, LouderActor actor_perspective, int choice_index)
int read_hub_expressions(LouderContext* ctx, LouderName conversation, LouderPosition position, char* expressions, int max_length)
int read_hub_callbacks(LouderContext* ctx, LouderName conversation, LouderPosition position, char* callbacks, int max_length)
HubFlags get_hub_flags(LouderContext* ctx, LouderName conversation, LouderPosition position)
LouderName next_hub_recipient(LouderContext* ctx, LouderName conversation, LouderPosition position, LouderName start)
LouderName get_hub_channel(LouderContext* ctx, LouderName conversation, LouderPosition position)
int get_hub_priority(LouderContext* ctx, LouderName conversation, LouderPosition position)
VariantType get_hub_additional_type(LouderContext* ctx, LouderName conversation, LouderPosition position, LouderName key)
int get_hub_additional_int(LouderContext* ctx, LouderName conversation, LouderPosition position, LouderName key)
float get_hub_additional_float(LouderContext* ctx, LouderName conversation, LouderPosition position, LouderName key)
int read_hub_additional_string(LouderContext* ctx, LouderName conversation, LouderPosition position, LouderName key, char *buffer, int max_size)

LouderPosition next_hub_by_channel(LouderContext* ctx, LouderName conversation, LouderName channel, LouderPosition start)
LouderPosition next_hub_spoken_by(LouderContext* ctx, LouderName conversation, LouderName speaker, LouderPosition start)
LouderPosition next_hub_heard_by(LouderContext* ctx, LouderName conversation, LouderName speaker, LouderPosition start)
 - Uses speakers' listening speakers to search for hubs that may appear to actors speaking as this speaker.
LouderPosition next_hub_by_priority(LouderContext* ctx, LouderName conversation, int priority_min, int priority_max, LouderPosition start)
LouderPosition next_hub_from_tags(LouderContext* ctx, LouderName conversation, const LouderName* include, int include_count, int exact, LouderPosition start)
LouderPosition next_hub_from_flags(LouderContext* ctx, LouderName conversation, HubFlags include, int exact, LouderPosition start)

VariantType get_story_state_type(LouderContext* ctx, LouderName conversation, LouderName state)
int get_story_state_int(LouderContext* ctx, LouderName conversation, LouderName state)
float get_story_state_float(LouderContext* ctx, LouderName conversation, LouderName state)
int read_story_state_string(LouderContext* ctx, LouderName conversation, LouderName state, char *buffer, int max_size)

int read_state_json(LouderContext* ctx, LouderName conversation, char* buffer, int max_size)
 - Serialize the story state in JSON.
int read_snapshot_json(LouderContext* ctx, LouderName conversation, char* buffer, int max_size)
 - Serialize the story state, as well as the positions and details of all actors in JSON.

# Modifiers
LouderContext* init_louder()
 - Initialize the event queue and ask cache.
int deinit_louder(LouderContext* ctx)
 - Remove all conversations, speaker objects, and the event queue.
LoadError load_conversation_json(LouderContext* ctx, LouderName conversation, char* buffer, int size)
 - Standard JSON syntax errors are returned
 - JSON is expected in array form, returning an error otherwise.
 - Standard properties are left default or read from matching JSON fields.
 - Additional JSON fields are treated as additional data on hubs.
 - Error returns if the provided name is already occupied by another conversation.
 - Errors relating to individual objects, such as DUPLICATE_ID_LOADED in the conversation are emitted as events with the invalid hub's position as get_event_int(LouderContext* ctx, index, SOURCE_POSITION)
int remove_conversation(LouderContext* ctx, LouderName conversation)
 - Remove a conversation if available and remove all associated actors and objects.
 - Returns error if conversation name was not loaded.
LoadError load_conversation_config(LouderContext* ctx, char* buffer, int size)
 - JSON array form like conversation. 
 - Same error scheme and data format, but with speakers as the target objects instead of hubs. Config is also a singleton as opposed to a named instance.
 - Objects are speakers and their configurations.
int clear_config(LouderContext* ctx)
 - Remove all speakers in this config
int add_actor(LouderContext* ctx, LouderName conversation, LouderPosition start_position, ActorFlags flags, const LouderName* speak_as, int speak_as_count, const LouderName* listen_to, int listen_to_count)
 - returns int id-ing this actor.
int remove_actor(LouderContext* ctx, LouderName conversation, LouderActor actor)
int select_choice(LouderContext* ctx, LouderName conversation, LouderActor actor, int choice_index)
int force_warp(LouderContext* ctx, LouderName conversation, LouderActor actor, LouderPosition target_position, LouderName fake_channel, int fake_priority)
 - fake_priority unused if 0, otherwise overrides priority of the destination
 - fake_channel unused if -1, otherwise overrides channel of the destination
int force_position(LouderContext* ctx, LouderName conversation, LouderActor actor, LouderPosition target_position, LouderName fake_channel, int fake_priority)
int set_story_state_int(LouderContext* ctx, LouderName conversation, LouderName key, int value)
int set_story_state_float(LouderContext* ctx, LouderName conversation, LouderName key, float value)
int set_story_state_string(LouderContext* ctx, LouderName conversation, LouderName key, const char* buffer, int buf_size)