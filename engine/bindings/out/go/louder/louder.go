// Bindings for Go, generated from the C header of the Louder engine.
//
// Do not edit. Run `python engine/bindings/generate.py` and this file comes back.
//
// One engine context and everything that comes with it.

package louder

// #cgo windows LDFLAGS: ${SRCDIR}/louder_rust.dll.lib
// #cgo linux LDFLAGS: -L${SRCDIR} -llouder_rust
// #cgo darwin LDFLAGS: -L${SRCDIR} -llouder_rust
// #include "louder.h"
import "C"

import (
    "errors"
    "fmt"
    "iter"
    "unsafe"
)

// The boundary's types, named as the header names them.
//
// Every one is an alias of the C type rather than a new type, because
// the two are the same thing and a caller should not have to convert.

type Context = C.LouderContext
type LoadError = C.LoadError
type LouderActor = C.LouderActor
type LouderCallback = C.LouderCallback
type LouderExpression = C.LouderExpression
type LouderName = C.LouderName
type LouderPosition = C.LouderPosition
type EventDataTag = C.EventData_Tag
type LouderError = C.LouderError
type LouderOptionLouderAskTag = C.LouderOption_LouderAsk_Tag
type LouderOptionLouderEventTag = C.LouderOption_LouderEvent_Tag
type LouderVariantTag = C.LouderVariant_Tag
type ActorFlags = C.ActorFlags
type AdditionalDataChange = C.AdditionalDataChange
type ChoiceAppearance = C.ChoiceAppearance
type ConditionOutcome = C.ConditionOutcome
type DialogueRules = C.DialogueRules
type HubFlags = C.HubFlags
type LoadFailure = C.LoadFailure
type LouderAsk = C.LouderAsk
type LouderEvent = C.LouderEvent
type Motion = C.Motion
type SpeakerFlags = C.SpeakerFlags
type EventData = C.EventData
type LouderOptionLouderAsk = C.LouderOption_LouderAsk
type LouderOptionLouderEvent = C.LouderOption_LouderEvent
type LouderVariant = C.LouderVariant

// The constants and the values of the enums.
const (
    LOUDER_ACTOR_NONE = C.LOUDER_ACTOR_NONE
    LOUDER_LOAD_ALREADY_LOADED = C.LOUDER_LOAD_ALREADY_LOADED
    LOUDER_LOAD_MALFORMED_JSON = C.LOUDER_LOAD_MALFORMED_JSON
    LOUDER_LOAD_NOT_AN_OBJECT = C.LOUDER_LOAD_NOT_AN_OBJECT
    LOUDER_LOAD_NOT_A_LIST = C.LOUDER_LOAD_NOT_A_LIST
    LOUDER_LOAD_OK = C.LOUDER_LOAD_OK
    LOUDER_NAME_NONE = C.LOUDER_NAME_NONE
)

const (
    EventDataNone = C.None
    EventDataRegularMotion = C.RegularMotion
    EventDataWarp = C.Warp
    EventDataForceMotion = C.ForceMotion
    EventDataChoiceAppearance = C.EventData_Tag_ChoiceAppearance
    EventDataChannelChange = C.ChannelChange
    EventDataAdditionalDataChange = C.EventData_Tag_AdditionalDataChange
    EventDataSpeakerChange = C.SpeakerChange
    EventDataTextChange = C.TextChange
    EventDataStatement = C.Statement
    EventDataCondition = C.Condition
    EventDataCallback = C.Callback
    EventDataActorAdded = C.ActorAdded
    EventDataActorRemoved = C.ActorRemoved
    EventDataSpeakerAdded = C.SpeakerAdded
    EventDataSpeakerRemoved = C.SpeakerRemoved
    EventDataConversationAdded = C.ConversationAdded
    EventDataConversationRemoved = C.ConversationRemoved
    EventDataLoadError = C.EventData_Tag_LoadError
)

const (
    ErrorOk = C.Ok
    ErrorMissing = C.Missing
)

const (
    AskNoneLouderAsk = C.None_LouderAsk
    AskSomeLouderAsk = C.Some_LouderAsk
)

const (
    EventNoneLouderEvent = C.None_LouderEvent
    EventSomeLouderEvent = C.Some_LouderEvent
)

const (
    VariantInt = C.Int
    VariantFloat = C.Float
    VariantString = C.String
)

// The flags of a actor.
const (
    ActorFlagsSpectator = C.ActorFlags_SPECTATOR
    ActorFlagsDebug = C.ActorFlags_DEBUG
)

// The flags of a DialogueRules.
const (
    DialogueRulesSpoke = C.DialogueRules_SPOKE
    DialogueRulesSingleplayer = C.DialogueRules_SINGLEPLAYER
)

