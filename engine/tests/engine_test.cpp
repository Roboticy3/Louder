// End-to-end tests for the Louder engine, driven through the generated C++
// header and the built DLL.
//
// Run (from engine/ directory): ./build_and_test.ps1

#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

// The unions come out of the generated header as a tag and an anonymous union,
// so the engine builds them through make_* rather than leaving the layout to
// this side to repeat.
#include "louder_rust.h"

static int g_failures = 0;
static int g_checks = 0;

#define CHECK(condition, ...)                                     \
  do {                                                            \
    g_checks++;                                                   \
    if (!(condition)) {                                           \
      g_failures++;                                               \
      std::printf("  FAIL %s:%d: ", __FILE__, __LINE__);          \
      std::printf(__VA_ARGS__);                                   \
      std::printf("\n");                                          \
    }                                                             \
  } while (0)

namespace {

LouderContext* g_ctx = nullptr;

// The integer a variant carries, or -1 if it is carrying something else.
int32_t AsInt(LouderVariant variant) {
  if (variant.tag != LouderVariant::Tag::Int) {
    return -1;
  }
  return variant.int_._0;
}

// An event with nothing attached to it, carrying only the data given.
LouderEvent BareEvent(EventData data) {
  LouderEvent event{};
  event.conversation = LOUDER_NAME_NONE;
  event.channel = LOUDER_NAME_NONE;
  event.actor = LOUDER_ACTOR_NONE;
  event.data = data;
  return event;
}

// Read a name back out of the engine as a std::string.
std::string ReadName(LouderName name) {
  std::vector<uint8_t> buffer(4096, 0);
  size_t written = read_name(g_ctx, name, buffer.data(), buffer.size() - 1, 0);
  return std::string(reinterpret_cast<char*>(buffer.data()), written);
}

// Look a name up without creating it.
LouderName Lookup(const std::string& text) {
  return get_named_id(g_ctx, reinterpret_cast<const uint8_t*>(text.data()), text.size());
}

// Declare a name, reusing it if the engine already knows it.
LouderName Declare(const std::string& text) {
  LouderName existing = get_named_id(g_ctx, reinterpret_cast<const uint8_t*>(text.data()), text.size());
  if (existing != LOUDER_NAME_NONE) {
    return existing;
  }
  return declare_name(g_ctx, reinterpret_cast<const uint8_t*>(text.data()), text.size());
}

std::string ReadJson(LouderName (*fn)(const LouderContext*, LouderName, uint8_t*, size_t, LouderEncoding),
                    LouderName conversation) {
  std::vector<uint8_t> buffer(65536, 0);
  int32_t written = fn(g_ctx, conversation, buffer.data(), buffer.size() - 1, 0);
  if (written < 0) {
    return "<error>";
  }
  return std::string(reinterpret_cast<char*>(buffer.data()), written);
}

// Every queued event, drained from index 0 upwards.
std::vector<LouderEvent> DrainEvents() {
  std::vector<LouderEvent> events;
  for (size_t i = 0; ; ++i) {
    LouderOption<LouderEvent> option = get_event(g_ctx, i);
    if (option.tag == LouderOption<LouderEvent>::Tag::None) {
      break;
    }
    events.push_back(option.some._0);
  }
  flush_events(g_ctx);
  return events;
}

int CountEventsOfKind(const std::vector<LouderEvent>& events, EventData::Tag tag) {
  int count = 0;
  for (const auto& event : events) {
    if (event.data.tag == tag) {
      ++count;
    }
  }
  return count;
}

const char* ConversationJson() {
  return R"([
    {"type":"hub","conversation_id":0,"speaker":"A","text":"Hello","tags":"start main","flags":0},
    {"type":"hub","conversation_id":5,"speaker":"A","text":"World","flags":0},
    {"type":"hub","conversation_id":9,"text":"The end","callbacks":"exit","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":5},
    {"type":"connection","from_id":5,"from_port":0,"to_id":9},
    {"type":"camera","position:x":"0.0","position:y":"0.0","zoom":"1.0"}
  ])";
}

