/* Bindings for Go, generated from the C header of the Louder engine.
 *
 * One difference from the header the engine ships: this one compiles. C keeps
 * struct tags, typedef names, and enum constants in a single namespace, and the
 * engine has a variant of `EventData_Tag` named `ChoiceAppearance` alongside a
 * struct of the same name. A constant that would redeclare something else in the
 * file is spelled with the enum's name in front of it; nothing else is renamed.
 *
 * This header is what cgo includes. The Go itself is in `louder.go`, and it is
 * the file to read first.
 *
 * Do not edit. Run `python engine/bindings/generate.py` and this file comes back.
 */

#ifndef LOUDER_H
#define LOUDER_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/*
 * What an LouderEvent is about.
 *
 * A tagged union cannot be read as one value by a host that has to switch on it, so this is
 * the discriminant of an EventData as an `int`, and the payload is read through the
 * accessors that match it. A host that does not recognise a value is looking at something a
 * newer engine produced.
 */
typedef int32_t EventData_Tag;
enum EventData_Tag_values {
    None = 0,
    RegularMotion = 1,
    Warp = 2,
    ForceMotion = 3,
    EventData_Tag_ChoiceAppearance = 4,
    ChannelChange = 5,
    EventData_Tag_AdditionalDataChange = 6,
    SpeakerChange = 7,
    TextChange = 8,
    Statement = 9,
    Condition = 10,
    Callback = 11,
    ActorAdded = 12,
    ActorRemoved = 13,
    SpeakerAdded = 14,
    SpeakerRemoved = 15,
    ConversationAdded = 16,
    ConversationRemoved = 17,
    EventData_Tag_LoadError = 18,
};

/*
 * An error code returned by most operations.
 *
 * - LouderError::Ok: no error. - LouderError::Missing: the requested object does not exist.
 */
typedef int32_t LouderError;
enum LouderError_values {
    Ok = 0,
    Missing = 1,
};

/*
 * Custom Option type exposed to the C FFI
 */
typedef int32_t LouderOption_LouderAsk_Tag;
enum LouderOption_LouderAsk_Tag_values {
    None_LouderAsk = 0,
    Some_LouderAsk = 1,
};

/*
 * Custom Option type exposed to the C FFI
 */
typedef int32_t LouderOption_LouderEvent_Tag;
enum LouderOption_LouderEvent_Tag_values {
    None_LouderEvent = 0,
    Some_LouderEvent = 1,
};

/*
 * One unit of data in an actor, hub, speaker, or story state.
 *
 * Booleans are not a separate kind: they are carried as LouderVariant::Int 1 or 0, which is
 * what makes them interoperable with the arithmetic and comparison rules of the engine
 * guide.
 */
typedef int32_t LouderVariant_Tag;
enum LouderVariant_Tag_values {
    Int = 0,
    Float = 1,
    String = 2,
};

/*
 * Flag bits describing an actor.
 */
typedef struct ActorFlags {
    int32_t bits;
} ActorFlags;

/*
 * A key of an actor's additional data being replaced.
 */
typedef struct AdditionalDataChange {
    int32_t key;
} AdditionalDataChange;

/*
 * A choice becoming available on a hub.
 */
typedef struct ChoiceAppearance {
    int32_t position;
    int32_t choice_index;
} ChoiceAppearance;

typedef union {
    int32_t int_;
    float float_;
    int32_t string;
} LouderVariant_Body;

/*
 * One unit of data in an actor, hub, speaker, or story state.
 *
 * Booleans are not a separate kind: they are carried as LouderVariant::Int 1 or 0, which is
 * what makes them interoperable with the arithmetic and comparison rules of the engine
 * guide.
 */
typedef struct {
    LouderVariant_Tag tag;
    LouderVariant_Body body;
} LouderVariant;

/*
 * What a condition on a hub came to, when an actor landed on it.
 */
typedef struct ConditionOutcome {
    int32_t condition;
    LouderVariant outcome;
} ConditionOutcome;

/*
 * The dialogue rules a host can turn on and off.
 *
 * Both are on by default, because each is what makes the engine usable without every
 * conversation having to be written around its absence. They are gathered here rather than
 * left as separate switches so that a host reads and writes the whole configuration in one
 * call, and so that adding a rule does not add another pair of engine calls.
 */
typedef struct DialogueRules {
    int32_t bits;
} DialogueRules;

/*
 * One move of an actor from one hub to another.
 *
 * Every event a move produces carries the same motion, which is how a host reads where an
 * actor came from and where it went.
 */
typedef struct Motion {
    int32_t source_position;
    int32_t target_position;
    int32_t choice_index;
    bool background;
} Motion;

/*
 * A record in a document the engine could not read.
 *
 * Carries the position of the record that was at fault, or LOUDER_POSITION_NONE when the
 * problem was with the document as a whole.
 */
