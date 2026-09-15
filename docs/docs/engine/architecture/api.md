# API

The set of functions revealed to the user.

# Names
Name get_named_id(char* name, int name_length)
Get the id of a string. If it didn't exist before, this function returns -1.

Name declare_name(char* name, int name_length)
Assign the id of a string.

int read_name(Name name, char* name, int max_length)
From an id, read a name. If no name is associated, write 0 bytes.

When this api refers to a "name", it's referring to an integer in this system.

Conversations are id'd by names containing the file they were loaded from.

# Events
int poll_event(int index)
 - Get an event by index. Returns 0 id the index is invalid.
EventFlags get_event_flags(int index)
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
 - ACTOR_ADDITONAL_DATA_CHANGE
 - ACTOR_ADDED
 - ACTOR_REMOVED
 - CONVERSATION_ADDED
 - CONVERSATION_REMOVED
 - SPEAKER_ADDED
 - SPEAKER_REMOVED
 - DUPLICATE_ID_LOADED
int get_event_int(int index, EventInt key)
 - Returns -1 if not available or invalid index.
 - ACTOR: Get an actor associated with this event's motion or just the actor.
 - SOURCE_POSITION: Get the source position associated with this event's motion.
 - SOURCE_CHOICE_SELECTED: Get the index of the choice selected on the source position by the actor of this event's motion.
 - TARGET_POSITION: Get the target position associated with this event's motion.
 - CHOICE_INDEX: Get the choice index of this event.
 - CHOICE_LINKED_POSITION: Get the position linked to by the choice of thie event.
 - CONVERSATION: Get a conversation by event index.is associated with this event.
 - CHANNEL: Get the channel name of this event if the event indicates a channel change.
int read_event_string(int index, EventString key, char* buffer, int max_size)
 - Reads 0 bytes if index is invalid or if no `key` object is associated with this event.
 - TEXT: Get the text of this event if it's a hub appearance event.
 - CHOICE_TEXT: Get the text of the choice of this event if it's a choice appearance event.
 - CALLBACK: Get the callback of this event if it's a hub appearance event and the event indicates a callback.
 - STATMENT: Get the statement of this event if it's a hub appearance event and the event indicates a state change.
 - CONDITION: Get the condition of this event if it's a hub appearance event and the event indicates a condition evaluation.
int get_event_priority(int index)
 - Get the priority of this event if the event indicates a hub appearance. 0 otherwise or if invalid index.

VariantType get_event_condition_variant_type(int index)
int get_event_condition_int(int index)
float get_event_condition_float(int index)
int read_event_condition_string(int index, char* buffer, int max_size)