const char* ConfigJson() {
  return R"([
    {"type":"speaker","name":"A","flags":1},
    {"type":"speaker","name":"B","flags":0},
    {"type":"speaker","name":"Quiet","flags":0,"listening":"A\nB"}
  ])";
}

void TestNames() {
  std::printf("names\n");

  LouderName alpha = Declare("alpha");
  CHECK(alpha != LOUDER_NAME_NONE, "declaring a new name failed");

  LouderName again = declare_name(g_ctx, reinterpret_cast<const uint8_t*>("alpha"), 5);
  CHECK(again == LOUDER_NAME_NONE, "re-declaring a name should report it already exists");

  CHECK(Lookup("alpha") == alpha, "looking a name up should find the declared id");
  CHECK(Lookup("never declared") == LOUDER_NAME_NONE, "an undeclared name should not resolve");
  CHECK(ReadName(alpha) == "alpha", "reading a name back should round-trip");
  CHECK(ReadName(LOUDER_NAME_NONE) == "", "an absent name reads as empty");
}

void TestLoadConversation() {
  std::printf("load conversation\n");

  LouderName conversation = Declare("conversation under test");
  LoadError error = load_conversation_json(g_ctx, conversation,
                                           reinterpret_cast<uint8_t*>(const_cast<char*>(ConversationJson())),
                                           std::strlen(ConversationJson()), 0);
  CHECK(error == 0, "loading a well formed conversation returned %d", error);

  LoadError again = load_conversation_json(g_ctx, conversation,
                                           reinterpret_cast<uint8_t*>(const_cast<char*>(ConversationJson())),
                                           std::strlen(ConversationJson()), 0);
  CHECK(again != 0, "loading over an occupied conversation name should fail");

  // Positions are sparse and unordered in the document: 0, 5, 9.
  CHECK(get_hub_text(g_ctx, conversation, 0) != LOUDER_NAME_NONE, "hub 0 should have text");
  CHECK(ReadName(get_hub_text(g_ctx, conversation, 0)) == "Hello", "hub 0 text should read back");
  CHECK(ReadName(get_hub_speaker(g_ctx, conversation, 0)) == "A", "hub 0 should be spoken by A");
  CHECK(ReadName(get_hub_text(g_ctx, conversation, 9)) == "The end", "hub 9 text should read back");
  CHECK(get_hub_speaker(g_ctx, conversation, 9) == LOUDER_NAME_NONE, "hub 9 has no speaker");

  CHECK(ReadName(get_hub_callbacks(g_ctx, conversation, 9)) == "exit", "hub 9 should carry its callback");
  CHECK(get_hub_callbacks(g_ctx, conversation, 0) == LOUDER_NAME_NONE, "hub 0 has no callbacks");

  // Iteration walks positions in ascending order despite the gaps.
  std::vector<LouderPosition> positions;
  for (LouderPosition p = next_hub_by_channel(g_ctx, conversation, Declare("main"), -1);
       p != -1; p = next_hub_by_channel(g_ctx, conversation, Declare("main"), p)) {
    positions.push_back(p);
  }
  CHECK(positions.size() == 3, "three hubs should be in the main channel, saw %zu", positions.size());
  CHECK(positions.size() == 3 && positions[0] == 0 && positions[1] == 5 && positions[2] == 9,
        "hubs should iterate in ascending position order");

  // Tags.
  LouderName start_tag = Lookup("start");
  CHECK(start_tag != LOUDER_NAME_NONE, "the start tag should be interned");
  CHECK(next_hub_from_tags(g_ctx, conversation, &start_tag, 1, 0, -1) == 0,
        "the tagged hub should be found from a blank start");

  LouderName tags[2] = {Lookup("start"), Lookup("main")};
  CHECK(next_hub_from_tags(g_ctx, conversation, tags, 2, 1, -1) == 0,
        "an exact tag match should find hub 0");

  // A bad document is rejected without disturbing the loaded conversation.
  const char* garbage = "not json at all";
  LoadError bad = load_conversation_json(g_ctx, Declare("garbage conversation"),
                                        reinterpret_cast<uint8_t*>(const_cast<char*>(garbage)),
                                        std::strlen(garbage), 0);
  CHECK(bad != 0, "malformed JSON should fail to load");
}