// The flags of a hub.
const (
    HubFlagsAuto = C.HubFlags_AUTO
    HubFlagsAsk = C.HubFlags_ASK
)

// The flags of a speaker.
const (
    SpeakerFlagsPublic = C.SpeakerFlags_PUBLIC
)

// The raw entry points, one per function in the header.

// Add an actor to a conversation and return the integer id'ing it.
func AddActor(ctx *Context, conversation int32, start_position int32, flags ActorFlags, speak_as *C.int32_t, speak_as_count int32, listen_to *C.int32_t, listen_to_count int32) int32 {
    return int32(C.add_actor(ctx, C.int32_t(conversation), C.int32_t(start_position), flags, speak_as, C.int32_t(speak_as_count), listen_to, C.int32_t(listen_to_count)))
}

// Answer the ask at `index` with `verdict`.
func AnswerAsk(ctx *Context, index int, verdict int32) LouderError {
    return LouderError(C.answer_ask(ctx, C.size_t(index), C.int32_t(verdict)))
}

// Remove all speakers in the config.
func ClearConfig(ctx *Context) int32 {
    return int32(C.clear_config(ctx))
}

// Assign the id of a string.
func DeclareName(ctx *Context, buffer *C.uint8_t, max_size int) int32 {
    return int32(C.declare_name(ctx, buffer, C.size_t(max_size)))
}

// Release a context and everything it holds.
func DeinitLouder(ctx *Context) LouderError {
    return LouderError(C.deinit_louder(ctx))
}

// Remove every queued event.
func FlushEvents(ctx *Context) {
    C.flush_events(ctx)
}

// Move an actor to another position directly.
func ForcePosition(ctx *Context, conversation int32, actor int32, target_position int32, fake_channel int32, fake_priority int32) int32 {
    return int32(C.force_position(ctx, C.int32_t(conversation), C.int32_t(actor), C.int32_t(target_position), C.int32_t(fake_channel), C.int32_t(fake_priority)))
}

// Move an actor to another position as if it had moved through the network.
func ForceWarp(ctx *Context, conversation int32, actor int32, target_position int32, fake_channel int32, fake_priority int32) int32 {
    return int32(C.force_warp(ctx, C.int32_t(conversation), C.int32_t(actor), C.int32_t(target_position), C.int32_t(fake_channel), C.int32_t(fake_priority)))
}

// Get an actor's additional data under `key`.
func GetActorAdditional(ctx *Context, conversation int32, actor int32, key int32) LouderVariant {
    return LouderVariant(C.get_actor_additional(ctx, C.int32_t(conversation), C.int32_t(actor), C.int32_t(key)))
}

// Get the channel of an actor.
func GetActorChannel(ctx *Context, conversation int32, actor int32) int32 {
    return int32(C.get_actor_channel(ctx, C.int32_t(conversation), C.int32_t(actor)))
}

// Get the flags of an actor.
func GetActorFlags(ctx *Context, conversation int32, actor int32) ActorFlags {
    return ActorFlags(C.get_actor_flags(ctx, C.int32_t(conversation), C.int32_t(actor)))
}

// Get the position of an actor.
func GetActorPosition(ctx *Context, conversation int32, actor int32) int32 {
    return int32(C.get_actor_position(ctx, C.int32_t(conversation), C.int32_t(actor)))
}

// Get the priority of an actor.
func GetActorPriority(ctx *Context, conversation int32, actor int32) int32 {
    return int32(C.get_actor_priority(ctx, C.int32_t(conversation), C.int32_t(actor)))
}

// Get the ask at `index`.
func GetAsk(ctx *Context, index int) LouderOptionLouderAsk {
    return LouderOptionLouderAsk(C.get_ask(ctx, C.size_t(index)))
}

// Read the dialogue rules currently in effect.
func GetDialogueRules(ctx *Context) DialogueRules {
    return DialogueRules(C.get_dialogue_rules(ctx))
}

// Get the event at `index`.
func GetEvent(ctx *Context, index int) LouderOptionLouderEvent {
    return LouderOptionLouderEvent(C.get_event(ctx, C.size_t(index)))
}

// Get a hub's additional data under `key`.
func GetHubAdditional(ctx *Context, conversation int32, position int32, key int32) LouderVariant {
    return LouderVariant(C.get_hub_additional(ctx, C.int32_t(conversation), C.int32_t(position), C.int32_t(key)))
}

// Get the callbacks associated with a hub as a name.
func GetHubCallbacks(ctx *Context, conversation int32, position int32) int32 {
    return int32(C.get_hub_callbacks(ctx, C.int32_t(conversation), C.int32_t(position)))
}