typedef struct LoadFailure {
    int32_t position;
    int32_t error;
} LoadFailure;

typedef union {
    Motion regular_motion;
    Motion warp;
    Motion force_motion;
    ChoiceAppearance choice_appearance;
    AdditionalDataChange additional_data_change;
    int32_t speaker_change;
    int32_t text_change;
    int32_t statement;
    ConditionOutcome condition;
    int32_t callback;
    int32_t speaker_added;
    int32_t speaker_removed;
    int32_t conversation_added;
    int32_t conversation_removed;
    LoadFailure load_error;
} EventData_Body;

/*
 * What an LouderEvent is about.
 *
 * A tagged union cannot be read as one value by a host that has to switch on it, so this is
 * the discriminant of an EventData as an `int`, and the payload is read through the
 * accessors that match it. A host that does not recognise a value is looking at something a
 * newer engine produced.
 */
typedef struct {
    EventData_Tag tag;
    EventData_Body body;
} EventData;

/*
 * Flag bits describing a hub.
 */
typedef struct HubFlags {
    int32_t bits;
} HubFlags;

/*
 * A question the engine has asked the host and is waiting on.
 *
 * Answering an ask moves it to the back of the cache, so indexes shift; read the cache
 * again after each answer.
 */
typedef struct LouderAsk {
    int32_t conversation;
    int32_t channel;
    int32_t actor;
    Motion motion;
    int32_t ask;
} LouderAsk;

/*
 * One entry in the event queue.
 *
 * The queue is polled from index 0 upwards and emptied with
 * LouderContext::flush_events(super::context::LouderContext).
 */
typedef struct LouderEvent {
    int32_t conversation;
    int32_t channel;
    int32_t actor;
    EventData data;
} LouderEvent;

typedef union {
    LouderAsk some;
} LouderOption_LouderAsk_Body;

/*
 * Custom Option type exposed to the C FFI
 */
typedef struct {
    LouderOption_LouderAsk_Tag tag;
    LouderOption_LouderAsk_Body body;
} LouderOption_LouderAsk;

typedef union {
    LouderEvent some;
} LouderOption_LouderEvent_Body;

/*
 * Custom Option type exposed to the C FFI
 */
typedef struct {
    LouderOption_LouderEvent_Tag tag;
    LouderOption_LouderEvent_Body body;
} LouderOption_LouderEvent;

/*
 * Flag bits describing a speaker.
 */
typedef struct SpeakerFlags {
    int32_t bits;
} SpeakerFlags;

/*
 * The opaque handle to the engine.
 *
 * Obtained from crate::init_louder and released with crate::deinit_louder. The contents are
 * internal to the engine and not part of the public API.
 *
 * Story state is held here rather than on Conversation so that the shared buckets survive
 * loading and unloading individual conversations.
 *
 * The engine's whole vocabulary is reached through this one type: names, conversations,
 * actors, hubs, speakers, story state, the event queue, and the ask cache.
 */
typedef struct LouderContext LouderContext;

typedef int32_t LoadError;

typedef int32_t LouderActor;

typedef int32_t LouderCallback;

typedef int32_t LouderExpression;

typedef int32_t LouderName;

typedef int32_t LouderPosition;

#define ActorFlags_SPECTATOR ((int32_t)(1))
#define ActorFlags_DEBUG ((int32_t)(2))
#define DialogueRules_SPOKE ((int32_t)(1))
#define DialogueRules_SINGLEPLAYER ((int32_t)(2))
#define HubFlags_AUTO ((int32_t)(1))
#define HubFlags_ASK ((int32_t)(2))
#define SpeakerFlags_PUBLIC ((int32_t)(1))
#define LOUDER_ACTOR_NONE ((int32_t)(-1))
#define LOUDER_LOAD_ALREADY_LOADED ((int32_t)(4))
#define LOUDER_LOAD_MALFORMED_JSON ((int32_t)(1))
#define LOUDER_LOAD_NOT_AN_OBJECT ((int32_t)(3))
#define LOUDER_LOAD_NOT_A_LIST ((int32_t)(2))
#define LOUDER_LOAD_OK ((int32_t)(0))
#define LOUDER_NAME_NONE ((int32_t)(-1))

/*
 * Add an actor to a conversation and return the integer id'ing it.
 *
 * The actor starts at `start_position` on the default channel, with priority 0. Ids are per
 * conversation and are never reused, so ascending order is a stable iteration order.
 *
 * - `speak_as` and `listen_to` are arrays of `count` names; either may be null when its
 * count is 0. - ActorFlags::SPECTATOR and ActorFlags::DEBUG actors are given no channel or
 * priority.
 *
 * Emits an EventData::ActorAdded event on success. Returns the new actor id, or
 * LOUDER_ACTOR_NONE if the conversation is not loaded or a pointer was null.
 */