void TestConfig() {
  std::printf("speaker config\n");

  LoadError error = load_conversation_config(g_ctx,
                                             reinterpret_cast<uint8_t*>(const_cast<char*>(ConfigJson())),
                                             std::strlen(ConfigJson()), 0);
  CHECK(error == 0, "loading the config returned %d", error);

  LouderName a = Lookup("A");
  LouderName b = Lookup("B");
  CHECK(a != LOUDER_NAME_NONE && b != LOUDER_NAME_NONE, "config speakers should be interned");

  CHECK(get_speaker_flags(g_ctx, a).bits == SpeakerFlags_PUBLIC.bits, "A is declared public");
  CHECK(get_speaker_flags(g_ctx, b).bits == 0, "B is not declared public");
  CHECK(get_speaker_flags(g_ctx, Lookup("Undeclared")).bits == 0,
        "an undeclared speaker defaults to no flags, which reads as public");

  // Quiet listens to A then B.
  LouderName quiet = Lookup("Quiet");
  CHECK(get_next_speaker_listen_to(g_ctx, quiet, LOUDER_NAME_NONE) == a,
        "Quiet should listen to A first");
  CHECK(get_next_speaker_listen_to(g_ctx, quiet, a) == b, "Quiet should then listen to B");
  CHECK(get_next_speaker_listen_to(g_ctx, quiet, b) == LOUDER_NAME_NONE,
        "iteration should end after the last entry");
}

void TestStoryState() {
  std::printf("story state\n");

  LouderName conversation = Lookup("conversation under test");

  LouderName local_lower = Declare("local_lower");
  set_story_state(g_ctx, conversation, local_lower, make_variant_int(7));
  CHECK(AsInt(get_story_state(g_ctx, conversation, local_lower)) == 7, "a local variable should store its value");

  LouderName shared_upper = Declare("Shared");
  set_story_state(g_ctx, conversation, shared_upper, make_variant_int(9));
  CHECK(AsInt(get_story_state(g_ctx, conversation, shared_upper)) == 9, "a shared variable should store its value");

  // An unwritten variable reads as zero rather than being absent.
  CHECK(AsInt(get_story_state(g_ctx, conversation, Declare("never_written"))) == 0,
        "an unwritten variable should read as 0");

  // Shared state outlives the conversation it was written from.
  CHECK(remove_conversation(g_ctx, conversation) == 0, "the conversation should unload");
  CHECK(AsInt(get_story_state(g_ctx, LOUDER_NAME_NONE, shared_upper)) == 9,
        "a shared variable should survive its conversation being removed");
  CHECK(remove_conversation(g_ctx, conversation) != 0, "removing it twice should fail");

  // A local variable is gone with its conversation.
  CHECK(AsInt(get_story_state(g_ctx, LOUDER_NAME_NONE, local_lower)) == 0,
        "a local variable should not outlive its conversation");
}