// Get the channel of a hub.
func GetHubChannel(ctx *Context, conversation int32, position int32) int32 {
    return int32(C.get_hub_channel(ctx, C.int32_t(conversation), C.int32_t(position)))
}

// Get the number of choices on a hub from an actor's perspective.
func GetHubChoiceCount(ctx *Context, conversation int32, position int32, actor_perspective int32) int32 {
    return int32(C.get_hub_choice_count(ctx, C.int32_t(conversation), C.int32_t(position), C.int32_t(actor_perspective)))
}

// Get the position a choice on a hub links to.
func GetHubChoiceLink(ctx *Context, conversation int32, position int32, actor_perspective int32, choice_index int32) int32 {
    return int32(C.get_hub_choice_link(ctx, C.int32_t(conversation), C.int32_t(position), C.int32_t(actor_perspective), C.int32_t(choice_index)))
}

// Get the text of a choice on a hub as a name.
func GetHubChoiceText(ctx *Context, conversation int32, position int32, actor_perspective int32, choice_index int32) int32 {
    return int32(C.get_hub_choice_text(ctx, C.int32_t(conversation), C.int32_t(position), C.int32_t(actor_perspective), C.int32_t(choice_index)))
}

// Get the expressions associated with a hub as a name.
func GetHubExpressions(ctx *Context, conversation int32, position int32) int32 {
    return int32(C.get_hub_expressions(ctx, C.int32_t(conversation), C.int32_t(position)))
}

// Get the flags of a hub.
func GetHubFlags(ctx *Context, conversation int32, position int32) HubFlags {
    return HubFlags(C.get_hub_flags(ctx, C.int32_t(conversation), C.int32_t(position)))
}

// Get the priority of a hub.
func GetHubPriority(ctx *Context, conversation int32, position int32) int32 {
    return int32(C.get_hub_priority(ctx, C.int32_t(conversation), C.int32_t(position)))
}

// Get the speaker of a hub.
func GetHubSpeaker(ctx *Context, conversation int32, position int32) int32 {
    return int32(C.get_hub_speaker(ctx, C.int32_t(conversation), C.int32_t(position)))
}

// Get the text of a hub as a name.
func GetHubText(ctx *Context, conversation int32, position int32) int32 {
    return int32(C.get_hub_text(ctx, C.int32_t(conversation), C.int32_t(position)))
}

// Get the id of a string.
func GetNamedId(ctx *Context, buffer *C.uint8_t, max_size int) int32 {
    return int32(C.get_named_id(ctx, buffer, C.size_t(max_size)))
}

// Iterate the speakers that `speaker` is listening to.
func GetNextSpeakerListenTo(ctx *Context, speaker int32, start int32) int32 {
    return int32(C.get_next_speaker_listen_to(ctx, C.int32_t(speaker), C.int32_t(start)))
}

// Get a speaker's additional data under `key`.
func GetSpeakerAdditional(ctx *Context, speaker int32, key int32) LouderVariant {
    return LouderVariant(C.get_speaker_additional(ctx, C.int32_t(speaker), C.int32_t(key)))
}

// Get the flags of a speaker.
func GetSpeakerFlags(ctx *Context, speaker int32) SpeakerFlags {
    return SpeakerFlags(C.get_speaker_flags(ctx, C.int32_t(speaker)))
}

// Get a story state value under `state`.
func GetStoryState(ctx *Context, conversation int32, state int32) LouderVariant {
    return LouderVariant(C.get_story_state(ctx, C.int32_t(conversation), C.int32_t(state)))
}

// Initialize the engine and return a new context.
func InitLouder() *C.uint8_t {
    return *C.uint8_t(C.init_louder())
}

// Load the conversation config from a JSON document in `buffer`.
func LoadConversationConfig(ctx *Context, buffer *C.uint8_t, max_size int) int32 {
    return int32(C.load_conversation_config(ctx, buffer, C.size_t(max_size)))
}

// Load a conversation from a JSON document in `buffer`.
func LoadConversationJson(ctx *Context, conversation int32, buffer *C.uint8_t, max_size int) int32 {
    return int32(C.load_conversation_json(ctx, C.int32_t(conversation), buffer, C.size_t(max_size)))
}

// Build an EventData::ActorAdded..
func MakeEventDataActorAdded() EventData {
    return EventData(C.make_event_data_actor_added())
}

// Build an EventData::ActorRemoved..
func MakeEventDataActorRemoved() EventData {
    return EventData(C.make_event_data_actor_removed())
}