extern int32_t add_actor(
    LouderContext* ctx,
    int32_t conversation,
    int32_t start_position,
    ActorFlags flags,
    const int32_t* speak_as,
    int32_t speak_as_count,
    const int32_t* listen_to,
    int32_t listen_to_count
);
/*
 * Answer the ask at `index` with `verdict`.
 *
 * Answering marks the ask complete and moves it to the back of the cache, so indexes shift:
 * read get_ask again after each answer. Answered asks stay readable but no longer accept
 * answers, so a host can simply keep answering until LouderError::Missing comes back and
 * the cache has flushed itself.
 *
 * Returns LouderError::Ok if the answer was recorded, or LouderError::Missing if `index`
 * was out of range or that ask had already been answered.
 */
extern LouderError answer_ask(LouderContext* ctx, size_t index, int32_t verdict);
/*
 * Remove all speakers in the config.
 *
 * Returns 0 on success, or 1 if `ctx` was null. Conversations and actors are untouched, so
 * hubs naming a now-missing speaker become public again.
 */
extern int32_t clear_config(LouderContext* ctx);
/*
 * Assign the id of a string.
 *
 * If the string was already declared, returns LOUDER_NAME_NONE and leaves the existing id
 * alone. Otherwise a new id is assigned and returned.
 */
extern int32_t declare_name(LouderContext* ctx, const uint8_t* buffer, size_t max_size);
/*
 * Release a context and everything it holds.
 *
 * Returns LouderError::Ok on success, or LouderError::Missing if `ctx` was null. Every
 * conversation, speaker, actor, and queued event goes with it.
 */
extern LouderError deinit_louder(LouderContext* ctx);
/*
 * Remove every queued event.
 *
 * The queue is indexed from 0 again afterwards, so the next get_event with index 0 reads
 * the first event queued after this call.
 */
extern void flush_events(LouderContext* ctx);
/*
 * Move an actor to another position directly.
 *
 * - `fake_priority` is unused if `0`, otherwise overrides the priority of the destination.
 * Because 0 is the ordinary default priority, this cannot force an actor's priority *to* 0
 * over a hub that sets one. - `fake_channel` is unused if LOUDER_NAME_NONE, otherwise
 * overrides the channel of the destination.
 *
 * Returns 0 on success, or 1 if the conversation, actor, or destination was missing, or if
 * an auto hub ran out of choices.
 */
extern int32_t force_position(
    LouderContext* ctx,
    int32_t conversation,
    int32_t actor,
    int32_t target_position,
    int32_t fake_channel,
    int32_t fake_priority
);
/*
 * Move an actor to another position as if it had moved through the network.
 *
 * - `fake_priority` is unused if `0`, otherwise overrides the priority of the destination.
 * Because 0 is the ordinary default priority, this cannot force an actor's priority *to* 0
 * over a hub that sets one. - `fake_channel` is unused if LOUDER_NAME_NONE, otherwise
 * overrides the channel of the destination.
 *
 * Other actors sharing the actor's channel may be pulled along, exactly as if the actor had
 * chosen. Returns 0 on success, or 1 if the conversation, actor, or destination was
 * missing, or if an auto hub ran out of choices.
 */
extern int32_t force_warp(
    LouderContext* ctx,
    int32_t conversation,
    int32_t actor,
    int32_t target_position,
    int32_t fake_channel,
    int32_t fake_priority
);
/*
 * Get an actor's additional data under `key`.
 *
 * Returns a zero variant if the actor does not exist or has nothing stored under `key`.
 */
extern LouderVariant get_actor_additional(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t actor,
    int32_t key
);
/*
 * Get the channel of an actor.
 *
 * Actors with ActorFlags::SPECTATOR or ActorFlags::DEBUG are not part of the flow and read
 * as LOUDER_NAME_NONE.
 */
extern int32_t get_actor_channel(const LouderContext* ctx, int32_t conversation, int32_t actor);
/*
 * Get the flags of an actor.
 *
 * Returns ActorFlags::empty if the actor does not exist.
 */
extern ActorFlags get_actor_flags(const LouderContext* ctx, int32_t conversation, int32_t actor);
/*
 * Get the position of an actor.
 *
 * Returns BLANK_POSITION if the actor does not exist.
 */
extern int32_t get_actor_position(const LouderContext* ctx, int32_t conversation, int32_t actor);
/*
 * Get the priority of an actor.
 *
 * Actors outside the flow read as 0.
 */
extern int32_t get_actor_priority(const LouderContext* ctx, int32_t conversation, int32_t actor);
/*
 * Get the ask at `index`.
 *
 * Asks are not in a stable order, so read them until one comes back LouderOption::None
 * rather than assuming the queue only grows.
 */