void TestActors() {
  std::printf("actors\n");

  LouderName conversation = Declare("actor conversation");
  std::string json = ConversationJson();
  LoadError error = load_conversation_json(g_ctx, conversation,
                                           reinterpret_cast<uint8_t*>(&json[0]), json.size(), 0);
  CHECK(error == 0, "loading the conversation returned %d", error);

  LouderName a = Lookup("A");
  LouderName b = Lookup("B");

  LouderName speak_a[1] = {a};
  LouderName speak_b[1] = {b};

  LouderActor actor_a = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  LouderActor actor_b = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_b, 1, nullptr, 0);

  CHECK(actor_a == 0, "the first actor should get id 0, got %d", actor_a);
  CHECK(actor_b == 1, "the second actor should get id 1, got %d", actor_b);

  CHECK(get_actor_position(g_ctx, conversation, actor_a) == 0, "the actor should start at its start position");
  CHECK(ReadName(get_actor_channel(g_ctx, conversation, actor_a)) == "main",
        "an actor should start on the main channel");
  CHECK(get_actor_flags(g_ctx, conversation, actor_a).bits == 0, "an actor should start with no flags");

  // A spectator is not part of the flow, so it gets no channel.
  LouderActor spectator = add_actor(g_ctx, conversation, 0, ActorFlags_SPECTATOR, nullptr, 0, nullptr, 0);
  CHECK(spectator == 2, "the third actor should get id 2, got %d", spectator);
  CHECK(get_actor_channel(g_ctx, conversation, spectator) == LOUDER_NAME_NONE,
        "a spectator should have no channel");

  // Iterate the actors, which should come back in id order.
  std::vector<LouderActor> found;
  for (LouderActor id = next_actor_speaking_as(g_ctx, conversation, a, LOUDER_ACTOR_NONE);
       id != LOUDER_ACTOR_NONE; id = next_actor_speaking_as(g_ctx, conversation, a, id)) {
    found.push_back(id);
  }
  CHECK(found.size() == 1 && found[0] == actor_a, "only the first actor speaks as A");

  // A spectator sees the hub but is not given choices to decide with.
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, spectator) == 0,
        "a spectator should see no choices, saw %d", get_hub_choice_count(g_ctx, conversation, 0, spectator));

  // Unless it is also a debug actor.
  LouderActor debug = add_actor(g_ctx, conversation, 0, ActorFlags_SPECTATOR | ActorFlags_DEBUG,
                                nullptr, 0, nullptr, 0);
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, debug) == 1,
        "a spectator that is also a debug actor should see the choice");

  CHECK(debug == 3, "the debug actor should get id 3, got %d", debug);

  // An exact match needs every bit to agree, so the debug actor is only found
  // by naming both of its flags.
  CHECK(next_actor_from_flags(g_ctx, conversation, ActorFlags_DEBUG, 1, LOUDER_ACTOR_NONE) == LOUDER_ACTOR_NONE,
        "an exact DEBUG-only match should not find a spectator that is also debug");
  CHECK(next_actor_from_flags(g_ctx, conversation,
                              ActorFlags_SPECTATOR | ActorFlags_DEBUG, 1,
                              LOUDER_ACTOR_NONE) == debug,
        "the debug spectator should be found by an exact match on both flags");

  // A non-exact match only requires the named bits to be present.
  CHECK(next_actor_from_flags(g_ctx, conversation, ActorFlags_DEBUG, 0, LOUDER_ACTOR_NONE) == debug,
        "a containing match on DEBUG should find the debug spectator");
  CHECK(next_actor_from_flags(g_ctx, conversation, ActorFlags_SPECTATOR, 1, LOUDER_ACTOR_NONE) == spectator,
        "the plain spectator should be found by an exact flag match");

  CHECK(remove_actor(g_ctx, conversation, actor_b) == 0, "removing an actor should succeed");
  CHECK(remove_actor(g_ctx, conversation, actor_b) != 0, "removing it twice should fail");
  CHECK(get_actor_position(g_ctx, conversation, actor_b) == -1, "a removed actor should not resolve");
}