// Build an EventData::AdditionalDataChange..
func MakeEventDataAdditionalDataChange(change AdditionalDataChange) EventData {
    return EventData(C.make_event_data_additional_data_change(change))
}

// Build an EventData::Callback, naming the callback..
func MakeEventDataCallback(callback int32) EventData {
    return EventData(C.make_event_data_callback(C.int32_t(callback)))
}

// Build an EventData::ChannelChange..
func MakeEventDataChannelChange() EventData {
    return EventData(C.make_event_data_channel_change())
}

// Build an EventData::ChoiceAppearance..
func MakeEventDataChoiceAppearance(appearance ChoiceAppearance) EventData {
    return EventData(C.make_event_data_choice_appearance(appearance))
}

// Build an EventData::Condition..
func MakeEventDataCondition(outcome ConditionOutcome) EventData {
    return EventData(C.make_event_data_condition(outcome))
}

// Build an EventData::ConversationAdded, naming the conversation..
func MakeEventDataConversationAdded(conversation int32) EventData {
    return EventData(C.make_event_data_conversation_added(C.int32_t(conversation)))
}

// Build an EventData::ConversationRemoved, naming the conversation..
func MakeEventDataConversationRemoved(conversation int32) EventData {
    return EventData(C.make_event_data_conversation_removed(C.int32_t(conversation)))
}

// Build an EventData::ForceMotion..
func MakeEventDataForceMotion(motion Motion) EventData {
    return EventData(C.make_event_data_force_motion(motion))
}

// Build an EventData::LoadError.
func MakeEventDataLoadError(failure LoadFailure) EventData {
    return EventData(C.make_event_data_load_error(failure))
}

// Build the EventData::None variant, which carries nothing..
func MakeEventDataNone() EventData {
    return EventData(C.make_event_data_none())
}

// Build an EventData::RegularMotion..
func MakeEventDataRegularMotion(motion Motion) EventData {
    return EventData(C.make_event_data_regular_motion(motion))
}

// Build an EventData::SpeakerAdded, naming the speaker..
func MakeEventDataSpeakerAdded(speaker int32) EventData {
    return EventData(C.make_event_data_speaker_added(C.int32_t(speaker)))
}

// Build an EventData::SpeakerChange, naming the speaker..
func MakeEventDataSpeakerChange(speaker int32) EventData {
    return EventData(C.make_event_data_speaker_change(C.int32_t(speaker)))
}

// Build an EventData::SpeakerRemoved, naming the speaker..
func MakeEventDataSpeakerRemoved(speaker int32) EventData {
    return EventData(C.make_event_data_speaker_removed(C.int32_t(speaker)))
}

// Build an EventData::Statement, naming the expression..
func MakeEventDataStatement(expression int32) EventData {
    return EventData(C.make_event_data_statement(C.int32_t(expression)))
}

// Build an EventData::TextChange, naming the text..
func MakeEventDataTextChange(text int32) EventData {
    return EventData(C.make_event_data_text_change(C.int32_t(text)))
}

// Build an EventData::Warp..
func MakeEventDataWarp(motion Motion) EventData {
    return EventData(C.make_event_data_warp(motion))
}

// Build a LouderOption::Some holding an ask..
func MakeSomeAsk(value LouderAsk) LouderOptionLouderAsk {
    return LouderOptionLouderAsk(C.make_some_ask(value))
}

// Build a LouderOption::Some holding an event..
func MakeSomeEvent(value LouderEvent) LouderOptionLouderEvent {
    return LouderOptionLouderEvent(C.make_some_event(value))
}

// Build a boolean as a LouderVariant::Int of 1 or 0..
func MakeVariantBool(value bool) LouderVariant {
    return LouderVariant(C.make_variant_bool(C.bool(value)))
}

// Build a LouderVariant::Float..
func MakeVariantFloat(value float32) LouderVariant {
    return LouderVariant(C.make_variant_float(C.float(value)))
}

// Build a LouderVariant::Int..
func MakeVariantInt(value int32) LouderVariant {
    return LouderVariant(C.make_variant_int(C.int32_t(value)))
}

// Build a LouderVariant::String, naming the string rather than carrying it..
func MakeVariantString(value int32) LouderVariant {
    return LouderVariant(C.make_variant_string(C.int32_t(value)))
}

// Build the LouderVariant::Int of 0 that an unwritten variable reads as..
func MakeVariantZero() LouderVariant {
    return LouderVariant(C.make_variant_zero())
}

// Iterate the actors matching certain flags.
func NextActorFromFlags(ctx *Context, conversation int32, include ActorFlags, exact int32, start int32) int32 {
    return int32(C.next_actor_from_flags(ctx, C.int32_t(conversation), include, C.int32_t(exact), C.int32_t(start)))
}