VariantType get_event_additional_variant_type(int index, Name key)
int get_event_additional_int(int index, Name key)
float get_event_additional_float(int index, Name key)
int read_event_additional_string(int index, Name key, char* buffer, int max_size)

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
"read" methods are either in buffer form (int read_...(... type* buffer, int max_count) = # of items written to buffer), or iterator form (type read_...(... type start = -1) = next iterator item or -1 if no items left, where type is an int, Name, or similar). Iterator form readers will have the default argument start=-1 filled in, while buffer methods will not and will take a pointer and size. 

int is_speaker_public(Name speaker)
Name read_speaker_listening(Name speaker, Name start=-1)
VariantType get_speaker_additional_type(Name conversation, Name speaker, Name key)
int get_speaker_additional_int(Name conversation, Name speaker, Name key)
float get_speaker_additional_float(Name conversation, Name speaker, Name key)
int read_speaker_additional_string(Name conversation, Name speaker, Name key, char *buffer, int max_size)

Name get_actor_channel(Name conversation, int actor)
int get_actor_position(Name conversation, int actor)
Name read_actor_speak_as(Name conversation, int actor, Name start=-1)
Name read_actor_listening_base(Name conversation, int actor, Name start=-1)
Name read_actor_listening(Name conversation, int actor, Name start=-1)
 - listen_to + speak as listening speakers
ActorFlags get_actor_flags(Name conversation, int actor)
int get_actor_priority(Name conversation, int actor)
VariantType get_actor_additional_type(Name conversation, int actor, Name key)
int get_actor_additional_int(Name conversation, int actor, Name key)
float get_actor_additional_float(Name conversation, int actor, Name key)
int read_actor_additional_string(Name conversation, int actor, Name key, char *buffer, int max_size)

int read_actors_in_channel(Name conversation, Name channel, int start=-1)
int read_actors_in_channel_and_position(Name conversation, Name channel, int position, int start=-1)
int read_actors_speaking_as(Name conversation, Name speaker, int start=-1)
int read_actors_listening(Name conversation, Name speaker, int start=-1)
 - Includes listen_to and listening actors derived from listening speakers in speak_as.
int read_actors_from_flags(Name conversation, ActorFlags include, int exact, int start=-1)

Name read_hub_tags(Name conversation, int position, Name start=-1)
Name get_hub_speaker(Name conversation, int position)
int read_hub_text(Name conversation, int position, char* text, int max_length)
int get_hub_choice_count(Name conversation, int position, int actor_perspective)
int read_hub_choice_text(Name conversation, int position, int actor_perspective, int choice_index, char* text, int max_length)
int get_hub_choice_link(Name conversation, int position, int actor_perspective, int choice_index)
int read_hub_expressions(Name conversation, int position, char* expressions, int max_length)
int read_hub_callbacks(Name conversation, int position, char* callbacks, int max_length)
HubFlags get_hub_flags(Name conversation, int position)
Name read_hub_recipients(Name conversation, int position, Name start=-1)
Name get_hub_channel(Name conversation, int position)
int get_hub_priority(Name conversation, int position)
VariantType get_hub_additional_type(Name conversation, int position, Name key)
int get_hub_additional_int(Name conversation, int position, Name key)
float get_hub_additional_float(Name conversation, int position, Name key)
int read_hub_additional_string(Name conversation, int position, Name key, char *buffer, int max_size)

int read_hubs_by_channel(Name conversation, Name channel, int start=-1)
int read_hubs_spoken_by(Name conversation, Name speaker, int start=-1)
int read_hubs_heard_by(Name conversation, Name speaker, int start=-1)
 - Uses speakers' listening speakers to search for hubs that may appear to actors speaking as this speaker.
int read_hubs_by_priority(Name conversation, int priority_min, int priority_max, int start=-1)
int read_hubs_from_tags(Name conversation, Name* include, int include_count, int exact, int start=-1)
int read_hubs_from_flags(Name conversation, HubFlags include, int exact, int start=-1)

VariantType get_story_state_type(Name conversation, Name state)
int get_story_state_int(Name conversation, Name state)
float get_story_state_float(Name conversation, Name state)
int read_story_state_string(Name conversation, Name state, char *buffer, int max_size)

int read_state_json(Name conversation, char* buffer, int max_size)
 - Serialize the story state in JSON.
int read_snapshot_json(Name conversation, char* buffer, int max_size)
 - Serialize the story state, as well as the positions and details of all actors in JSON.

# Modifiers
int init_louder()
 - Initialize the event queue
int deinit_louder()
 - Remove all conversations, speaker objects, and the event queue.
LoadError load_conversation_json(Name conversation, char* buffer, int size)
 - Standard JSON syntax errors are returned
 - JSON is expected in array form, returning an error otherwise.
 - Error returns if the provided name is already occupied by another conversation.
 - Errors relating to individual objects, such as DUPLICATE_ID_LOADED in the conversation are emitted as events with the invalid hub's position as get_event_int(index, SOURCE_POSITION)
LoadError load_conversation_config(char* buffer, int size)
int remove_conversation(Name conversation)
 - Remove a conversation if available and remove all associated actors and objects.
 - Returns error if conversation name was not loaded.
int add_actor(Name* speak_as, int speak_as_count, Name* listen_to, int listen_to_count)

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