void TestChoices() {
  std::printf("choices and events\n");

  LouderName conversation = Declare("choice conversation");
  std::string json = ConversationJson();
  LoadError error = load_conversation_json(g_ctx, conversation,
                                           reinterpret_cast<uint8_t*>(&json[0]), json.size(), 0);
  CHECK(error == 0, "loading the conversation returned %d", error);

  LouderName a = Lookup("A");
  LouderName speak_a[1] = {a};
  LouderActor actor = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  CHECK(actor == 0, "the actor should get id 0");

  flush_events(g_ctx);

  // Hub 0 links to hub 5, which is also spoken by A, so the choice is available.
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 1,
        "the actor should see exactly one choice, saw %d", get_hub_choice_count(g_ctx, conversation, 0, actor));
  CHECK(get_hub_choice_link(g_ctx, conversation, 0, actor, 0) == 5, "choice 0 should link to hub 5");
  CHECK(ReadName(get_hub_choice_text(g_ctx, conversation, 0, actor, 0)) == "World",
        "a choice's text comes from the hub it links to");
  CHECK(get_hub_choice_link(g_ctx, conversation, 0, actor, 5) == -1, "an out of range choice should not resolve");

  CHECK(select_choice(g_ctx, conversation, actor, 1) != 0, "an unavailable choice should be refused");
  flush_events(g_ctx);

  CHECK(select_choice(g_ctx, conversation, actor, 0) == 0, "selecting the available choice should succeed");
  CHECK(get_actor_position(g_ctx, conversation, actor) == 5, "the actor should have moved to hub 5");

  std::vector<LouderEvent> events = DrainEvents();
  int appearances = 0;
  int choice_appearances = 0;
  for (const auto& event : events) {
    if (event.data.tag == EventData::Tag::RegularMotion) {
      if (event.data.regular_motion._0.target_position == 5) {
        ++appearances;
      }
    }
    if (event.data.tag == EventData::Tag::ChoiceAppearance) {
      ++choice_appearances;
    }
  }
  CHECK(appearances == 1, "one hub appearance should be emitted, saw %d", appearances);
  CHECK(choice_appearances == 1, "one choice appearance should be emitted, saw %d", choice_appearances);

  // Taking the last choice reaches the hub carrying the callback.
  flush_events(g_ctx);
  CHECK(select_choice(g_ctx, conversation, actor, 0) == 0, "the actor should be able to continue");
  CHECK(get_actor_position(g_ctx, conversation, actor) == 9, "the actor should have reached hub 9");

  events = DrainEvents();
  CHECK(CountEventsOfKind(events, EventData::Tag::Callback) == 1,
        "the callback should fire once, saw %d", CountEventsOfKind(events, EventData::Tag::Callback));

  // Callbacks are one-shot: revisiting the hub must not fire them again.
  flush_events(g_ctx);
  force_position(g_ctx, conversation, actor, 5, LOUDER_NAME_NONE, 0);
  force_position(g_ctx, conversation, actor, 9, LOUDER_NAME_NONE, 0);
  events = DrainEvents();
  CHECK(CountEventsOfKind(events, EventData::Tag::Callback) == 0,
        "a revisited hub should not refire its callback, saw %d",
        CountEventsOfKind(events, EventData::Tag::Callback));
}

// Hubs 1 and 2 are Auto and linked to each other, so an actor entering either
// one walks the pair forever unless the engine refuses to re-enter an auto hub
// it has already resolved through. Hub 4 is the control: an Auto hub with an
// exit, which should still resolve.
const char* AutoCycleJson() {
  return R"([
    {"type":"hub","conversation_id":0,"text":"Start","flags":0},
    {"type":"hub","conversation_id":1,"text":"Loop A","flags":1},
    {"type":"hub","conversation_id":2,"text":"Loop B","flags":1},
    {"type":"hub","conversation_id":3,"text":"Out","flags":0},
    {"type":"hub","conversation_id":4,"text":"Step","flags":1},
    {"type":"connection","from_id":0,"from_port":0,"to_id":1},
    {"type":"connection","from_id":1,"from_port":0,"to_id":2},
    {"type":"connection","from_id":2,"from_port":0,"to_id":1},
    {"type":"connection","from_id":4,"from_port":0,"to_id":3}
  ])";
}