// Iterate the actors in a channel.
func NextActorInChannel(ctx *Context, conversation int32, channel int32, start int32) int32 {
    return int32(C.next_actor_in_channel(ctx, C.int32_t(conversation), C.int32_t(channel), C.int32_t(start)))
}

// Iterate the actors in a channel at a specific position.
func NextActorInChannelAndPosition(ctx *Context, conversation int32, channel int32, position int32, start int32) int32 {
    return int32(C.next_actor_in_channel_and_position(ctx, C.int32_t(conversation), C.int32_t(channel), C.int32_t(position), C.int32_t(start)))
}

// Iterate the speakers an actor listens to.
func NextActorListening(ctx *Context, conversation int32, actor int32, start int32) int32 {
    return int32(C.next_actor_listening(ctx, C.int32_t(conversation), C.int32_t(actor), C.int32_t(start)))
}

// Iterate the base speakers an actor listens to.
func NextActorListeningBase(ctx *Context, conversation int32, actor int32, start int32) int32 {
    return int32(C.next_actor_listening_base(ctx, C.int32_t(conversation), C.int32_t(actor), C.int32_t(start)))
}

// Iterate the actors listening to a speaker.
func NextActorListeningTo(ctx *Context, conversation int32, speaker int32, start int32) int32 {
    return int32(C.next_actor_listening_to(ctx, C.int32_t(conversation), C.int32_t(speaker), C.int32_t(start)))
}

// Iterate the speakers an actor speaks as.
func NextActorSpeakAs(ctx *Context, conversation int32, actor int32, start int32) int32 {
    return int32(C.next_actor_speak_as(ctx, C.int32_t(conversation), C.int32_t(actor), C.int32_t(start)))
}

// Iterate the actors speaking as a speaker.
func NextActorSpeakingAs(ctx *Context, conversation int32, speaker int32, start int32) int32 {
    return int32(C.next_actor_speaking_as(ctx, C.int32_t(conversation), C.int32_t(speaker), C.int32_t(start)))
}

// Iterate the hubs in a channel.
func NextHubByChannel(ctx *Context, conversation int32, channel int32, start int32) int32 {
    return int32(C.next_hub_by_channel(ctx, C.int32_t(conversation), C.int32_t(channel), C.int32_t(start)))
}

// Iterate the hubs with a priority within `[priority_min, priority_max]`.
func NextHubByPriority(ctx *Context, conversation int32, priority_min int32, priority_max int32, start int32) int32 {
    return int32(C.next_hub_by_priority(ctx, C.int32_t(conversation), C.int32_t(priority_min), C.int32_t(priority_max), C.int32_t(start)))
}

// Iterate the hubs matching certain flags.
func NextHubFromFlags(ctx *Context, conversation int32, include HubFlags, exact int32, start int32) int32 {
    return int32(C.next_hub_from_flags(ctx, C.int32_t(conversation), include, C.int32_t(exact), C.int32_t(start)))
}

// Iterate the hubs containing all, or exactly, the given tags.
func NextHubFromTags(ctx *Context, conversation int32, include *C.int32_t, include_count int32, exact int32, start int32) int32 {
    return int32(C.next_hub_from_tags(ctx, C.int32_t(conversation), include, C.int32_t(include_count), C.int32_t(exact), C.int32_t(start)))
}

// Iterate the hubs heard by a speaker.
func NextHubHeardBy(ctx *Context, conversation int32, speaker int32, start int32) int32 {
    return int32(C.next_hub_heard_by(ctx, C.int32_t(conversation), C.int32_t(speaker), C.int32_t(start)))
}

// Iterate the recipients of a hub.
func NextHubRecipient(ctx *Context, conversation int32, position int32, start int32) int32 {
    return int32(C.next_hub_recipient(ctx, C.int32_t(conversation), C.int32_t(position), C.int32_t(start)))
}

// Iterate the hubs spoken by a speaker.
func NextHubSpokenBy(ctx *Context, conversation int32, speaker int32, start int32) int32 {
    return int32(C.next_hub_spoken_by(ctx, C.int32_t(conversation), C.int32_t(speaker), C.int32_t(start)))
}

// Iterate the tags of a position.
func NextHubTag(ctx *Context, conversation int32, position int32, start int32) int32 {
    return int32(C.next_hub_tag(ctx, C.int32_t(conversation), C.int32_t(position), C.int32_t(start)))
}

// Push an ask onto the ask cache, used for testing.
func PushAsk(ctx *Context, ask *LouderAsk) LouderError {
    return LouderError(C.push_ask(ctx, ask))
}