extern LouderOption_LouderAsk get_ask(const LouderContext* ctx, size_t index);
/*
 * Read the dialogue rules currently in effect.
 *
 * A fresh context has every rule on. Returns an empty set if `ctx` was null, since a null
 * context is not running any dialogue.
 */
extern DialogueRules get_dialogue_rules(const LouderContext* ctx);
/*
 * Get the event at `index`.
 *
 * Returns LouderOption::None once `index` is past the end of the queue, or if `ctx` is
 * null. Poll from index 0 upwards until that happens, then call flush_events.
 */
extern LouderOption_LouderEvent get_event(const LouderContext* ctx, size_t index);
/*
 * Get a hub's additional data under `key`.
 *
 * Returns a zero variant if the hub does not exist or has nothing stored under `key`.
 */
extern LouderVariant get_hub_additional(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t position,
    int32_t key
);
/*
 * Get the callbacks associated with a hub as a name.
 *
 * The schema stores callbacks newline separated; this returns the whole block as one name.
 * Returns LOUDER_NAME_NONE if the hub has none.
 */
extern int32_t get_hub_callbacks(const LouderContext* ctx, int32_t conversation, int32_t position);
/*
 * Get the channel of a hub.
 *
 * Returns LOUDER_NAME_NONE if the hub does not change the channel, in which case an actor
 * keeps whatever channel it already had.
 */
extern int32_t get_hub_channel(const LouderContext* ctx, int32_t conversation, int32_t position);
/*
 * Get the number of choices on a hub from an actor's perspective.
 *
 * Only choices the actor could actually take are counted, so this applies the visibility,
 * recipient, and spoke rules. Returns 0 for an unknown hub or actor.
 */
extern int32_t get_hub_choice_count(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t position,
    int32_t actor_perspective
);
/*
 * Get the position a choice on a hub links to.
 *
 * Returns BLANK_POSITION if the choice does not exist or is not available to
 * `actor_perspective`.
 */
extern int32_t get_hub_choice_link(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t position,
    int32_t actor_perspective,
    int32_t choice_index
);
/*
 * Get the text of a choice on a hub as a name.
 *
 * A choice has no text of its own, so this is the text of the hub it links to. Returns
 * LOUDER_NAME_NONE if the choice does not exist or is not available to `actor_perspective`.
 */
extern int32_t get_hub_choice_text(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t position,
    int32_t actor_perspective,
    int32_t choice_index
);
/*
 * Get the expressions associated with a hub as a name.
 *
 * The schema stores expressions newline separated; this returns the whole block as one
 * name, so the caller splits it to get individual expressions. Returns LOUDER_NAME_NONE if
 * the hub has none.
 */
extern int32_t get_hub_expressions(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t position
);
/*
 * Get the flags of a hub.
 *
 * Returns HubFlags::empty if the hub does not exist.
 */
extern HubFlags get_hub_flags(const LouderContext* ctx, int32_t conversation, int32_t position);
/*
 * Get the priority of a hub.
 *
 * Returns 0 if the hub does not exist, which is also the default priority.
 */
extern int32_t get_hub_priority(const LouderContext* ctx, int32_t conversation, int32_t position);
/*
 * Get the speaker of a hub.
 *
 * Returns LOUDER_NAME_NONE if the hub has no speaker, which means it is visible to
 * everyone, or if the hub does not exist.
 */
extern int32_t get_hub_speaker(const LouderContext* ctx, int32_t conversation, int32_t position);
/*
 * Get the text of a hub as a name.
 *
 * Returns LOUDER_NAME_NONE if the hub has no text or does not exist. Read the string back
 * with read_name.
 */
extern int32_t get_hub_text(const LouderContext* ctx, int32_t conversation, int32_t position);
/*
 * Get the id of a string.
 *
 * Returns LOUDER_NAME_NONE if the string was never declared, or if either pointer was null.
 *
 * - `buffer`: the string bytes to look up. - `max_size`: the number of bytes to read from
 * `buffer`.
 */
extern int32_t get_named_id(const LouderContext* ctx, const uint8_t* buffer, size_t max_size);
/*
 * Iterate the speakers that `speaker` is listening to.
 *
 * Pass LOUDER_NAME_NONE as `start` to begin, then pass the previous result until
 * LOUDER_NAME_NONE comes back. Returns LOUDER_NAME_NONE for an unknown speaker.
 */
extern int32_t get_next_speaker_listen_to(const LouderContext* ctx, int32_t speaker, int32_t start);
/*
 * Get a speaker's additional data under `key`.
 *
 * The speaker config is a singleton rather than a per-conversation object, so this takes no
 * conversation. Returns a zero variant if the speaker is unknown or has nothing stored
 * under `key`.
 */