// Two actors in one channel choosing against each other. Actor 0 moving makes
// actor 1 a follower, and actor 1 moving would make actor 0 a follower again,
// so an unguarded engine recurses until the stack gives out.
void TestWarpRecursion() {
  std::printf("warp recursion\n");

  LouderName conversation = Declare("warp conversation");
  std::string json = ConversationJson();
  LoadError error = load_conversation_json(g_ctx, conversation,
                                           reinterpret_cast<uint8_t*>(&json[0]), json.size(), 0);
  CHECK(error == 0, "loading the conversation returned %d", error);

  LouderName a = Lookup("A");
  LouderName b = Lookup("B");
  LouderName speak_a[1] = {a};
  LouderName speak_b[1] = {b};

  // Both start on the main channel at the same hub, which is the setup from the
  // engine guide: they can each see where the other lands.
  LouderActor first = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  LouderActor second = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_b, 1, nullptr, 0);
  CHECK(first == 0 && second == 1, "the two actors should get ids 0 and 1, got %d and %d", first, second);
  CHECK(ReadName(get_actor_channel(g_ctx, conversation, second)) == "main",
        "both actors should start on the main channel");

  flush_events(g_ctx);

  // Hub 5 has no speaker, so it is public and appears to the second actor. The
  // move must terminate rather than trading the pair back and forth.
  CHECK(select_choice(g_ctx, conversation, first, 0) == 0, "the choice should be accepted");
  CHECK(get_actor_position(g_ctx, conversation, first) == 5, "the chooser should have moved to hub 5");
  CHECK(get_actor_position(g_ctx, conversation, second) == 5,
        "the follower should have been warped to hub 5, saw %d",
        get_actor_position(g_ctx, conversation, second));

  // The warp is reported as a warp rather than as the follower choosing, which
  // is how a host tells the two apart.
  std::vector<LouderEvent> events = DrainEvents();
  CHECK(CountEventsOfKind(events, EventData::Tag::Warp) == 1,
        "the follower should be announced with one warp event, saw %d",
        CountEventsOfKind(events, EventData::Tag::Warp));
  CHECK(CountEventsOfKind(events, EventData::Tag::RegularMotion) == 1,
        "the chooser should be announced with one regular motion, saw %d",
        CountEventsOfKind(events, EventData::Tag::RegularMotion));

  // Having settled together, neither should pull the other anywhere on the
  // next choice, and the cascade should still terminate.
  flush_events(g_ctx);
  CHECK(select_choice(g_ctx, conversation, second, 0) == 0, "the second actor should be able to choose");
  CHECK(get_actor_position(g_ctx, conversation, first) == 9, "the first actor should follow to hub 9");
  CHECK(get_actor_position(g_ctx, conversation, second) == 9, "the second actor should be at hub 9");
  events = DrainEvents();
  CHECK(CountEventsOfKind(events, EventData::Tag::Warp) == 1,
        "exactly one actor should have been warped, saw %d",
        CountEventsOfKind(events, EventData::Tag::Warp));

  // A third actor forced into the same channel starts a fresh cascade, and the
  // guide says the others follow it, so both are expected to be pulled along
  // exactly once each.
  flush_events(g_ctx);
  LouderActor third = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_b, 1, nullptr, 0);
  CHECK(force_position(g_ctx, conversation, third, 5, LOUDER_NAME_NONE, 0) == 0,
        "a third actor should be able to move into the settled flow");
  CHECK(get_actor_position(g_ctx, conversation, third) == 5, "the forced actor should be at hub 5");
  CHECK(get_actor_position(g_ctx, conversation, first) == 5,
        "the first actor should have followed to hub 5, saw %d",
        get_actor_position(g_ctx, conversation, first));
  CHECK(get_actor_position(g_ctx, conversation, second) == 5,
        "the second actor should have followed to hub 5, saw %d",
        get_actor_position(g_ctx, conversation, second));

  events = DrainEvents();
  CHECK(CountEventsOfKind(events, EventData::Tag::Warp) == 2,
        "each of the two settled actors should be warped exactly once, saw %d",
        CountEventsOfKind(events, EventData::Tag::Warp));
  CHECK(CountEventsOfKind(events, EventData::Tag::ForceMotion) == 1,
        "only the forced actor should report a forced motion, saw %d",
        CountEventsOfKind(events, EventData::Tag::ForceMotion));
}