// Read the bytes associated with an id into `buffer`.
func ReadName(ctx *Context, name int32, buffer *C.uint8_t, max_size int) int {
    return int(C.read_name(ctx, C.int32_t(name), buffer, C.size_t(max_size)))
}

// Serialize the story state, along with the positions and details of every actor, in JSON into `buffer`.
func ReadSnapshotJson(ctx *Context, conversation int32, buffer *C.uint8_t, max_size int) int32 {
    return int32(C.read_snapshot_json(ctx, C.int32_t(conversation), buffer, C.size_t(max_size)))
}

// Serialize the story state in JSON into `buffer`.
func ReadStateJson(ctx *Context, conversation int32, buffer *C.uint8_t, max_size int) int32 {
    return int32(C.read_state_json(ctx, C.int32_t(conversation), buffer, C.size_t(max_size)))
}

// Remove an actor from a conversation.
func RemoveActor(ctx *Context, conversation int32, actor int32) int32 {
    return int32(C.remove_actor(ctx, C.int32_t(conversation), C.int32_t(actor)))
}

// Remove a conversation and all associated actors.
func RemoveConversation(ctx *Context, conversation int32) int32 {
    return int32(C.remove_conversation(ctx, C.int32_t(conversation)))
}

// Select a choice on an actor's current hub.
func SelectChoice(ctx *Context, conversation int32, actor int32, choice_index int32) int32 {
    return int32(C.select_choice(ctx, C.int32_t(conversation), C.int32_t(actor), C.int32_t(choice_index)))
}

// Replace the dialogue rules.
func SetDialogueRules(ctx *Context, rules DialogueRules) int32 {
    return int32(C.set_dialogue_rules(ctx, rules))
}

// Set a story state value.
func SetStoryState(ctx *Context, conversation int32, key int32, value LouderVariant) {
    C.set_story_state(ctx, C.int32_t(conversation), C.int32_t(key), value)
}

// TestLinkage calls test_linkage.
func TestLinkage() int32 {
    return int32(C.test_linkage())
}

// Push an event onto the event queue, used for testing.
func TestPushEvent(ctx *Context, event *LouderEvent) LouderError {
    return LouderError(C.test_push_event(ctx, event))
}

// Louder is one engine context.
//
// It owns the name table, every conversation, and the event queue.
// Close it when you are done; there is no finalizer, because the engine
// cannot be closed from the garbage collector's thread.
type Louder struct {
    ctx *C.LouderContext
}

// Open starts an engine context.
func Open() (*Louder, error) {
    ctx := C.init_louder()
    if ctx == nil {
        return nil, errors.New("the engine did not start")
    }
    return &Louder{ctx: ctx}, nil
}

// Close releases the context and everything it holds.
func (l *Louder) Close() {
    if l.ctx != nil {
        C.deinit_louder(l.ctx)
        l.ctx = nil
    }
}

// Handle is the raw context pointer, for anything this layer does not wrap.
func (l *Louder) Handle() *C.LouderContext { return l.ctx }

// Lookup answers the id of some text, or LOUDER_NAME_NONE if it was never
// declared.
func (l *Louder) Lookup(text string) LouderName {
    raw, length := bytes(text)
    return C.get_named_id(l.ctx, raw, length)
}

// Name answers the id of some text, declaring it the first time.
//
// declare_name refuses to make a second id for text that already has one,
// returning LOUDER_NAME_NONE, so this looks first and declares second.
func (l *Louder) Name(text string) LouderName {
    found := l.Lookup(text)
    if found != LOUDER_NAME_NONE {
        return found
    }
    raw, length := bytes(text)
    return C.declare_name(l.ctx, raw, length)
}

// Text answers the bytes behind an id, decoded as a Go string.
//
// The engine reports how many bytes fitted rather than refusing, so the
// only way to be sure of the whole answer is to ask again with more room.
// Two calls is the ordinary case.
func (l *Louder) Text(id LouderName) string {
    if id == LOUDER_NAME_NONE {
        return ""
    }
    for size := 4096; ; size *= 4 {
        buffer := make([]byte, size)
        got := C.read_name(l.ctx, id, (*C.uint8_t)(unsafe.Pointer(&buffer[0])), C.size_t(size-1))
        if got == 0 {
            return ""
        }
        if int(got)+1 < size {
            return string(buffer[:int(got)])
        }
    }
}

// Failure is why a document was refused.
type Failure struct { Code int32 }