extern LouderVariant get_speaker_additional(const LouderContext* ctx, int32_t speaker, int32_t key);
/*
 * Get the flags of a speaker.
 *
 * A speaker that has not been declared reads as SpeakerFlags::empty, which is public by
 * default, so a hub naming it still appears to everyone.
 */
extern SpeakerFlags get_speaker_flags(const LouderContext* ctx, int32_t speaker);
/*
 * Get a story state value under `state`.
 *
 * The variable's own name decides where it is stored: a leading underscore means it is not
 * saved, and an uppercase first letter means it is shared across conversations. Variables
 * that have never been written read as 0. Reading from a conversation that is not loaded
 * also reads as 0.
 */
extern LouderVariant get_story_state(const LouderContext* ctx, int32_t conversation, int32_t state);
/*
 * Initialize the engine and return a new context.
 *
 * Initializes the name table, the event queue, and the ask cache. Release the returned
 * handle with deinit_louder. Returns null only if allocation failed.
 */
extern LouderContext* init_louder(void);
/*
 * Load the conversation config from a JSON document in `buffer`.
 *
 * - The bytes have to be UTF-8 and the error scheme is the one in load_conversation_json,
 * with speakers as the target objects instead of hubs. - The config is a singleton as
 * opposed to a named conversation instance, so this takes no conversation name. - Objects
 * are speakers and their configurations. A speaker already in the config is replaced rather
 * than treated as an error.
 */
extern int32_t load_conversation_config(LouderContext* ctx, uint8_t* buffer, size_t max_size);
/*
 * Load a conversation from a JSON document in `buffer`.
 *
 * The buffer holds `max_size` bytes. JSON is text, so those bytes have to be UTF-8 for the
 * engine to read them; LOUDER_LOAD_MALFORMED_JSON is returned if they are not, and
 * re-encoding them is the host's job rather than the engine's.
 *
 * - The JSON is expected in array form; LOUDER_LOAD_NOT_A_LIST is returned otherwise. -
 * Standard JSON syntax errors are returned as LOUDER_LOAD_MALFORMED_JSON. - Standard
 * properties are left default or read from matching JSON fields. - Additional JSON fields
 * are treated as additional data on hubs. - LOUDER_LOAD_ALREADY_LOADED is returned if
 * `conversation` is already occupied by another loaded conversation. - Problems with
 * individual records do not fail the load. They are emitted as events instead, carrying the
 * offending hub's position in Motion::source_position and the LoadError in
 * Motion::target_position.
 */
extern int32_t load_conversation_json(
    LouderContext* ctx,
    int32_t conversation,
    uint8_t* buffer,
    size_t max_size
);
/*
 * Build an EventData::ActorAdded.
 */
extern EventData make_event_data_actor_added(void);
/*
 * Build an EventData::ActorRemoved.
 */
extern EventData make_event_data_actor_removed(void);
/*
 * Build an EventData::AdditionalDataChange.
 */
extern EventData make_event_data_additional_data_change(AdditionalDataChange change);
/*
 * Build an EventData::Callback, naming the callback.
 */
extern EventData make_event_data_callback(int32_t callback);
/*
 * Build an EventData::ChannelChange.
 */
extern EventData make_event_data_channel_change(void);
/*
 * Build an EventData::ChoiceAppearance.
 */
extern EventData make_event_data_choice_appearance(ChoiceAppearance appearance);
/*
 * Build an EventData::Condition.
 */
extern EventData make_event_data_condition(ConditionOutcome outcome);
/*
 * Build an EventData::ConversationAdded, naming the conversation.
 */
extern EventData make_event_data_conversation_added(int32_t conversation);
/*
 * Build an EventData::ConversationRemoved, naming the conversation.
 */
extern EventData make_event_data_conversation_removed(int32_t conversation);
/*
 * Build an EventData::ForceMotion.
 */
extern EventData make_event_data_force_motion(Motion motion);
/*
 * Build an EventData::LoadError.
 *
 * The engine emits these itself when it cannot read a record of a loaded document, so this
 * exists to round out the set rather than because a host has a reason to raise one.
 */
extern EventData make_event_data_load_error(LoadFailure failure);
/*
 * Build the EventData::None variant, which carries nothing.
 */
extern EventData make_event_data_none(void);
/*
 * Build an EventData::RegularMotion.
 */
extern EventData make_event_data_regular_motion(Motion motion);
/*
 * Build an EventData::SpeakerAdded, naming the speaker.
 */
extern EventData make_event_data_speaker_added(int32_t speaker);
/*
 * Build an EventData::SpeakerChange, naming the speaker.
 */
extern EventData make_event_data_speaker_change(int32_t speaker);
/*
 * Build an EventData::SpeakerRemoved, naming the speaker.
 */