// An auto hub that leads back to itself, directly or through a partner, has no
// exit. The engine has to give up on it rather than resolving it forever.
void TestAutoCycle() {
  std::printf("auto hub cycle\n");

  LouderName conversation = Declare("auto cycle conversation");
  std::string json = AutoCycleJson();
  LoadError error = load_conversation_json(g_ctx, conversation,
                                           reinterpret_cast<uint8_t*>(&json[0]), json.size(), 0);
  CHECK(error == 0, "loading the conversation returned %d", error);

  LouderName speak_a[1] = {Lookup("A")};
  LouderActor actor = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  CHECK(actor == 0, "the actor should get id 0");

  flush_events(g_ctx);

  // Hub 0 links to the auto cycle, so entering it walks 1 -> 2 -> 1 and has to
  // stop rather than spin.
  CHECK(force_position(g_ctx, conversation, actor, 1, LOUDER_NAME_NONE, 0) != 0,
        "entering a closed cycle of auto hubs should be refused");
  CHECK(get_actor_position(g_ctx, conversation, actor) == 0,
        "a refused move should leave the actor where it was, saw %d",
        get_actor_position(g_ctx, conversation, actor));

  // The refusal is about the cycle, not about auto hubs generally: hub 4 is
  // auto as well but leads out to hub 3, so it resolves normally.
  flush_events(g_ctx);
  CHECK(force_position(g_ctx, conversation, actor, 4, LOUDER_NAME_NONE, 0) == 0,
        "an auto hub with an exit should resolve");
  CHECK(get_actor_position(g_ctx, conversation, actor) == 3,
        "resolving through hub 4 should land on hub 3, saw %d",
        get_actor_position(g_ctx, conversation, actor));

  // An auto hub is never something an actor can choose onto, so the one choice
  // on hub 0 offers nothing, and choosing is refused rather than looping.
  flush_events(g_ctx);
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 0,
        "a choice onto an auto hub should not be offered, saw %d",
        get_hub_choice_count(g_ctx, conversation, 0, actor));
  CHECK(select_choice(g_ctx, conversation, actor, 0) != 0,
        "choosing onto an auto hub should be refused");
  CHECK(get_actor_position(g_ctx, conversation, actor) == 3,
        "a refused choice should leave the actor where it was, saw %d",
        get_actor_position(g_ctx, conversation, actor));
}

void TestAsks() {
  std::printf("asks\n");

  LouderAsk ask{};
  ask.conversation = LOUDER_NAME_NONE;
  ask.actor = LOUDER_ACTOR_NONE;
  ask.ask = Declare("get hero money");
  CHECK(push_ask(g_ctx, &ask) == LouderError::Ok, "pushing an ask should succeed");

  LouderOption<LouderAsk> fetched = get_ask(g_ctx, 0);
  CHECK(fetched.tag == LouderOption<LouderAsk>::Tag::Some, "the pushed ask should be readable");
  CHECK(ReadName(fetched.some._0.ask) == "get hero money", "the ask should carry its callback name");

  CHECK(answer_ask(g_ctx, 0, 42) == LouderError::Ok, "answering an ask should succeed");
  CHECK(answer_ask(g_ctx, 0, 43) == LouderError::Missing, "answering the same ask twice should fail");
  CHECK(answer_ask(g_ctx, 99, 0) == LouderError::Missing, "answering out of range should fail");
}