func (f Failure) Error() string {
    switch f.Code {
    case LOUDER_LOAD_MALFORMED_JSON:
        return "the document is not JSON"
    case LOUDER_LOAD_NOT_A_LIST:
        return "the document is not a list"
    case LOUDER_LOAD_NOT_AN_OBJECT:
        return "the document is not an object"
    case LOUDER_LOAD_ALREADY_LOADED:
        return "a conversation by that name is already loaded"
    default:
        return fmt.Sprintf("the engine refused the document (%d)", f.Code)
    }
}

// LoadConversation loads a conversation from a JSON document.
func (l *Louder) LoadConversation(name LouderName, document string) error {
    raw, length := bytes(document)
    if code := C.load_conversation_json(l.ctx, name, raw, length); code != LOUDER_LOAD_OK {
        return Failure{Code: int32(code)}
    }
    return nil
}

// LoadConfig loads the speaker config, which is a singleton.
func (l *Louder) LoadConfig(document string) error {
    raw, length := bytes(document)
    if code := C.load_conversation_config(l.ctx, raw, length); code != LOUDER_LOAD_OK {
        return Failure{Code: int32(code)}
    }
    return nil
}

// StateJSON is the whole story state as a JSON document.
func (l *Louder) StateJSON(conversation LouderName) (string, error) {
    for size := 4096; ; size *= 4 {
        buffer := make([]byte, size)
        written := C.read_state_json(l.ctx, conversation, (*C.uint8_t)(unsafe.Pointer(&buffer[0])), C.size_t(size-1))
        if written < 0 {
            return "", errors.New("the conversation is not loaded")
        }
        if int(written)+1 < size {
            return string(buffer[:int(written)]), nil
        }
    }
}

// SnapshotJSON is the story state and every actor, as a JSON document.
func (l *Louder) SnapshotJSON(conversation LouderName) (string, error) {
    for size := 4096; ; size *= 4 {
        buffer := make([]byte, size)
        written := C.read_snapshot_json(l.ctx, conversation, (*C.uint8_t)(unsafe.Pointer(&buffer[0])), C.size_t(size-1))
        if written < 0 {
            return "", errors.New("the conversation is not loaded")
        }
        if int(written)+1 < size {
            return string(buffer[:int(written)]), nil
        }
    }
}

// Variant is one value from the story state, decoded.
type Variant struct {
    Kind LouderVariantTag
    Int   int32
    Float float32
    Name  LouderName
}

// Variant answers the story state value under key, decoded.
func (l *Louder) Variant(conversation LouderName, key string) Variant {
    raw := C.get_story_state(l.ctx, conversation, l.Name(key))
    out := Variant{Kind: LouderVariantTag(raw.tag)}
    switch out.Kind {
    case VariantInt:
        out.Int = *(*int32)(unsafe.Pointer(&raw.body[0]))
    case VariantFloat:
        out.Float = *(*float32)(unsafe.Pointer(&raw.body[0]))
    case VariantString:
        out.Name = *(*LouderName)(unsafe.Pointer(&raw.body[0]))
    }
    return out
}

// SetInt writes an integer story state value.
func (l *Louder) SetInt(conversation LouderName, key string, value int32) {
    C.set_story_state(l.ctx, conversation, l.Name(key), C.make_variant_int(C.int32_t(value)))
}

// SetFloat writes a float story state value.
func (l *Louder) SetFloat(conversation LouderName, key string, value float32) {
    C.set_story_state(l.ctx, conversation, l.Name(key), C.make_variant_float(C.float(value)))
}

// SetText writes a text story state value, stored as the interned name.
func (l *Louder) SetText(conversation LouderName, key string, value string) {
    C.set_story_state(l.ctx, conversation, l.Name(key), C.make_variant_string(l.Name(value)))
}

// bytes hands a Go string over as the pointer and length the engine wants.
//
// The empty string has no first element to point at, and the engine accepts
// a null pointer when the length is zero.
func bytes(text string) (*C.uint8_t, C.size_t) {
    if len(text) == 0 {
        return nil, 0
    }
    return (*C.uint8_t)(unsafe.Pointer(unsafe.StringData(text))), C.size_t(len(text))
}