extern EventData make_event_data_speaker_removed(int32_t speaker);
/*
 * Build an EventData::Statement, naming the expression.
 */
extern EventData make_event_data_statement(int32_t expression);
/*
 * Build an EventData::TextChange, naming the text.
 */
extern EventData make_event_data_text_change(int32_t text);
/*
 * Build an EventData::Warp.
 */
extern EventData make_event_data_warp(Motion motion);
/*
 * Build a LouderOption::Some holding an ask.
 */
extern LouderOption_LouderAsk make_some_ask(LouderAsk value);
/*
 * Build a LouderOption::Some holding an event.
 */
extern LouderOption_LouderEvent make_some_event(LouderEvent value);
/*
 * Build a boolean as a LouderVariant::Int of 1 or 0.
 */
extern LouderVariant make_variant_bool(bool value);
/*
 * Build a LouderVariant::Float.
 */
extern LouderVariant make_variant_float(float value);
/*
 * Build a LouderVariant::Int.
 */
extern LouderVariant make_variant_int(int32_t value);
/*
 * Build a LouderVariant::String, naming the string rather than carrying it.
 */
extern LouderVariant make_variant_string(int32_t value);
/*
 * Build the LouderVariant::Int of 0 that an unwritten variable reads as.
 */
extern LouderVariant make_variant_zero(void);
/*
 * Iterate the actors matching certain flags.
 *
 * With `exact` nonzero the flags must match exactly; otherwise every bit in `include` must
 * be present. Pass LOUDER_ACTOR_NONE as `start` to begin.
 */
extern int32_t next_actor_from_flags(
    const LouderContext* ctx,
    int32_t conversation,
    ActorFlags include,
    int32_t exact,
    int32_t start
);
/*
 * Iterate the actors in a channel.
 *
 * Pass LOUDER_ACTOR_NONE as `start` to begin. Actors outside the flow have no channel and
 * are never returned.
 */
extern int32_t next_actor_in_channel(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t channel,
    int32_t start
);
/*
 * Iterate the actors in a channel at a specific position.
 *
 * Pass LOUDER_ACTOR_NONE as `start` to begin.
 */
extern int32_t next_actor_in_channel_and_position(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t channel,
    int32_t position,
    int32_t start
);
/*
 * Iterate the speakers an actor listens to.
 *
 * Includes the actor's `listen_to` speakers plus the listening speakers of the speakers it
 * speaks as. Pass LOUDER_NAME_NONE as `start` to begin.
 */
extern int32_t next_actor_listening(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t actor,
    int32_t start
);
/*
 * Iterate the base speakers an actor listens to.
 *
 * These are the actor's own `listen_to` speakers, without the ones reachable through who it
 * speaks as. See next_actor_listening for the wider set.
 */
extern int32_t next_actor_listening_base(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t actor,
    int32_t start
);
/*
 * Iterate the actors listening to a speaker.
 *
 * This covers `listen_to` plus the listening actors derived from the listening speakers of
 * the speakers an actor speaks as. Pass LOUDER_ACTOR_NONE as `start` to begin.
 */
extern int32_t next_actor_listening_to(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t speaker,
    int32_t start
);
/*
 * Iterate the speakers an actor speaks as.
 *
 * Pass LOUDER_NAME_NONE as `start` to begin, then pass the previous result until
 * LOUDER_NAME_NONE comes back.
 */
extern int32_t next_actor_speak_as(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t actor,
    int32_t start
);
/*
 * Iterate the actors speaking as a speaker.
 *
 * Pass LOUDER_ACTOR_NONE as `start` to begin.
 */
extern int32_t next_actor_speaking_as(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t speaker,
    int32_t start
);
/*
 * Iterate the hubs in a channel.
 *
 * Hubs that name no channel belong to the default channel, so searching for it also finds
 * them. Pass BLANK_POSITION as `start` to begin.
 */
extern int32_t next_hub_by_channel(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t channel,
    int32_t start
);
/*
 * Iterate the hubs with a priority within `[priority_min, priority_max]`.
 *
 * The bounds are inclusive. Pass BLANK_POSITION as `start` to begin.
 */
extern int32_t next_hub_by_priority(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t priority_min,
    int32_t priority_max,
    int32_t start
);
/*
 * Iterate the hubs matching certain flags.
 *
 * With `exact` nonzero the flags must match exactly; otherwise every bit in `include` must
 * be present. Pass BLANK_POSITION as `start` to begin.
 */
extern int32_t next_hub_from_flags(
    const LouderContext* ctx,
    int32_t conversation,
    HubFlags include,
    int32_t exact,
    int32_t start
);
/*
 * Iterate the hubs containing all, or exactly, the given tags.
 *
 * With `exact` nonzero the hub must carry exactly the listed tags; otherwise carrying extra
 * tags is fine. Pass BLANK_POSITION as `start` to begin.
 */