void TestSerialization() {
  std::printf("serialization\n");

  LouderName conversation = Declare("serialized conversation");
  std::string json = ConversationJson();
  load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]), json.size(), 0);

  LouderName a = Lookup("A");
  LouderName speak_a[1] = {a};
  add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);

  set_story_state(g_ctx, conversation, Declare("counter"), make_variant_int(3));
  set_story_state(g_ctx, conversation, Declare("ratio"), make_variant_float(1.5f));
  set_story_state(g_ctx, conversation, Declare("label"), make_variant_string(Declare("hello")));

  std::string state = ReadJson(read_state_json, conversation);
  CHECK(state.find("\"counter\":3") != std::string::npos, "state should carry the counter, got %s", state.c_str());
  CHECK(state.find("\"ratio\":1.5") != std::string::npos, "state should carry the float, got %s", state.c_str());
  CHECK(state.find("\"label\":\"hello\"") != std::string::npos, "state should carry the string, got %s", state.c_str());

  std::string snapshot = ReadJson(read_snapshot_json, conversation);
  CHECK(snapshot.find("\"actors\":[") != std::string::npos, "snapshot should carry actors, got %s", snapshot.c_str());
  CHECK(snapshot.find("\"position\":0") != std::string::npos, "snapshot should carry positions, got %s", snapshot.c_str());
  CHECK(snapshot.find("\"channel\":\"main\"") != std::string::npos,
        "snapshot should carry channels, got %s", snapshot.c_str());
}

void TestBuiltEvents() {
  std::printf("built events\n");

  flush_events(g_ctx);

  // Events the engine built itself and handed back through the queue, which is
  // what says the make_* constructors agree with the layout it uses.
  LouderEvent callback = BareEvent(make_event_data_callback(Declare("on_arrival")));
  CHECK(test_push_event(g_ctx, &callback) == LouderError::Ok, "pushing a built event should succeed");

  LouderEvent motion = BareEvent(make_event_data_regular_motion(Motion{0, 5, 1, false}));
  CHECK(test_push_event(g_ctx, &motion) == LouderError::Ok, "pushing a built motion should succeed");

  std::vector<LouderEvent> events = DrainEvents();
  CHECK(events.size() == 2, "both pushed events should come back, saw %zu", events.size());
  CHECK(events.size() == 2 && events[0].data.tag == EventData::Tag::Callback,
        "the first event should still read as a callback");
  CHECK(events.size() == 2 && ReadName(events[0].data.callback._0) == "on_arrival",
        "the callback should still carry its name");
  CHECK(events.size() == 2 && events[1].data.tag == EventData::Tag::RegularMotion,
        "the second event should still read as a motion");
  CHECK(events.size() == 2 && events[1].data.regular_motion._0.target_position == 5,
        "the motion should still carry its target position");
  CHECK(events.size() == 2 && events[1].data.regular_motion._0.choice_index == 1,
        "the motion should still carry its choice index");
}

void TestNullSafety() {
  std::printf("null handles\n");

  CHECK(init_louder() != nullptr, "initialising should succeed");

  CHECK(get_event(nullptr, 0).tag == LouderOption<LouderEvent>::Tag::None, "get_event should reject null");
  CHECK(get_ask(nullptr, 0).tag == LouderOption<LouderAsk>::Tag::None, "get_ask should reject null");
  CHECK(get_hub_text(nullptr, 0, 0) == LOUDER_NAME_NONE, "get_hub_text should reject null");
  CHECK(get_actor_position(nullptr, 0, 0) == -1, "get_actor_position should reject null");
  CHECK(get_story_state(nullptr, 0, 0).tag == LouderVariant::Tag::Int, "get_story_state should reject null");
  CHECK(read_name(nullptr, 0, nullptr, 0, 0) == 0, "read_name should reject null");
  flush_events(nullptr);
  set_story_state(nullptr, 0, 0, make_variant_zero());
  CHECK(deinit_louder(nullptr) == LouderError::Missing, "deinit should reject null");
}

}  // namespace

int main(int argc, char** argv) {
  g_ctx = init_louder();
  if (g_ctx == nullptr) {
    std::printf("failed to initialise the engine\n");
    return 1;
  }

  std::printf("running engine tests\n");
  TestNames();
  TestLoadConversation();
  TestConfig();
  TestStoryState();
  TestActors();
  TestChoices();
  TestWarpRecursion();
  TestAutoCycle();
  TestAsks();
  TestSerialization();
  TestBuiltEvents();
  TestNullSafety();

  deinit_louder(g_ctx);

  std::printf("\n%d checks, %d failures\n", g_checks, g_failures);
  return g_failures == 0 ? 0 : 1;
}