// SpeakerListenTo walks speaker_listen_to.
//
// Iterate the speakers that `speaker` is listening to.
func (l *Louder) SpeakerListenTo(ctx *Context, speaker int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.get_next_speaker_listen_to(l.ctx, ctx, C.int32_t(speaker), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// ActorFromFlags walks actor_from_flags.
//
// Iterate the actors matching certain flags.
func (l *Louder) ActorFromFlags(ctx *Context, conversation int32, include ActorFlags, exact int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_actor_from_flags(l.ctx, ctx, C.int32_t(conversation), include, C.int32_t(exact), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// ActorInChannel walks actor_in_channel.
//
// Iterate the actors in a channel.
func (l *Louder) ActorInChannel(ctx *Context, conversation int32, channel int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_actor_in_channel(l.ctx, ctx, C.int32_t(conversation), C.int32_t(channel), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// ActorInChannelAndPosition walks actor_in_channel_and_position.
//
// Iterate the actors in a channel at a specific position.
func (l *Louder) ActorInChannelAndPosition(ctx *Context, conversation int32, channel int32, position int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_actor_in_channel_and_position(l.ctx, ctx, C.int32_t(conversation), C.int32_t(channel), C.int32_t(position), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// ActorListening walks actor_listening.
//
// Iterate the speakers an actor listens to.
func (l *Louder) ActorListening(ctx *Context, conversation int32, actor int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_actor_listening(l.ctx, ctx, C.int32_t(conversation), C.int32_t(actor), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// ActorListeningBase walks actor_listening_base.
//
// Iterate the base speakers an actor listens to.
func (l *Louder) ActorListeningBase(ctx *Context, conversation int32, actor int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_actor_listening_base(l.ctx, ctx, C.int32_t(conversation), C.int32_t(actor), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// ActorListeningTo walks actor_listening_to.
//
// Iterate the actors listening to a speaker.
func (l *Louder) ActorListeningTo(ctx *Context, conversation int32, speaker int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_actor_listening_to(l.ctx, ctx, C.int32_t(conversation), C.int32_t(speaker), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// ActorSpeakAs walks actor_speak_as.
//
// Iterate the speakers an actor speaks as.
func (l *Louder) ActorSpeakAs(ctx *Context, conversation int32, actor int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_actor_speak_as(l.ctx, ctx, C.int32_t(conversation), C.int32_t(actor), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// ActorSpeakingAs walks actor_speaking_as.
//
// Iterate the actors speaking as a speaker.
func (l *Louder) ActorSpeakingAs(ctx *Context, conversation int32, speaker int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_actor_speaking_as(l.ctx, ctx, C.int32_t(conversation), C.int32_t(speaker), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// HubByChannel walks hub_by_channel.
//
// Iterate the hubs in a channel.
func (l *Louder) HubByChannel(ctx *Context, conversation int32, channel int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_hub_by_channel(l.ctx, ctx, C.int32_t(conversation), C.int32_t(channel), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// HubByPriority walks hub_by_priority.
//
// Iterate the hubs with a priority within `[priority_min, priority_max]`.
func (l *Louder) HubByPriority(ctx *Context, conversation int32, priority_min int32, priority_max int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_hub_by_priority(l.ctx, ctx, C.int32_t(conversation), C.int32_t(priority_min), C.int32_t(priority_max), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// HubFromFlags walks hub_from_flags.
//
// Iterate the hubs matching certain flags.
func (l *Louder) HubFromFlags(ctx *Context, conversation int32, include HubFlags, exact int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_hub_from_flags(l.ctx, ctx, C.int32_t(conversation), include, C.int32_t(exact), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// HubFromTags walks hub_from_tags.
//
// Iterate the hubs containing all, or exactly, the given tags.
func (l *Louder) HubFromTags(ctx *Context, conversation int32, include *C.int32_t, include_count int32, exact int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_hub_from_tags(l.ctx, ctx, C.int32_t(conversation), include, C.int32_t(include_count), C.int32_t(exact), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// HubHeardBy walks hub_heard_by.
//
// Iterate the hubs heard by a speaker.
func (l *Louder) HubHeardBy(ctx *Context, conversation int32, speaker int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_hub_heard_by(l.ctx, ctx, C.int32_t(conversation), C.int32_t(speaker), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// HubRecipient walks hub_recipient.
//
// Iterate the recipients of a hub.
func (l *Louder) HubRecipient(ctx *Context, conversation int32, position int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_hub_recipient(l.ctx, ctx, C.int32_t(conversation), C.int32_t(position), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// HubSpokenBy walks hub_spoken_by.
//
// Iterate the hubs spoken by a speaker.
func (l *Louder) HubSpokenBy(ctx *Context, conversation int32, speaker int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_hub_spoken_by(l.ctx, ctx, C.int32_t(conversation), C.int32_t(speaker), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

// HubTag walks hub_tag.
//
// Iterate the tags of a position.
func (l *Louder) HubTag(ctx *Context, conversation int32, position int32) iter.Seq[int32] {
    return func(yield func(int32) bool) {
        for start := int32(-1); ; {
            found := C.next_hub_tag(l.ctx, ctx, C.int32_t(conversation), C.int32_t(position), start)
            if found == int32(-1) {
                return
            }
            if !yield(found) {
                return
            }
            start = found
        }
    }
}