extern int32_t next_hub_from_tags(
    const LouderContext* ctx,
    int32_t conversation,
    const int32_t* include,
    int32_t include_count,
    int32_t exact,
    int32_t start
);
/*
 * Iterate the hubs heard by a speaker.
 *
 * This searches the speaker's own listening speakers as well as the speaker itself, which
 * are the hubs that may appear to an actor speaking as this speaker. Pass BLANK_POSITION as
 * `start` to begin.
 */
extern int32_t next_hub_heard_by(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t speaker,
    int32_t start
);
/*
 * Iterate the recipients of a hub.
 *
 * Returns LOUDER_NAME_NONE when the hub has no recipients, which means the hub is not
 * restricted. Pass LOUDER_NAME_NONE as `start` to begin.
 */
extern int32_t next_hub_recipient(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t position,
    int32_t start
);
/*
 * Iterate the hubs spoken by a speaker.
 *
 * Pass BLANK_POSITION as `start` to begin.
 */
extern int32_t next_hub_spoken_by(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t speaker,
    int32_t start
);
/*
 * Iterate the tags of a position.
 *
 * Pass LOUDER_NAME_NONE as `start` to begin.
 */
extern int32_t next_hub_tag(
    const LouderContext* ctx,
    int32_t conversation,
    int32_t position,
    int32_t start
);
/*
 * Push an ask onto the ask cache, used for testing.
 *
 * Returns LouderError::Ok on success, or LouderError::Missing if either pointer was null.
 */
extern LouderError push_ask(LouderContext* ctx, const LouderAsk* ask);
/*
 * Read the bytes associated with an id into `buffer`.
 *
 * The bytes come back exactly as they went in, so a name declared in some encoding is read
 * in that same one. Decoding them is the host's job, since the engine never displays
 * anything itself.
 *
 * Writes 0 bytes and a terminator if no name is associated with the id, or if either
 * pointer was null. Returns the number of bytes written, excluding the terminator.
 */
extern size_t read_name(const LouderContext* ctx, int32_t name, uint8_t* buffer, size_t max_size);
/*
 * Serialize the story state, along with the positions and details of every actor, in JSON
 * into `buffer`.
 *
 * The document has a `state` object and an `actors` array, both ordered so that the output
 * is reproducible.
 *
 * The document is written as UTF-8. Returns the number of bytes written excluding the
 * terminator; a negative value means the conversation is not loaded.
 */
extern int32_t read_snapshot_json(
    const LouderContext* ctx,
    int32_t conversation,
    uint8_t* buffer,
    size_t max_size
);
/*
 * Serialize the story state in JSON into `buffer`.
 *
 * Every bucket is included: the conversation's own saved and unsaved variables, and the
 * shared ones. Keys are written in ascending id order so the output is reproducible.
 *
 * The document is written as UTF-8. Returns the number of bytes written excluding the
 * terminator; a negative value means the conversation is not loaded.
 */
extern int32_t read_state_json(
    const LouderContext* ctx,
    int32_t conversation,
    uint8_t* buffer,
    size_t max_size
);
/*
 * Remove an actor from a conversation.
 *
 * Returns 0 on success, or 1 if the actor or conversation did not exist. Emits an
 * EventData::ActorRemoved event on success.
 */
extern int32_t remove_actor(LouderContext* ctx, int32_t conversation, int32_t actor);
/*
 * Remove a conversation and all associated actors.
 *
 * Returns 0 on success, or 1 if the conversation was not loaded. Shared story state is
 * kept, since it outlives any one conversation.
 */
extern int32_t remove_conversation(LouderContext* ctx, int32_t conversation);
/*
 * Select a choice on an actor's current hub.
 *
 * The choice must be one the actor could actually take; see get_hub_choice_count for what
 * that means. Auto hubs along the way are resolved by taking the first choice available at
 * each one.
 *
 * Returns 0 on success, or 1 if the actor, hub, or choice was missing, if the choice is not
 * available to this actor, or if an auto hub ran out of choices.
 */
extern int32_t select_choice(
    LouderContext* ctx,
    int32_t conversation,
    int32_t actor,
    int32_t choice_index
);
/*
 * Replace the dialogue rules.
 *
 * A rule takes effect on the next move rather than being applied to the conversation as it
 * stands, so a host can turn one off while an actor is standing on a hub it would otherwise
 * resolve through.
 *
 * Bits outside DialogueRules are ignored rather than refused, so a host carrying a
 * configuration written for a newer engine still gets the rules this build knows about.
 */
extern int32_t set_dialogue_rules(LouderContext* ctx, DialogueRules rules);
/*
 * Set a story state value.
 *
 * The variable's own name decides where it is stored, as in get_story_state. Doing nothing
 * if `ctx` is null.
 */
extern void set_story_state(
    LouderContext* ctx,
    int32_t conversation,
    int32_t key,
    LouderVariant value
);
extern int32_t test_linkage(void);
/*
 * Push an event onto the event queue, used for testing.
 *
 * The caller fills in the event themselves, so nothing about `code` is interpreted here.
 * Returns LouderError::Ok on success, or LouderError::Missing if either pointer was null.
 */
extern LouderError test_push_event(LouderContext* ctx, const LouderEvent* event);
/* Layout, checked by the compiler rather than trusted. */
#if defined(__STDC_VERSION__) && __STDC_VERSION__ >= 201112L
#define LOUDER_ASSERT(name, cond) _Static_assert(cond, name)
#define LOUDER_ASSERT_FIELD(type, field, offset) \
    _Static_assert(offsetof(type, field) == offset, #type "." #field " is not at the offset the engine expects")
LOUDER_ASSERT("ActorFlags is not the size the engine expects", sizeof(ActorFlags) == 4);
LOUDER_ASSERT("AdditionalDataChange is not the size the engine expects", sizeof(AdditionalDataChange) == 4);
LOUDER_ASSERT("ChoiceAppearance is not the size the engine expects", sizeof(ChoiceAppearance) == 8);
LOUDER_ASSERT_FIELD(ChoiceAppearance, choice_index, 4);
LOUDER_ASSERT("LouderVariant_Body is not the size the engine expects", sizeof(LouderVariant_Body) == 4);
LOUDER_ASSERT("LouderVariant is not the size the engine expects", sizeof(LouderVariant) == 8);
LOUDER_ASSERT_FIELD(LouderVariant, body, 4);
LOUDER_ASSERT("ConditionOutcome is not the size the engine expects", sizeof(ConditionOutcome) == 12);
LOUDER_ASSERT_FIELD(ConditionOutcome, outcome, 4);
LOUDER_ASSERT("DialogueRules is not the size the engine expects", sizeof(DialogueRules) == 4);
LOUDER_ASSERT("Motion is not the size the engine expects", sizeof(Motion) == 16);
LOUDER_ASSERT_FIELD(Motion, target_position, 4);
LOUDER_ASSERT_FIELD(Motion, choice_index, 8);
LOUDER_ASSERT_FIELD(Motion, background, 12);
LOUDER_ASSERT("LoadFailure is not the size the engine expects", sizeof(LoadFailure) == 8);
LOUDER_ASSERT_FIELD(LoadFailure, error, 4);
LOUDER_ASSERT("EventData_Body is not the size the engine expects", sizeof(EventData_Body) == 16);
LOUDER_ASSERT("EventData is not the size the engine expects", sizeof(EventData) == 20);
LOUDER_ASSERT_FIELD(EventData, body, 4);
LOUDER_ASSERT("HubFlags is not the size the engine expects", sizeof(HubFlags) == 4);
LOUDER_ASSERT("LouderAsk is not the size the engine expects", sizeof(LouderAsk) == 32);
LOUDER_ASSERT_FIELD(LouderAsk, channel, 4);
LOUDER_ASSERT_FIELD(LouderAsk, actor, 8);
LOUDER_ASSERT_FIELD(LouderAsk, motion, 12);
LOUDER_ASSERT_FIELD(LouderAsk, ask, 28);
LOUDER_ASSERT("LouderEvent is not the size the engine expects", sizeof(LouderEvent) == 32);
LOUDER_ASSERT_FIELD(LouderEvent, channel, 4);
LOUDER_ASSERT_FIELD(LouderEvent, actor, 8);
LOUDER_ASSERT_FIELD(LouderEvent, data, 12);
LOUDER_ASSERT("LouderOption_LouderAsk_Body is not the size the engine expects", sizeof(LouderOption_LouderAsk_Body) == 32);
LOUDER_ASSERT("LouderOption_LouderAsk is not the size the engine expects", sizeof(LouderOption_LouderAsk) == 36);
LOUDER_ASSERT_FIELD(LouderOption_LouderAsk, body, 4);
LOUDER_ASSERT("LouderOption_LouderEvent_Body is not the size the engine expects", sizeof(LouderOption_LouderEvent_Body) == 32);
LOUDER_ASSERT("LouderOption_LouderEvent is not the size the engine expects", sizeof(LouderOption_LouderEvent) == 36);
LOUDER_ASSERT_FIELD(LouderOption_LouderEvent, body, 4);
LOUDER_ASSERT("SpeakerFlags is not the size the engine expects", sizeof(SpeakerFlags) == 4);
#endif

#endif /* LOUDER_H */
