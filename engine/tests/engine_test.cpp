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

// Read a name back out of the engine as a std::string. The engine hands over the
// bytes it was given, so the decoding on this side is all there is to it.
std::string ReadName(LouderName name) {
  std::vector<uint8_t> buffer(4096, 0);
  size_t written = read_name(g_ctx, name, buffer.data(), buffer.size() - 1);
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

std::string ReadJson(LouderName (*fn)(const LouderContext*, LouderName, uint8_t*, size_t),
                    LouderName conversation) {
  std::vector<uint8_t> buffer(65536, 0);
  int32_t written = fn(g_ctx, conversation, buffer.data(), buffer.size() - 1);
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

// The move an actor made onto a hub, if it made one.
//
// Every kind of move carries the same motion, so the search covers all of them:
// what a test is looking for is where somebody went and who the hub was heard
// as while they were going there.
bool FindMotion(const std::vector<LouderEvent>& events, LouderActor actor, LouderPosition target,
                Motion* found) {
  for (const auto& event : events) {
    if (event.actor != actor) {
      continue;
    }

    const Motion* motion = nullptr;
    if (event.data.tag == EventData::Tag::RegularMotion) {
      motion = &event.data.regular_motion._0;
    } else if (event.data.tag == EventData::Tag::Warp) {
      motion = &event.data.warp._0;
    } else if (event.data.tag == EventData::Tag::ForceMotion) {
      motion = &event.data.force_motion._0;
    }

    if (motion != nullptr && motion->target_position == target) {
      *found = *motion;
      return true;
    }
  }

  return false;
}

// Every load error in the queue, as (position, error) pairs.
//
// The order matters only in that it is the order the loader reported them in,
// which is the order the records appear in the document.
std::vector<LoadFailure> DrainLoadErrors() {
  std::vector<LoadFailure> failures;
  for (const auto& event : DrainEvents()) {
    if (event.data.tag == EventData::Tag::LoadError) {
      failures.push_back(event.data.load_error._0);
    }
  }
  return failures;
}

// The rules the engine is running, as it reports them back.
DialogueRules CurrentRules() { return get_dialogue_rules(g_ctx); }

// Run `body` with only `rules` switched on, putting the old ones back after.
//
// The rules live on the context rather than on a conversation, so a test that
// changed them without restoring would be answering for every test after it.
template <typename Body>
void WithRules(DialogueRules rules, Body body) {
  DialogueRules previous = get_dialogue_rules(g_ctx);
  CHECK(set_dialogue_rules(g_ctx, rules) == 0, "the rules should be settable");
  body();
  set_dialogue_rules(g_ctx, previous);
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

// A private hub whose speaker is not its recipient. Hub 5 is spoken by A but
// addressed to B, so B owns it and may lead it without speaking it.
//
// Every speaker here is non-public, so nothing reaches an actor by default: an
// actor only sees or leads what it actually hears. That way a passing test
// cannot be an accident of a public speaker leaking the hub.
const char* PrivateJson() {
  return R"([
    {"type":"hub","conversation_id":0,"speaker":"A","text":"ask","tags":"start","flags":0},
    {"type":"hub","conversation_id":5,"speaker":"A","text":"aside","recipients":"B","choices":"1","flags":0},
    {"type":"hub","conversation_id":9,"speaker":"C","text":"secret","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":5},
    {"type":"connection","from_id":5,"from_port":0,"to_id":9}
  ])";
}

// B listens to A and to C, and both for the same reason: an actor may only
// decide from a hub that appears to it, and may only take a choice that leads
// somewhere that appears to it. So B hears A's speech in order to stand on hub 0
// and walk into the aside addressed to it, and hears C in order to lead out of
// that aside and into the secret. A and C hear nobody but themselves, which is
// what leaves them unable to hear anything else.
const char* PrivateConfigJson() {
  return R"([
    {"type":"speaker","name":"A","flags":0},
    {"type":"speaker","name":"B","flags":0,"listening":"A\nC"},
    {"type":"speaker","name":"C","flags":0}
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
                                           std::strlen(ConversationJson()));
  CHECK(error == 0, "loading a well formed conversation returned %d", error);

  LoadError again = load_conversation_json(g_ctx, conversation,
                                           reinterpret_cast<uint8_t*>(const_cast<char*>(ConversationJson())),
                                           std::strlen(ConversationJson()));
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
                                        std::strlen(garbage));
  CHECK(bad != 0, "malformed JSON should fail to load");
}

void TestConfig() {
  std::printf("speaker config\n");

  LoadError error = load_conversation_config(g_ctx,
                                             reinterpret_cast<uint8_t*>(const_cast<char*>(ConfigJson())),
                                             std::strlen(ConfigJson()));
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

// A hub is gated by its conditions, and a condition is allowed to combine
// values. The guide's truthiness rules are 0, false and the empty string, so a
// string that is neither empty nor "0" is true even where it stands next to a
// number. Visibility is decided by the read-only pass and reported by the
// travelling pass, so both have to agree: this checks that they do.
void TestConditionsUseTheGuidesTruthinessRules() {
  std::printf("condition truthiness\n");

  clear_config(g_ctx);
  const char* config = R"([{"type":"speaker","name":"A","flags":1}])";
  CHECK(load_conversation_config(g_ctx, reinterpret_cast<uint8_t*>(const_cast<char*>(config)),
                                 std::strlen(config)) == 0,
        "the config should load");

  const char* document = R"([
    {"type":"hub","conversation_id":0,"speaker":"A","text":"start","flags":0},
    {"type":"hub","conversation_id":1,"speaker":"A","text":"shown","expressions":"gate && 1","flags":0},
    {"type":"hub","conversation_id":2,"speaker":"A","text":"hidden","expressions":"gate && 1","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":1},
    {"type":"connection","from_id":0,"from_port":1,"to_id":2}
  ])";

  LouderName conversation = Declare("condition conversation");
  CHECK(load_conversation_json(g_ctx, conversation,
                               reinterpret_cast<uint8_t*>(const_cast<char*>(document)),
                               std::strlen(document)) == 0,
        "the conversation should load");

  LouderName speaker[1] = {Lookup("A")};
  LouderActor actor = add_actor(g_ctx, conversation, 0, ActorFlags(), speaker, 1, nullptr, 0);
  CHECK(actor == 0, "the actor should get id 0, got %d", actor);

  LouderName gate = Declare("gate");

  // An unwritten variable reads as 0, so neither hub appears.
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 0,
        "an unwritten gate should close both choices, saw %d",
        get_hub_choice_count(g_ctx, conversation, 0, actor));

  // A non-empty string is true, so both conditions hold.
  set_story_state(g_ctx, conversation, gate, make_variant_string(Declare("hello")));
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 2,
        "a non-empty string should be truthy next to a number, saw %d choices",
        get_hub_choice_count(g_ctx, conversation, 0, actor));

  // The empty string is false, which is what the guide says about it.
  set_story_state(g_ctx, conversation, gate, make_variant_string(Declare("")));
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 0,
        "an empty string should be falsy, saw %d choices",
        get_hub_choice_count(g_ctx, conversation, 0, actor));

  // And so is the string "0", which is what lets a bare string act like a number.
  set_story_state(g_ctx, conversation, gate, make_variant_string(Declare("0")));
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 0,
        "the string \"0\" should be falsy, saw %d choices",
        get_hub_choice_count(g_ctx, conversation, 0, actor));

  remove_conversation(g_ctx, conversation);
  clear_config(g_ctx);
}

void TestActors() {
  std::printf("actors\n");

  LouderName conversation = Declare("actor conversation");
  std::string json = ConversationJson();
  LoadError error = load_conversation_json(g_ctx, conversation,
                                           reinterpret_cast<uint8_t*>(&json[0]), json.size());
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
                                           reinterpret_cast<uint8_t*>(&json[0]), json.size());
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
// exit, which should still resolve. Hub 6 is the third shape: an Auto hub with
// no choices at all, which has nowhere to resolve to.
const char* AutoCycleJson() {
  return R"([
    {"type":"hub","conversation_id":0,"text":"Start","flags":0},
    {"type":"hub","conversation_id":1,"text":"Loop A","flags":1},
    {"type":"hub","conversation_id":2,"text":"Loop B","flags":1},
    {"type":"hub","conversation_id":3,"text":"Out","flags":0},
    {"type":"hub","conversation_id":4,"text":"Step","flags":1},
    {"type":"hub","conversation_id":6,"text":"Nowhere","flags":1},
    {"type":"connection","from_id":0,"from_port":0,"to_id":1},
    {"type":"connection","from_id":1,"from_port":0,"to_id":2},
    {"type":"connection","from_id":2,"from_port":0,"to_id":1},
    {"type":"connection","from_id":4,"from_port":0,"to_id":3}
  ])";
}

// Hub 0 has five choices, and no two actors are offered all of them. Hub 4 is
// Whisper's private voice, so it reaches only an actor that listens to Whisper,
// which leaves a hole in the middle of the available indices rather than a
// suffix. That is the shape a host has to be careful about: the visible list is
// numbered from zero, but the engine's indices are not.
const char* GappedChoicesJson() {
  return R"([
    {"type":"hub","conversation_id":0,"speaker":"A","text":"start","flags":0},
    {"type":"hub","conversation_id":1,"speaker":"A","text":"a","flags":0},
    {"type":"hub","conversation_id":2,"speaker":"B","text":"b","flags":0},
    {"type":"hub","conversation_id":3,"speaker":"C","text":"c","flags":0},
    {"type":"hub","conversation_id":4,"speaker":"Whisper","text":"whisper","flags":0},
    {"type":"hub","conversation_id":5,"speaker":"A","text":"e","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":1},
    {"type":"connection","from_id":0,"from_port":1,"to_id":2},
    {"type":"connection","from_id":0,"from_port":2,"to_id":3},
    {"type":"connection","from_id":0,"from_port":3,"to_id":4},
    {"type":"connection","from_id":0,"from_port":4,"to_id":5}
  ])";
}

// Only B listens to Whisper. A and C hear the public voices and nothing else.
const char* GappedChoicesConfigJson() {
  return R"([
    {"type":"speaker","name":"A","flags":1},
    {"type":"speaker","name":"B","flags":1,"listening":"Whisper"},
    {"type":"speaker","name":"C","flags":1},
    {"type":"speaker","name":"Whisper","flags":0}
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
                                           reinterpret_cast<uint8_t*>(&json[0]), json.size());
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
                                           reinterpret_cast<uint8_t*>(&json[0]), json.size());
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

  // An auto hub is never something an actor can stand on, but choosing *into* one
  // is how the auto logic gets entered. It is offered, and the move is then
  // refused only because this hub has no way out.
  flush_events(g_ctx);
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 1,
        "a choice into an auto hub should be offered, since entering one is how the actor moves, "
        "saw %d", get_hub_choice_count(g_ctx, conversation, 0, actor));
  CHECK(select_choice(g_ctx, conversation, actor, 0) != 0,
        "but choosing into a closed cycle of auto hubs should be refused");
  CHECK(get_actor_position(g_ctx, conversation, actor) == 3,
        "a refused choice should leave the actor where it was, saw %d",
        get_actor_position(g_ctx, conversation, actor));

  // Hub 6 is auto with no choices at all, so there is nothing for it to resolve
  // to. That is a dead end rather than a cycle, and it has to be refused the
  // same way: no state may be left in which an actor is standing somewhere it
  // can never leave.
  flush_events(g_ctx);
  CHECK(get_hub_choice_count(g_ctx, conversation, 6, actor) == 0,
        "the premise is an auto hub with nothing to resolve to, saw %d choices",
        get_hub_choice_count(g_ctx, conversation, 6, actor));
  CHECK(force_position(g_ctx, conversation, actor, 6, LOUDER_NAME_NONE, 0) != 0,
        "entering an auto hub with no exit at all should be refused");
  CHECK(get_actor_position(g_ctx, conversation, actor) == 3,
        "and the refusal should leave the actor where it was, saw %d",
        get_actor_position(g_ctx, conversation, actor));
}

// A host discovers which choices to show by probing indices, because the engine
// has no call for a hub's raw choice count and refuses an out-of-range index and
// an unavailable one identically. The contract that makes that possible is that
// get_hub_choice_count agrees with how many indices probing actually finds, so it
// is checked at every hub for every kind of actor rather than on one lucky path.
void TestChoiceCountAgreesWithProbing() {
  std::printf("choice count agrees with probing\n");

  LouderName conversation = Declare("probing conversation");
  std::string json = GappedChoicesJson();
  CHECK(load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]),
                               json.size()) == 0,
        "loading the gapped conversation returned %d", 0);

  LouderName a = Lookup("A");
  LouderActor speaking = add_actor(g_ctx, conversation, 0, ActorFlags(), &a, 1, nullptr, 0);
  LouderActor debug = add_actor(g_ctx, conversation, 0, ActorFlags_SPECTATOR | ActorFlags_DEBUG,
                                nullptr, 0, nullptr, 0);

  int hubs_offering_choices = 0;
  for (LouderPosition position = 0; position < 16; ++position) {
    for (LouderActor actor : {speaking, debug}) {
      const int32_t expected = get_hub_choice_count(g_ctx, conversation, position, actor);
      int32_t found = 0;
      for (int32_t index = 0; index < 64; ++index) {
        if (get_hub_choice_link(g_ctx, conversation, position, actor, index) != -1) {
          ++found;
        }
      }
      CHECK(found == expected,
            "at hub %d for actor %d the engine counts %d choices but probing finds %d", position,
            actor, expected, found);
      if (expected > 0) {
        ++hubs_offering_choices;
      }
    }
  }
  CHECK(hubs_offering_choices > 0, "the fixture should have hubs that offer choices at all");
}

// The engine's choice indices are positions in the hub's full choice list, and
// the ones available to an actor need not be contiguous. So a host cannot use
// the position on screen as the index it passes back: the second thing A is shown
// below is engine index 4.
void TestAvailableChoiceIndicesCanHaveGaps() {
  std::printf("available choice indices can have gaps\n");

  clear_config(g_ctx);
  const char* config = GappedChoicesConfigJson();
  CHECK(load_conversation_config(g_ctx, reinterpret_cast<uint8_t*>(const_cast<char*>(config)),
                                 std::strlen(config)) == 0,
        "the gapped config should load");

  LouderName conversation = Declare("gapped conversation");
  std::string json = GappedChoicesJson();
  CHECK(load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]),
                               json.size()) == 0,
        "loading the gapped conversation returned %d", 0);

  LouderName speak_a[1] = {Lookup("A")};
  LouderName speak_b[1] = {Lookup("B")};
  LouderName speak_c[1] = {Lookup("C")};

  // All three actors are added before anything is asked, because who is in the
  // conversation is part of the answer: once an actor is playing a voice, nobody
  // else may lead it.
  LouderActor actor_a = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  LouderActor actor_b = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_b, 1, nullptr, 0);
  LouderActor actor_c = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_c, 1, nullptr, 0);
  CHECK(actor_a == 0 && actor_b == 1 && actor_c == 2, "the actors should be 0, 1 and 2, got %d, %d, %d",
        actor_a, actor_b, actor_c);

  // A speaks as A, so it leads both of A's branches, at indices 0 and 4. B and C
  // hold their own voices and Whisper is not A's to hear, so the three indices
  // in between are closed to it. What A is shown is two choices, the second of
  // which is engine index 4.
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_a) == 2,
        "A should be offered two of the five, saw %d", get_hub_choice_count(g_ctx, conversation, 0,
                                                                              actor_a));
  CHECK(get_hub_choice_link(g_ctx, conversation, 0, actor_a, 0) == 1,
        "A's first choice should lead to hub 1");
  CHECK(get_hub_choice_link(g_ctx, conversation, 0, actor_a, 3) == -1,
        "index 3 leads to Whisper's voice, which A does not hear");
  CHECK(get_hub_choice_link(g_ctx, conversation, 0, actor_a, 4) == 5,
        "and A's second choice is engine index 4, not 1, and leads to hub 5");

  // B leads its own voice and Whisper's, at indices 1 and 3, and is refused A's
  // and C's even though both speakers are public.
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_b) == 2,
        "B should be offered two of the five, saw %d", get_hub_choice_count(g_ctx, conversation, 0,
                                                                              actor_b));
  CHECK(get_hub_choice_link(g_ctx, conversation, 0, actor_b, 1) == 2,
        "B's first choice should lead to hub 2");
  CHECK(get_hub_choice_link(g_ctx, conversation, 0, actor_b, 3) == 4,
        "B listens to Whisper, so its second choice is engine index 3 and leads to hub 4");

  // C holds only its own voice, and is left with the single choice at index 2.
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_c) == 1,
        "C should be offered just its own, saw %d", get_hub_choice_count(g_ctx, conversation, 0,
                                                                         actor_c));
  CHECK(get_hub_choice_link(g_ctx, conversation, 0, actor_c, 2) == 3,
        "C's only choice should be engine index 2, leading to hub 3");

  remove_conversation(g_ctx, conversation);
  clear_config(g_ctx);
}

void TestPrivateHubIsOwnedByItsRecipients() {
  std::printf("private hub ownership\n");

  // This fixture only means anything under its own config. Another one would
  // leave speaker C undefined, and an undefined speaker is public, so every hub
  // below would reach every actor and the blocks would pass for reasons that
  // have nothing to do with ownership. Loading it here, rather than inheriting
  // whatever an earlier test left, is what makes the rest of this meaningful.
  clear_config(g_ctx);
  CHECK(load_conversation_config(
            g_ctx,
            reinterpret_cast<uint8_t*>(const_cast<char*>(PrivateConfigJson())),
            std::strlen(PrivateConfigJson())) == 0,
        "the private hub config should load");

  LouderName conversation = Declare("private conversation");
  std::string json = PrivateJson();
  CHECK(load_conversation_json(g_ctx, conversation,
                               reinterpret_cast<uint8_t*>(&json[0]), json.size()) == 0,
        "loading the private conversation returned %d", 0);

  LouderName a = Lookup("A");
  LouderName b = Lookup("B");
  LouderName c = Lookup("C");
  CHECK(c != LOUDER_NAME_NONE, "hub 9's speaker should be interned");

  // The flags alone cannot show that the config above is the one in effect: an
  // undeclared speaker reports no flags, and no flags also reads as public. So
  // the listening list is checked too, since that is a declaration nothing else
  // in the suite would have left behind.
  CHECK(get_speaker_flags(g_ctx, a).bits != SpeakerFlags_PUBLIC.bits,
        "A should not be public here, or A would hear its own speech through the back door");
  CHECK(get_speaker_flags(g_ctx, b).bits != SpeakerFlags_PUBLIC.bits,
        "B should not be public here, or this would not be testing a private hub");
  CHECK(get_speaker_flags(g_ctx, c).bits != SpeakerFlags_PUBLIC.bits,
        "C should not be public here, or hub 9 would reach actors that never heard it");
  CHECK(get_next_speaker_listen_to(g_ctx, b, LOUDER_NAME_NONE) == a &&
            get_next_speaker_listen_to(g_ctx, b, a) == c,
        "B should listen to A then C; if this fails then PrivateConfigJson is not the config "
        "in effect");

  // Hub 5 is spoken by A and addressed to B. With a B actor alone, hub 5 must be
  // reachable and leadable: B speaks as B, which is how B owns it.
  {
    LouderName speak_b[1] = {b};
    LouderActor actor = add_actor(g_ctx, conversation, 5, ActorFlags(), speak_b, 1, nullptr, 0);
    CHECK(actor == 0, "the first actor should get id 0, got %d", actor);
    CHECK(get_hub_choice_count(g_ctx, conversation, 5, actor) == 1,
          "the recipient should own the private hub and be offered its choice, saw %d",
          get_hub_choice_count(g_ctx, conversation, 5, actor));
    CHECK(get_hub_choice_link(g_ctx, conversation, 5, actor, 0) == 9,
          "and that choice should lead to 9");
  }

  // An A actor does not own it: A speaks the hub but is not a recipient, and
  // ownership is not the speaker. A must be offered nothing at all, and must not
  // be able to stand there either.
  {
    LouderName speak_a[1] = {a};
    LouderActor actor = add_actor(g_ctx, conversation, 5, ActorFlags(), speak_a, 1, nullptr, 0);
    CHECK(get_hub_choice_count(g_ctx, conversation, 5, actor) == 0,
          "the speaker of a private hub is not its owner and should get no choices, saw %d",
          get_hub_choice_count(g_ctx, conversation, 5, actor));

    // The choice into it from hub 0 is equally closed to A.
    CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 0,
          "A should not be offered the choice into its own speech addressed to B, saw %d",
          get_hub_choice_count(g_ctx, conversation, 0, actor));
  }

  // With both actors present, the recipient still owns it and the speaker still
  // does not. This is the case the demo conversation runs into.
  {
    LouderName speak_a[1] = {a};
    LouderName speak_b[1] = {b};
    LouderActor actor_a = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
    LouderActor actor_b = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_b, 1, nullptr, 0);

    CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_a) == 0,
          "with a B actor in the conversation, A should have no way into the private hub");
    CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_b) == 1,
          "with an A actor in the conversation, B should still own the private hub, saw %d",
          get_hub_choice_count(g_ctx, conversation, 0, actor_b));

    CHECK(select_choice(g_ctx, conversation, actor_b, 0) == 0,
          "the owner should be able to walk onto the private hub");
    CHECK(get_actor_position(g_ctx, conversation, actor_b) == 5,
          "and should land on it, got %d", get_actor_position(g_ctx, conversation, actor_b));
    CHECK(get_actor_position(g_ctx, conversation, actor_a) != 5,
          "the non-owner should not be dragged onto a hub it does not own, got %d",
          get_actor_position(g_ctx, conversation, actor_a));
  }

  // And a private hub has no owner at all among the actors, so nothing can be
  // taken out of it.
  {
    LouderName speak_c[1] = {c};
    LouderActor actor_c = add_actor(g_ctx, conversation, 5, ActorFlags(), speak_c, 1, nullptr, 0);
    CHECK(get_hub_choice_count(g_ctx, conversation, 5, actor_c) == 0,
          "an actor who neither speaks as nor listens to the recipient owns nothing, saw %d",
          get_hub_choice_count(g_ctx, conversation, 5, actor_c));
  }

  remove_conversation(g_ctx, conversation);
  clear_config(g_ctx);
}

// The worked example from the engine guide's "Example Flows" section, verbatim:
//
//   Hubs 0, 1, 2, 3, 4.
//   Speakers: 2: "Speaker A", 3: "Speaker B"
//   Choices:  0: "A Choice to Make" -> 1
//             1: "(invisible choice 1)" -> 2, "(invisible choice 2)" -> 3
//             2: "Finish Conversation" -> 4
//   Flags:    1: Auto
//   Channels: 2: "Channel A"
//   Tags:     0: "Start"
//   Callbacks: 4: "exit"
//
// Two actors, one speaking as A and one as B, both starting on the start hub in
// the main channel.
const char* ExampleFlowJson() {
  return R"([
    {"type":"hub","conversation_id":0,"speaker":"A","text":"Ask something","tags":"Start","flags":0},
    {"type":"hub","conversation_id":1,"text":"","choices":"2","flags":1},
    {"type":"hub","conversation_id":2,"speaker":"A","text":"Finish Conversation","channel":"Channel A","choices":"1","flags":0},
    {"type":"hub","conversation_id":3,"speaker":"B","text":"B's answer","flags":0},
    {"type":"hub","conversation_id":4,"text":"The end","callbacks":"exit","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":1},
    {"type":"connection","from_id":1,"from_port":0,"to_id":2},
    {"type":"connection","from_id":1,"from_port":1,"to_id":3},
    {"type":"connection","from_id":2,"from_port":0,"to_id":4}
  ])";
}

const char* ExampleFlowConfigJson() {
  return R"([
    {"type":"speaker","name":"A","flags":1},
    {"type":"speaker","name":"B","flags":1},
    {"type":"speaker","name":"Channel A","flags":0}
  ])";
}

// The guide's worked example, checked move by move. The point it pins down is
// the divergence: both actors choose the same thing from hub 0, both enter auto
// hub 1, and auto logic then resolves each of them *separately*, so actor 0 lands
// on hub 2 and actor 1 on hub 3. A follower that replayed the leader's choice
// instead would have copied hub 2, and the conversation would have collapsed
// into one actor talking to itself.
void TestExampleFlow() {
  std::printf("example flow from the guide\n");

  clear_config(g_ctx);
  CHECK(load_conversation_config(
            g_ctx,
            reinterpret_cast<uint8_t*>(const_cast<char*>(ExampleFlowConfigJson())),
            std::strlen(ExampleFlowConfigJson())) == 0,
        "the example flow config should load");

  LouderName conversation = Declare("example flow conversation");
  std::string json = ExampleFlowJson();
  CHECK(load_conversation_json(g_ctx, conversation,
                               reinterpret_cast<uint8_t*>(&json[0]), json.size()) == 0,
        "loading the example flow conversation returned %d", 0);

  LouderName a = Lookup("A");
  LouderName b = Lookup("B");
  LouderName speak_a[1] = {a};
  LouderName speak_b[1] = {b};

  // The guide has actor 0 speaking as A and actor 1 speaking as B, both on the
  // start hub.
  LouderActor actor_a = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  LouderActor actor_b = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_b, 1, nullptr, 0);
  CHECK(actor_a == 0 && actor_b == 1, "the actors should be 0 and 1, got %d and %d", actor_a, actor_b);

  // A debug spectator, as the guide describes, to watch the auto hub arrive.
  LouderActor debug = add_actor(g_ctx, conversation, 0, ActorFlags_SPECTATOR | ActorFlags_DEBUG,
                                nullptr, 0, nullptr, 0);

  flush_events(g_ctx);

  // Both actors see hub 0's single choice, and the auto hub it leads to is
  // offered to them even though auto hubs never appear.
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_a) == 1,
        "actor 0 should be offered the choice out of hub 0, saw %d",
        get_hub_choice_count(g_ctx, conversation, 0, actor_a));
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_b) == 1,
        "actor 1 should be offered the same choice, saw %d",
        get_hub_choice_count(g_ctx, conversation, 0, actor_b));

  // Client 0 sends the choice at index 0.
  CHECK(select_choice(g_ctx, conversation, actor_a, 0) == 0,
        "actor 0's choice should be accepted");

  // Auto logic for actor 0: "(invisible choice 1)" links to hub 2, which is
  // spoken by A, and actor 0 speaks as A, so that branch is its to take.
  CHECK(get_actor_position(g_ctx, conversation, actor_a) == 2,
        "actor 0 should resolve through the auto hub onto hub 2, saw %d",
        get_actor_position(g_ctx, conversation, actor_a));
  CHECK(ReadName(get_actor_channel(g_ctx, conversation, actor_a)) == "Channel A",
        "hub 2 should have put actor 0 in Channel A, saw %s",
        ReadName(get_actor_channel(g_ctx, conversation, actor_a)).c_str());

  // The warping that follows. Actor 1 was in the main channel, the same channel
  // actor 0 left, and actor 0 passed through the auto hub, so actor 1 is warped
  // *through hub 1* rather than replaying actor 0's choice.
  //
  // Auto logic for actor 1: the first branch links to hub 2, spoken by A. Actor 1
  // does not speak as A, and allowing it would leave two choices available, so it
  // is skipped. The second branch links to hub 3, spoken by B, which actor 1
  // does speak as, so that one is taken.
  CHECK(get_actor_position(g_ctx, conversation, actor_b) == 3,
        "actor 1 should diverge onto hub 3, saw %d", get_actor_position(g_ctx, conversation, actor_b));
  CHECK(ReadName(get_actor_channel(g_ctx, conversation, actor_b)) == "main",
        "hub 3 should have left actor 1 on the main channel, saw %s",
        ReadName(get_actor_channel(g_ctx, conversation, actor_b)).c_str());

  // The two actors are now in different channels and in different places, which
  // is the summary the guide gives: actor 0 in position 2 with one choice in a
  // newly created channel, actor 1 in position 3 with no choices.
  CHECK(get_hub_choice_count(g_ctx, conversation, 2, actor_a) == 1,
        "actor 0 should have one choice left, saw %d",
        get_hub_choice_count(g_ctx, conversation, 2, actor_a));
  CHECK(get_hub_choice_count(g_ctx, conversation, 3, actor_b) == 0,
        "actor 1 should have no choices, saw %d",
        get_hub_choice_count(g_ctx, conversation, 3, actor_b));

  // The divergence is announced as a warp, so a host can tell it apart from a
// second player choosing.
  std::vector<LouderEvent> events = DrainEvents();
  CHECK(CountEventsOfKind(events, EventData::Tag::Warp) == 1,
        "actor 1's divergence should be one warp event, saw %d",
        CountEventsOfKind(events, EventData::Tag::Warp));

  // The auto hub itself is announced to nobody but the debug actor, which is the
  // one thing the guide says it appears to. It is a regular motion like any
  // other appearance, so it is counted per actor rather than in total.
  bool debug_saw_the_auto_hub = false;
  bool player_saw_the_auto_hub = false;
  int actor_a_motions = 0;
  for (const auto& event : events) {
    if (event.data.tag == EventData::Tag::Warp || event.data.tag == EventData::Tag::ForceMotion) {
      continue;
    }
    if (event.data.tag != EventData::Tag::RegularMotion) continue;

    const Motion& motion = event.data.regular_motion._0;
    if (event.actor == actor_a) ++actor_a_motions;
    if (motion.target_position != 1) continue;
    if (event.actor == debug) debug_saw_the_auto_hub = true;
    else player_saw_the_auto_hub = true;
  }
  CHECK(actor_a_motions == 1, "actor 0 should have announced one move of its own, saw %d",
        actor_a_motions);
  CHECK(debug_saw_the_auto_hub, "the auto hub should be announced to the debug actor");
  CHECK(!player_saw_the_auto_hub, "the auto hub should not be announced to the players");

  // The guide's last step: actor 0 takes its remaining choice to hub 4, which
  // fires the "exit" callback. Hub 4 has no speaker, so it is public and appears
  // to actor 1 too, but actor 1 is in another channel, so it is not warped.
  flush_events(g_ctx);
  CHECK(select_choice(g_ctx, conversation, actor_a, 0) == 0,
        "actor 0 should be able to finish the conversation");
  CHECK(get_actor_position(g_ctx, conversation, actor_a) == 4,
        "actor 0 should be at hub 4, saw %d", get_actor_position(g_ctx, conversation, actor_a));
  CHECK(get_actor_position(g_ctx, conversation, actor_b) == 3,
        "actor 1 should have stayed in its own channel at hub 3, saw %d",
        get_actor_position(g_ctx, conversation, actor_b));

  events = DrainEvents();
  CHECK(CountEventsOfKind(events, EventData::Tag::Callback) == 1,
        "the exit callback should fire once, saw %d",
        CountEventsOfKind(events, EventData::Tag::Callback));
  bool saw_exit = false;
  for (const auto& event : events) {
    if (event.data.tag == EventData::Tag::Callback && ReadName(event.data.callback._0) == "exit") {
      saw_exit = true;
    }
  }
  CHECK(saw_exit, "and the callback that fired should be the exit callback");

  remove_conversation(g_ctx, conversation);
  clear_config(g_ctx);
}

// The shape the demo conversation's auto hub 27 has, which is the case that
// needs both halves of the auto rules at once:
//
//   0: -> 27 (auto) -> [0] 29 (private voice)  [1] 28 (public, spoken by B)
//   2: -> 26
//
// B listens to a private voice, which is only B's. When B chooses into the auto
// hub, auto logic takes the first branch available to B, which is 29 because B
// hears it. B cannot see the second branch as unavailable-for-B (B speaks 28),
// so this pins that a multi-choice auto hub still resolves for the actor who
// owns the branch it leads to.
//
// A second actor who does not hear the private voice must not be routed to 29.
// When B leaves the auto hub, that actor is warped through 27 and emulates it as
// itself. It has no claim on branch 0 (29 does not appear to it), and branch 1
// (28, public, spoken by B) is B's, so emulation falls back to the first branch
// that appears: 28. That is the whole point of the split: B sees their own
// private voice, the other actor sees only the public line.
const char* AutoSplitJson() {
  return R"([
    {"type":"hub","conversation_id":0,"text":"start","tags":"Start","flags":0},
    {"type":"hub","conversation_id":26,"text":"lie","choices":"1","flags":0},
    {"type":"hub","conversation_id":27,"text":"","choices":"2","flags":1},
    {"type":"hub","conversation_id":28,"speaker":"B","text":"the public line","flags":0},
    {"type":"hub","conversation_id":29,"speaker":"Whisper","text":"the private line","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":26},
    {"type":"connection","from_id":26,"from_port":0,"to_id":27},
    {"type":"connection","from_id":27,"from_port":0,"to_id":29},
    {"type":"connection","from_id":27,"from_port":1,"to_id":28}
  ])";
}

// B is public and listens to Whisper; the second actor speaks as C, which is
// public and hears neither. C is standing on the same hub as B so both share
// the main channel and B drags C along when it moves.
const char* AutoSplitConfigJson() {
  return R"([
    {"type":"speaker","name":"B","flags":1,"listening":"Whisper"},
    {"type":"speaker","name":"C","flags":1},
    {"type":"speaker","name":"Whisper","flags":0}
  ])";
}

void TestAutoSplitOnPrivateVoice() {
  std::printf("auto hub split on a private voice\n");

  clear_config(g_ctx);
  CHECK(load_conversation_config(
            g_ctx,
            reinterpret_cast<uint8_t*>(const_cast<char*>(AutoSplitConfigJson())),
            std::strlen(AutoSplitConfigJson())) == 0,
        "the auto split config should load");

  LouderName conversation = Declare("auto split conversation");
  std::string json = AutoSplitJson();
  CHECK(load_conversation_json(g_ctx, conversation,
                               reinterpret_cast<uint8_t*>(&json[0]), json.size()) == 0,
        "loading the auto split conversation returned %d", 0);

  LouderName b = Lookup("B");
  LouderName c = Lookup("C");
  LouderName speak_b[1] = {b};
  LouderName speak_c[1] = {c};

  LouderActor actor_b = add_actor(g_ctx, conversation, 26, ActorFlags(), speak_b, 1, nullptr, 0);
  LouderActor actor_c = add_actor(g_ctx, conversation, 26, ActorFlags(), speak_c, 1, nullptr, 0);
  CHECK(actor_b == 0 && actor_c == 1, "the actors should be 0 and 1, got %d and %d", actor_b, actor_c);

  flush_events(g_ctx);

  // Both the choice into the auto hub and the private-voice branch are offered
  // to B, in that order. The private branch is available because B listens to
  // Whisper even though B does not speak it; without that, the auto hub would
  // have no branch B may take other than the public one and would send B to 28.
  CHECK(get_hub_choice_count(g_ctx, conversation, 26, actor_b) == 1,
        "B should be offered the single choice into the auto hub, saw %d",
        get_hub_choice_count(g_ctx, conversation, 26, actor_b));
  CHECK(get_hub_choice_count(g_ctx, conversation, 27, actor_b) == 2,
        "B should be offered both branches out of the auto hub, the private voice and the public "
        "line, saw %d",
        get_hub_choice_count(g_ctx, conversation, 27, actor_b));
  CHECK(get_hub_choice_link(g_ctx, conversation, 27, actor_b, 0) == 29,
        "B's first branch should be the private voice");
  CHECK(get_hub_choice_link(g_ctx, conversation, 27, actor_b, 1) == 28,
        "B's second branch should be the public line");

  // The other actor hears neither the private voice nor a claim on the public
  // line, so neither branch is available to it and auto logic falls back to the
  // public line for it. Emulation, not the leader's choice, is what gets it
  // there.
  CHECK(get_hub_choice_count(g_ctx, conversation, 27, actor_c) == 0,
        "the other actor has no claim on either branch of the auto hub, saw %d",
        get_hub_choice_count(g_ctx, conversation, 27, actor_c));

  // B chooses into the auto hub. B should resolve through it onto the private
  // voice, 29, because that is the first branch available to B.
  CHECK(select_choice(g_ctx, conversation, actor_b, 0) == 0,
        "B's choice into the auto hub should be accepted");
  CHECK(get_actor_position(g_ctx, conversation, actor_b) == 29,
        "B should resolve through the auto hub onto the private voice it hears, saw %d",
        get_actor_position(g_ctx, conversation, actor_b));

  // C is warped through the same auto hub and emulates it as itself. C's first
  // available branch is none, so C takes the first branch that appears to it:
  // the public line, 28. This is the divergence the demo's note describes.
  CHECK(get_actor_position(g_ctx, conversation, actor_c) == 28,
        "the other actor should diverge onto the public line, saw %d",
        get_actor_position(g_ctx, conversation, actor_c));

  // The divergence is a warp, not a second choice, so a host can tell it apart
  // from B leading.
  std::vector<LouderEvent> events = DrainEvents();
  CHECK(CountEventsOfKind(events, EventData::Tag::Warp) == 1,
        "the other actor's divergence should be exactly one warp, saw %d",
        CountEventsOfKind(events, EventData::Tag::Warp));

  remove_conversation(g_ctx, conversation);
  clear_config(g_ctx);
}

// The guide's Configuration section says the spoke rule and the singleplayer
// rule can be turned on and off through the engine. Both are on in a fresh
// context, and turning either off has to change what an actor is offered rather
// than only what it can be told about.
//
// The spoke rule is what lets a lone choice out of a hub go ahead when the voice
// behind it belongs to somebody the actor neither speaks nor hears. Whisper is
// that voice here: hub 5 is spoken by it, nothing is public, and the actor
// speaks only A. So the whole choice comes down to whether the rule is on.
// The spoke rule is the one choice rule the guide does not qualify with "no
// other actor has that speaker in its speak as list". Cases 2 and 3 are about a
// voice nobody is playing; case 4 is about a hub with nowhere else to go. So a
// fixture for it needs a voice that is held elsewhere and reachable anyway:
//
//   0: A (public)  ->  5: B (public)  ->  9: A
//
// B is public, so hub 5 appears to the A actor, but another actor speaks B, so
// the second and third cases refuse it. Hub 0 has one choice, so the rule lets
// it through anyway. Turn the rule off and the A actor is left standing on hub 0
// with nothing, which is the dead end the rule exists to prevent.
const char* SpokeRuleJson() {
  return R"([
    {"type":"hub","conversation_id":0,"speaker":"A","text":"ask","flags":0},
    {"type":"hub","conversation_id":5,"speaker":"B","text":"B's line","flags":0},
    {"type":"hub","conversation_id":9,"speaker":"A","text":"after","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":5},
    {"type":"connection","from_id":5,"from_port":0,"to_id":9}
  ])";
}

// Both voices are public, so "reachable" is not what is being tested here. What
// separates case 4 from cases 2 and 3 is the other actor holding B. Whisper is
// declared and not public, since an undefined speaker is public by default and
// would make the last block below meaningless.
const char* SpokeRuleConfigJson() {
  return R"([
    {"type":"speaker","name":"A","flags":1},
    {"type":"speaker","name":"B","flags":1},
    {"type":"speaker","name":"Whisper","flags":0}
  ])";
}

void TestSpokeRuleIsConfigurable() {
  std::printf("spoke rule configuration\n");

  clear_config(g_ctx);
  CHECK(load_conversation_config(
            g_ctx,
            reinterpret_cast<uint8_t*>(const_cast<char*>(SpokeRuleConfigJson())),
            std::strlen(SpokeRuleConfigJson())) == 0,
        "the spoke rule config should load");

  LouderName conversation = Declare("spoke rule conversation");
  std::string json = SpokeRuleJson();
  CHECK(load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]),
                               json.size()) == 0,
        "loading the spoke rule conversation returned %d", 0);

  LouderName speak_a[1] = {Lookup("A")};
  LouderName speak_b[1] = {Lookup("B")};

  // Both actors are present throughout. Who is in the conversation is part of
  // the answer: without a B actor there is no held voice, and the second and
  // third cases would settle this without the rule at all.
  LouderActor actor = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  add_actor(g_ctx, conversation, 0, ActorFlags(), speak_b, 1, nullptr, 0);
  CHECK(actor == 0, "the first actor should get id 0, got %d", actor);

  flush_events(g_ctx);

  // The premise, stated rather than assumed: hub 5 does appear to this actor,
  // and it is the other actor who is playing B.
  CHECK(get_hub_text(g_ctx, conversation, 5) != LOUDER_NAME_NONE,
        "hub 5 should have loaded, or the premise is wrong");
  CHECK(get_speaker_flags(g_ctx, Lookup("B")).bits == SpeakerFlags_PUBLIC.bits,
        "B should be public here, or this would not be testing case 4");
  CHECK(get_speaker_flags(g_ctx, Lookup("A")).bits == SpeakerFlags_PUBLIC.bits,
        "A should be public here, or the A actor could not see hub 5 at all");

  // With the rule on, the lone choice out of hub 0 goes ahead even though B's
  // voice is somebody else's.
  WithRules(DialogueRules_SPOKE | DialogueRules_SINGLEPLAYER, [&] {
    CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 1,
          "the spoke rule should let the lone choice through, saw %d",
          get_hub_choice_count(g_ctx, conversation, 0, actor));
    CHECK(select_choice(g_ctx, conversation, actor, 0) == 0,
          "and the actor should be able to take it");
    CHECK(get_actor_position(g_ctx, conversation, actor) == 5,
          "landing on the other actor's line, saw %d",
          get_actor_position(g_ctx, conversation, actor));
  });

  // Back to hub 0 for the half that says the rule is what opened it.
  force_position(g_ctx, conversation, actor, 0, LOUDER_NAME_NONE, 0);
  flush_events(g_ctx);
  WithRules(DialogueRules_SINGLEPLAYER, [&] {
    CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 0,
          "without the spoke rule a held voice closes the choice, saw %d",
          get_hub_choice_count(g_ctx, conversation, 0, actor));
    CHECK(select_choice(g_ctx, conversation, actor, 0) != 0,
          "and selecting it should be refused");
    CHECK(get_actor_position(g_ctx, conversation, actor) == 0,
          "a refused choice should leave the actor where it was, saw %d",
          get_actor_position(g_ctx, conversation, actor));
  });

  // Turning it back on restores the offer, which is what says the switch is
  // read per choice rather than latched at load time.
  WithRules(DialogueRules_SPOKE | DialogueRules_SINGLEPLAYER, [&] {
    CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 1,
          "turning the rule back on should restore the choice, saw %d",
          get_hub_choice_count(g_ctx, conversation, 0, actor));
  });

  // And off again is the same answer as it was the first time, which is what
  // says the switch is a switch and not a one-shot.
  WithRules(DialogueRules_SINGLEPLAYER, [&] {
    CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor) == 0,
          "and turning it off again should close it again, saw %d",
          get_hub_choice_count(g_ctx, conversation, 0, actor));
  });

  // The rule needs the voice to be reachable. Whisper is neither public nor
  // heard by this actor, so hub 5 below is somewhere the actor cannot see, and
  // no rule may offer a choice into it.
  {
    LouderName private_hub = Declare("private hub conversation");
    const char* document = R"([
      {"type":"hub","conversation_id":0,"speaker":"A","text":"ask","flags":0},
      {"type":"hub","conversation_id":5,"speaker":"Whisper","text":"whisper","flags":0},
      {"type":"connection","from_id":0,"from_port":0,"to_id":5}
    ])";
    CHECK(load_conversation_json(g_ctx, private_hub,
                                 reinterpret_cast<uint8_t*>(const_cast<char*>(document)),
                                 std::strlen(document)) == 0,
          "the unreachable conversation should load");

    LouderActor alone = add_actor(g_ctx, private_hub, 0, ActorFlags(), speak_a, 1, nullptr, 0);
    WithRules(DialogueRules_SPOKE | DialogueRules_SINGLEPLAYER, [&] {
      CHECK(get_hub_choice_count(g_ctx, private_hub, 0, alone) == 0,
            "the spoke rule must not offer a choice into a hub the actor cannot see, saw %d",
            get_hub_choice_count(g_ctx, private_hub, 0, alone));
    });
    remove_conversation(g_ctx, private_hub);
  }

  // A null context is running no dialogue, so it has no rules to report and
  // refuses to be given any.
  CHECK(get_dialogue_rules(nullptr).bits == 0, "a null context should report no rules");
  CHECK(set_dialogue_rules(nullptr, DialogueRules_SPOKE) != 0,
        "a null context should refuse rules");

  // Bits the engine does not know about are dropped rather than refused, so a
  // host carrying a configuration from a newer build still gets what it can use.
  WithRules(DialogueRules{/* .bits = */ (int32_t)0x7FFFFFFF}, [&] {
    CHECK(CurrentRules().bits == (DialogueRules_SPOKE | DialogueRules_SINGLEPLAYER).bits,
          "unknown bits should be dropped, got %d", CurrentRules().bits);
  });

  remove_conversation(g_ctx, conversation);
  clear_config(g_ctx);
}

// The singleplayer rule. Hub 5 is reached from hub 0, which branched, has one
// way out of it, and is spoken by P, who is the only actor that can hear it. So
// it resolves on its own, and the actor lands past it at hub 9 instead of
// standing on it.
//
// Hub 6 is the control that keeps the rule honest: one way out, reached from the
// same branching hub, but public, so the other actor can see it and the rule
// does not apply. Without that check the rule would be indistinguishable from
// "any hub with one choice resolves".
const char* SingleplayerJson() {
  return R"([
    {"type":"hub","conversation_id":0,"speaker":"A","text":"ask","flags":0},
    {"type":"hub","conversation_id":5,"speaker":"P","text":"aside","flags":0},
    {"type":"hub","conversation_id":6,"speaker":"A","text":"shared line","flags":0},
    {"type":"hub","conversation_id":9,"speaker":"P","text":"past the aside","flags":0},
    {"type":"hub","conversation_id":10,"speaker":"A","text":"past the shared line","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":5},
    {"type":"connection","from_id":0,"from_port":1,"to_id":6},
    {"type":"connection","from_id":5,"from_port":0,"to_id":9},
    {"type":"connection","from_id":6,"from_port":0,"to_id":10}
  ])";
}

// P is private and only P hears it. A is public, so hub 6 reaches both actors.
const char* SingleplayerConfigJson() {
  return R"([
    {"type":"speaker","name":"A","flags":1},
    {"type":"speaker","name":"P","flags":0}
  ])";
}

void TestSingleplayerRule() {
  std::printf("singleplayer rule\n");

  clear_config(g_ctx);
  CHECK(load_conversation_config(
            g_ctx,
            reinterpret_cast<uint8_t*>(const_cast<char*>(SingleplayerConfigJson())),
            std::strlen(SingleplayerConfigJson())) == 0,
        "the singleplayer config should load");

  LouderName conversation = Declare("singleplayer conversation");
  std::string json = SingleplayerJson();
  CHECK(load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]),
                               json.size()) == 0,
        "loading the singleplayer conversation returned %d", 0);

  LouderName speak_a[1] = {Lookup("A")};
  LouderName speak_p[1] = {Lookup("P")};

  // Both actors start at hub 0 so they share the main channel, which is what
  // lets one of them drag the other along.
  LouderActor actor_p = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_p, 1, nullptr, 0);
  LouderActor actor_a = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  CHECK(actor_p == 0 && actor_a == 1, "the actors should be 0 and 1, got %d and %d", actor_p,
        actor_a);

  flush_events(g_ctx);

  // P is offered the branch into its own private hub, and not the public one,
  // because A speaks A and the public hub belongs to A's voice.
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_p) == 1,
        "P should be offered the single choice into its own aside, saw %d",
        get_hub_choice_count(g_ctx, conversation, 0, actor_p));
  CHECK(get_hub_choice_link(g_ctx, conversation, 0, actor_p, 0) == 5,
        "and that choice should lead to hub 5");

  // With the rule on, choosing into hub 5 does not stop there. Hub 5 is one way
  // out, came from a hub that had two, and only P can see it, so it resolves.
  WithRules(DialogueRules_SPOKE | DialogueRules_SINGLEPLAYER, [&] {
    CHECK(select_choice(g_ctx, conversation, actor_p, 0) == 0,
          "the choice into the aside should be accepted");
    CHECK(get_actor_position(g_ctx, conversation, actor_p) == 9,
          "the aside should have resolved on its own and carried P to hub 9, saw %d",
          get_actor_position(g_ctx, conversation, actor_p));
  });

  // A is not dragged onto hub 5 by P leaving it: the aside is private to P, so
  // there is nothing about it A could be carried through to.
  CHECK(get_actor_position(g_ctx, conversation, actor_a) == 0,
        "the other actor should not have followed into a private hub, saw %d",
        get_actor_position(g_ctx, conversation, actor_a));

  // The control: the same shape, but public, so the rule cannot claim it. A
  // speaks A, and P's voice is held by P, so A is offered the public branch at
  // engine index 1 rather than the one at index 0.
  WithRules(DialogueRules_SPOKE | DialogueRules_SINGLEPLAYER, [&] {
    CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_a) == 1,
          "A should be offered just the public branch, saw %d",
          get_hub_choice_count(g_ctx, conversation, 0, actor_a));
    CHECK(get_hub_choice_link(g_ctx, conversation, 0, actor_a, 1) == 6,
          "and it should be the one at engine index 1");

    CHECK(select_choice(g_ctx, conversation, actor_a, 1) == 0,
          "A should be able to take the public branch");
    CHECK(get_actor_position(g_ctx, conversation, actor_a) == 6,
          "a public hub with one choice is an ordinary hub and should be stood on, saw %d",
          get_actor_position(g_ctx, conversation, actor_a));
  });

  remove_conversation(g_ctx, conversation);

  // The same conversation again with the rule off, which is the half that says
  // the rule is what did it. A fresh conversation keeps the first one out of the
  // way without having to reason about where anybody ended up.
  LouderName without = Declare("singleplayer conversation, rule off");
  CHECK(load_conversation_json(g_ctx, without, reinterpret_cast<uint8_t*>(&json[0]), json.size()) == 0,
        "the second singleplayer conversation should load");

  LouderActor plain_p = add_actor(g_ctx, without, 0, ActorFlags(), speak_p, 1, nullptr, 0);
  CHECK(plain_p == 0, "the second conversation's actor should get id 0");

  flush_events(g_ctx);

  WithRules(DialogueRules_SPOKE, [&] {
    CHECK(select_choice(g_ctx, without, plain_p, 0) == 0,
          "without the singleplayer rule the choice into the aside is still ordinary, and accepted");
    CHECK(get_actor_position(g_ctx, without, plain_p) == 5,
          "the actor should have stopped on the aside, saw %d",
          get_actor_position(g_ctx, without, plain_p));
    CHECK(get_hub_choice_count(g_ctx, without, 5, plain_p) == 1,
          "and been offered its one way out, saw %d",
          get_hub_choice_count(g_ctx, without, 5, plain_p));
  });

  remove_conversation(g_ctx, without);
  clear_config(g_ctx);
}

// A load error is reported through the event queue rather than by failing, so
// that one bad record does not cost the caller the rest of the document. The
// whole fixture below is full of them, and the conversation still loads.
//
// Three kinds are here, because the position they report is what tells them
// apart: a record that is not an object and so names no position at all, a
// record of a type the loader does not know, and a connection naming a hub the
// document never defines. The camera record is the control: it is ignored
// outright and must not be reported at all.
const char* BadRecordsJson() {
  return R"([
    {"type":"hub","conversation_id":0,"speaker":"A","text":"start","flags":0},
    {"type":"hub","conversation_id":7,"speaker":"A","text":"good","flags":0},
    "this record is not an object at all",
    {"type":"mystery","conversation_id":1},
    {"type":"connection","from_id":0,"from_port":0,"to_id":99},
    {"type":"camera","position:x":"0.0","position:y":"0.0","zoom":"1.0"}
  ])";
}

void TestLoadErrorsAreReportedAsEvents() {
  std::printf("load errors as events\n");

  LouderName conversation = Declare("bad records conversation");
  std::string json = BadRecordsJson();
  CHECK(load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]),
                               json.size()) == 0,
        "a document with unreadable records should still load, got %d", 0);

  // Both hubs made it, which is the point of reporting rather than failing. Hub
  // 0's connection pointed at a hub that does not exist, so it lost its choice
  // but kept everything else.
  CHECK(get_hub_text(g_ctx, conversation, 0) != LOUDER_NAME_NONE, "hub 0 should have loaded");
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, 0) == 0,
        "hub 0's choice pointed at a hub that is not there, so it should have been dropped");
  CHECK(get_hub_text(g_ctx, conversation, 7) != LOUDER_NAME_NONE, "hub 7 should have loaded");

  std::vector<LoadFailure> failures = DrainLoadErrors();
  CHECK(failures.size() == 3,
        "three records were unreadable, saw %zu", failures.size());

  // A record that is not an object has no position to point at, which is what
  // the -1 means: the document as a whole is at fault rather than a hub in it.
  if (failures.size() == 3) {
    CHECK(failures[0].position == -1,
          "a record that is not an object should report no position, got %d",
          failures[0].position);
    CHECK(failures[0].error == LOUDER_LOAD_NOT_AN_OBJECT,
          "and should report that it was not an object, got %d", failures[0].error);

    // A record of an unknown type names the position it claimed, so a host can
    // go and look at it in the editor.
    CHECK(failures[1].position == 1, "an unknown record should report its own position, got %d",
          failures[1].position);
    CHECK(failures[1].error == LOUDER_LOAD_NOT_AN_OBJECT, "and should report the same error, got %d",
          failures[1].error);

    // A connection to a hub that is not there is blamed on the hub it leaves,
    // since that is the one a host can go and fix.
    CHECK(failures[2].position == 0,
          "a connection to a missing hub should be blamed on its source, got %d",
          failures[2].position);
    CHECK(failures[2].error == LOUDER_LOAD_NOT_AN_OBJECT, "and should report the same error, got %d",
          failures[2].error);
  }

  // A document that is not JSON at all is a problem with the document rather
  // than with a record in it, and is reported the same way before the load is
  // refused.
  flush_events(g_ctx);
  const char* garbage = "not json at all";
  CHECK(load_conversation_json(g_ctx, Declare("garbage conversation"),
                               reinterpret_cast<uint8_t*>(const_cast<char*>(garbage)),
                               std::strlen(garbage)) != 0,
        "malformed JSON should fail to load");
  failures = DrainLoadErrors();
  CHECK(failures.size() == 1, "a malformed document should be reported once, saw %zu",
        failures.size());
  if (failures.size() == 1) {
    CHECK(failures[0].position == -1,
          "a problem with the document as a whole should report no position, got %d",
          failures[0].position);
    CHECK(failures[0].error == LOUDER_LOAD_MALFORMED_JSON,
          "and should say the document was malformed, got %d", failures[0].error);
  }

  // A document that parses but is not a list has no record to point at either.
  flush_events(g_ctx);
  const char* not_a_list = R"({"type":"hub","conversation_id":0})";
  CHECK(load_conversation_json(g_ctx, Declare("object conversation"),
                               reinterpret_cast<uint8_t*>(const_cast<char*>(not_a_list)),
                               std::strlen(not_a_list)) == LOUDER_LOAD_NOT_A_LIST,
        "a document that is not a list should be refused");
  failures = DrainLoadErrors();
  CHECK(failures.size() == 1, "a document that is not a list should be reported once, saw %zu",
        failures.size());
  if (failures.size() == 1) {
    CHECK(failures[0].position == -1, "and should report no position, got %d",
          failures[0].position);
    CHECK(failures[0].error == LOUDER_LOAD_NOT_A_LIST, "and should say so, got %d",
          failures[0].error);
  }

  remove_conversation(g_ctx, conversation);
}

// The config loader reports the same way. It used to give up on the first
// record it could not read, which cost the caller every speaker after it.
void TestConfigLoadErrorsAreReportedAsEvents() {
  std::printf("config load errors as events\n");

  clear_config(g_ctx);
  flush_events(g_ctx);

  const char* config = R"([
    {"type":"speaker","name":"Before","flags":1},
    "this record is not an object at all",
    {"type":"speaker","name":"After","flags":0}
  ])";

  CHECK(load_conversation_config(g_ctx,
                                 reinterpret_cast<uint8_t*>(const_cast<char*>(config)),
                                 std::strlen(config)) == 0,
        "a config with one bad record should still load");

  // The point of the change: the speakers on either side of the bad record both
  // made it, which is what "does not cost the caller the rest" means.
  CHECK(get_speaker_flags(g_ctx, Declare("Before")).bits == SpeakerFlags_PUBLIC.bits,
        "the speaker before the bad record should have loaded");
  CHECK(get_speaker_flags(g_ctx, Declare("After")).bits == 0,
        "the speaker after the bad record should have loaded too");

  std::vector<LoadFailure> failures = DrainLoadErrors();
  CHECK(failures.size() == 1, "the bad record should be reported once, saw %zu", failures.size());
  if (failures.size() == 1) {
    CHECK(failures[0].position == -1,
          "a config record has no position, so it should report the document, got %d",
          failures[0].position);
    CHECK(failures[0].error == LOUDER_LOAD_NOT_AN_OBJECT, "and should say what was wrong, got %d",
          failures[0].error);
  }

  // A config that is not JSON is still refused, and still reported.
  flush_events(g_ctx);
  const char* garbage = "not json at all";
  CHECK(load_conversation_config(g_ctx,
                                 reinterpret_cast<uint8_t*>(const_cast<char*>(garbage)),
                                 std::strlen(garbage)) != 0,
        "a malformed config should fail to load");
  failures = DrainLoadErrors();
  CHECK(failures.size() == 1, "a malformed config should be reported once, saw %zu",
        failures.size());
  if (failures.size() == 1) {
    CHECK(failures[0].error == LOUDER_LOAD_MALFORMED_JSON,
          "and should say the document was malformed, got %d", failures[0].error);
  }

  clear_config(g_ctx);
}

// The shape the demo conversation runs into, reproduced without the demo.
//
// Benjamin stands at a hub that branches, makes a real choice out of it, and
// lands on a hub only he can see because it carries him as a recipient. That is
// the singleplayer rule's premise exactly: a choice was just made, and what it
// led to is his alone, so it is the consequence of that choice rather than
// another one, and it resolves on its own.
//
//   0: Freya, branches -> 10 (his aside)  and  20 (his thought)
//   10: -> 30 (Memory, his private voice)  and  40 (Benjamin, public)
//   20: -> 50
//
// Hub 10 has two choices and hub 20 has one, and the rule treats them the same,
// because how many choices a hub carries is not what makes it a consequence.
// What the rule does need is somewhere else to see it, which hub 60 below is
// for: the same shape, but public, so it belongs to both actors and neither of
// them may be marched through it on the other's say-so.
//
// The second actor is not a bystander here. It is what makes "only he can see
// it" mean something, since an unopposed private hub is private to nobody in
// particular.
const char* PrivateBranchJson() {
  return R"([
    {"type":"hub","conversation_id":0,"speaker":"Freya","text":"ask","tags":"start","flags":0},
    {"type":"hub","conversation_id":10,"speaker":"Benjamin","text":"his aside","recipients":"Benjamin","flags":0},
    {"type":"hub","conversation_id":20,"speaker":"Benjamin","text":"his thought","recipients":"Benjamin","flags":0},
    {"type":"hub","conversation_id":30,"speaker":"Memory","text":"what he remembers","flags":0},
    {"type":"hub","conversation_id":40,"speaker":"Benjamin","text":"what he says","flags":0},
    {"type":"hub","conversation_id":50,"speaker":"Benjamin","text":"past the thought","flags":0},
    {"type":"hub","conversation_id":60,"speaker":"Benjamin","text":"a line they share","flags":0},
    {"type":"hub","conversation_id":70,"speaker":"Freya","text":"the reply","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":20},
    {"type":"connection","from_id":0,"from_port":1,"to_id":10},
    {"type":"connection","from_id":0,"from_port":2,"to_id":60},
    {"type":"connection","from_id":10,"from_port":0,"to_id":30},
    {"type":"connection","from_id":10,"from_port":1,"to_id":40},
    {"type":"connection","from_id":20,"from_port":0,"to_id":50},
    {"type":"connection","from_id":60,"from_port":0,"to_id":70}
  ])";
}

// Benjamin hears Memory, so hub 30 reaches him and nothing else. Freya hears
// neither, which is what keeps the private hubs off her screen while hub 60,
// which is public, stays on it.
const char* PrivateBranchConfigJson() {
  return R"([
    {"type":"speaker","name":"Freya","flags":1},
    {"type":"speaker","name":"Benjamin","flags":1,"listening":"Memory"},
    {"type":"speaker","name":"Memory","flags":0}
  ])";
}

void TestAPrivateHubReachedByAMeaningfulChoice() {
  std::printf("a private hub reached by a meaningful choice\n");

  clear_config(g_ctx);
  CHECK(load_conversation_config(
            g_ctx,
            reinterpret_cast<uint8_t*>(const_cast<char*>(PrivateBranchConfigJson())),
            std::strlen(PrivateBranchConfigJson())) == 0,
        "the private branch config should load");

  LouderName conversation = Declare("private branch conversation");
  std::string json = PrivateBranchJson();
  CHECK(load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]),
                               json.size()) == 0,
        "loading the private branch conversation returned %d", 0);

  LouderName speak_freya[1] = {Lookup("Freya")};
  LouderName speak_benjamin[1] = {Lookup("Benjamin")};

  LouderActor actor_benjamin =
      add_actor(g_ctx, conversation, 0, ActorFlags(), speak_benjamin, 1, nullptr, 0);
  LouderActor actor_freya = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_freya, 1, nullptr, 0);
  CHECK(actor_benjamin == 0 && actor_freya == 1, "the actors should be 0 and 1, got %d and %d",
        actor_benjamin, actor_freya);

  flush_events(g_ctx);

  // Hub 10 is the case. It carries two choices and none of them is hidden from
  // him, so he could perfectly well decide between them, and he is offered
  // both. What settles it is not the count but that the hub is addressed to him
  // alone and stands where a choice he just made put it.
  WithRules(DialogueRules_SPOKE | DialogueRules_SINGLEPLAYER, [&] {
    CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_benjamin) == 3,
          "he should be offered all three of the branches, saw %d",
          get_hub_choice_count(g_ctx, conversation, 0, actor_benjamin));
    CHECK(get_hub_choice_link(g_ctx, conversation, 0, actor_benjamin, 1) == 10,
          "the aside should be at engine index 1");
    CHECK(get_hub_choice_count(g_ctx, conversation, 10, actor_benjamin) == 2,
          "and both of the aside's own choices should be his to look at, saw %d",
          get_hub_choice_count(g_ctx, conversation, 10, actor_benjamin));

    // Choosing into it does not stop there. He is resolved through on the first
    // branch available to him, which is the voice only he hears.
    CHECK(select_choice(g_ctx, conversation, actor_benjamin, 1) == 0,
          "the choice into his aside should be accepted");
    CHECK(get_actor_position(g_ctx, conversation, actor_benjamin) == 30,
          "the aside should have resolved on its own, carrying him to the branch he alone hears at "
          "hub 30, saw %d", get_actor_position(g_ctx, conversation, actor_benjamin));
  });

  // Freya had no claim on either branch of hub 10, since it is addressed to him.
  // But she is not deciding anything either, so she is emulated through it and
  // takes the first branch that appears to her, which is hub 40 and not the hub
  // 30 she cannot hear. Being carried is not the same as being able to choose,
  // and this is where the two actors come apart.
  CHECK(get_actor_position(g_ctx, conversation, actor_freya) == 40,
        "Freya should have been carried through the aside onto the branch she can hear, saw %d",
        get_actor_position(g_ctx, conversation, actor_freya));

  // Hub 20 has one choice rather than two, and resolves the same way. The count
  // is not what does it, which is why both are here.
  force_position(g_ctx, conversation, actor_benjamin, 0, LOUDER_NAME_NONE, 0);
  force_position(g_ctx, conversation, actor_freya, 0, LOUDER_NAME_NONE, 0);
  flush_events(g_ctx);
  WithRules(DialogueRules_SPOKE | DialogueRules_SINGLEPLAYER, [&] {
    CHECK(select_choice(g_ctx, conversation, actor_benjamin, 0) == 0,
          "the choice into his thought should be accepted");
    CHECK(get_actor_position(g_ctx, conversation, actor_benjamin) == 50,
          "one choice out of a hub that branched resolves on its own too, carrying him to hub 50, "
          "saw %d", get_actor_position(g_ctx, conversation, actor_benjamin));
  });

  // Hub 60 is the control. Same shape, same single choice, reached from the same
  // branching hub, but public: both actors can see it, so it belongs to both of
  // them and the rule cannot claim it on one of their say-so. Without this the
  // rule would just be "any hub an actor walks into resolves".
  force_position(g_ctx, conversation, actor_benjamin, 0, LOUDER_NAME_NONE, 0);
  force_position(g_ctx, conversation, actor_freya, 0, LOUDER_NAME_NONE, 0);
  flush_events(g_ctx);
  WithRules(DialogueRules_SPOKE | DialogueRules_SINGLEPLAYER, [&] {
    CHECK(select_choice(g_ctx, conversation, actor_benjamin, 2) == 0,
          "the shared line should be an ordinary choice");
    CHECK(get_actor_position(g_ctx, conversation, actor_benjamin) == 60,
          "and he should stand on it, since the other actor can see it too, saw %d",
          get_actor_position(g_ctx, conversation, actor_benjamin));
  });

  // The one thing that would make the rule worth having: with it off, the same
  // move stops where the player was sent.
  force_position(g_ctx, conversation, actor_benjamin, 0, LOUDER_NAME_NONE, 0);
  force_position(g_ctx, conversation, actor_freya, 0, LOUDER_NAME_NONE, 0);
  flush_events(g_ctx);
  WithRules(DialogueRules_SPOKE, [&] {
    CHECK(select_choice(g_ctx, conversation, actor_benjamin, 1) == 0,
          "without the singleplayer rule the choice into his aside is ordinary, and accepted");
    CHECK(get_actor_position(g_ctx, conversation, actor_benjamin) == 10,
          "he should have stopped on it, since it is a decision he could have made, saw %d",
          get_actor_position(g_ctx, conversation, actor_benjamin));
    CHECK(get_hub_choice_count(g_ctx, conversation, 10, actor_benjamin) == 2,
          "and been offered both of its branches, saw %d",
          get_hub_choice_count(g_ctx, conversation, 10, actor_benjamin));
  });

  remove_conversation(g_ctx, conversation);
  clear_config(g_ctx);
}

// The other demo case. An interface shows a side with exactly one choice as
// CONTINUE, so both sides have to be able to be in that state at the same hub.
// What makes that possible is a hub nobody owns: it has no speaker and no
// recipients, so it reaches every actor, and it is held by nobody, so ownership
// never comes into it.
//
//   0: -> 60 -> 70
//
// Hub 60 is the one both actors stand on to be "continuing".
const char* ContinueJson() {
  return R"([
    {"type":"hub","conversation_id":0,"speaker":"A","text":"start","tags":"start","flags":0},
    {"type":"hub","conversation_id":60,"text":"","flags":0},
    {"type":"hub","conversation_id":70,"text":"the end","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":60},
    {"type":"connection","from_id":60,"from_port":0,"to_id":70}
  ])";
}

void TestAnUnownedHubWithOneChoiceIsEveryonesToTake() {
  std::printf("an unowned hub with one choice is everyone's to take\n");

  LouderName conversation = Declare("continue conversation");
  std::string json = ContinueJson();
  CHECK(load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]),
                               json.size()) == 0,
        "loading the continue conversation returned %d", 0);

  LouderName speak_a[1] = {Lookup("A")};
  LouderName speak_b[1] = {Lookup("B")};
  LouderActor actor_a = add_actor(g_ctx, conversation, 60, ActorFlags(), speak_a, 1, nullptr, 0);
  LouderActor actor_b = add_actor(g_ctx, conversation, 60, ActorFlags(), speak_b, 1, nullptr, 0);
  CHECK(actor_a == 0 && actor_b == 1, "the actors should be 0 and 1, got %d and %d", actor_a,
        actor_b);

  flush_events(g_ctx);

  // Hub 60 has no speaker and no recipients, so it reaches both actors and belongs
  // to neither. That is the whole premise of a CONTINUE: two sides that are both
  // showing one choice and neither of them is the other one's to take away.
  CHECK(get_hub_speaker(g_ctx, conversation, 60) == LOUDER_NAME_NONE,
        "hub 60 should carry no speaker, which is what leaves it unowned");
  CHECK(get_hub_choice_count(g_ctx, conversation, 60, actor_a) == 1,
        "the first actor should see exactly one choice, saw %d",
        get_hub_choice_count(g_ctx, conversation, 60, actor_a));
  CHECK(get_hub_choice_count(g_ctx, conversation, 60, actor_b) == 1,
        "and so should the second, which is what puts both sides in that state, saw %d",
        get_hub_choice_count(g_ctx, conversation, 60, actor_b));
  CHECK(get_hub_choice_link(g_ctx, conversation, 60, actor_a, 0) == 70,
        "the choice should lead to hub 70");
  CHECK(get_hub_choice_link(g_ctx, conversation, 60, actor_b, 0) == 70,
        "and to the same place for both, since neither of them chose for the other");

  // Each may take it for itself. What that does not mean is that one side
  // decides for the other: the other is carried, not commanded, and only
  // because the two share a channel and hub 70 is visible to both. A side on its
  // own channel would simply stay put, which is the case that keeps a CONTINUE
  // from being mistaken for one player moving the other.
  WithRules(DialogueRules_SPOKE | DialogueRules_SINGLEPLAYER, [&] {
    CHECK(select_choice(g_ctx, conversation, actor_a, 0) == 0,
          "the first actor should be able to continue");
    CHECK(get_actor_position(g_ctx, conversation, actor_a) == 70,
        "and should land on hub 70, saw %d", get_actor_position(g_ctx, conversation, actor_a));
    CHECK(get_actor_position(g_ctx, conversation, actor_b) == 70,
          "and the second should follow it there, since they share a channel and can both see hub "
          "70, saw %d", get_actor_position(g_ctx, conversation, actor_b));
  });

  // Put on separate channels and neither moves the other, which is what makes
  // the choice each side's rather than the conversation's.
  force_position(g_ctx, conversation, actor_a, 60, Declare("left"), 0);
  force_position(g_ctx, conversation, actor_b, 60, Declare("right"), 0);
  flush_events(g_ctx);
  WithRules(DialogueRules_SPOKE | DialogueRules_SINGLEPLAYER, [&] {
    CHECK(select_choice(g_ctx, conversation, actor_a, 0) == 0,
          "the first actor should still be able to continue");
    CHECK(get_actor_position(g_ctx, conversation, actor_a) == 70,
          "landing on hub 70, saw %d", get_actor_position(g_ctx, conversation, actor_a));
    CHECK(get_actor_position(g_ctx, conversation, actor_b) == 60,
          "while the other side, in another channel, is left to continue on its own, saw %d",
          get_actor_position(g_ctx, conversation, actor_b));
    CHECK(get_hub_choice_count(g_ctx, conversation, 60, actor_b) == 1,
          "and is still offered its one way on, saw %d",
          get_hub_choice_count(g_ctx, conversation, 60, actor_b));
  });

  // And it is a genuine choice rather than something the engine pushes through:
  // with the singleplayer rule off it is still offered, which is the difference
  // between "both sides may continue" and "both sides are continuing".
  force_position(g_ctx, conversation, actor_a, 60, LOUDER_NAME_NONE, 0);
  force_position(g_ctx, conversation, actor_b, 60, LOUDER_NAME_NONE, 0);
  WithRules(DialogueRules_SPOKE, [&] {
    CHECK(get_hub_choice_count(g_ctx, conversation, 60, actor_a) == 1,
          "an unowned hub is not the singleplayer rule's business, and stays a choice, saw %d",
          get_hub_choice_count(g_ctx, conversation, 60, actor_a));
    CHECK(get_hub_choice_count(g_ctx, conversation, 60, actor_b) == 1,
          "on both sides, saw %d", get_hub_choice_count(g_ctx, conversation, 60, actor_b));
  });

  remove_conversation(g_ctx, conversation);
}

// A hub with no speaker of its own is anonymous: it says whatever whoever is
// standing on it says, and the voice is projected onto the hub for as long as
// that actor is there.
//
//   0 -> 5 -> 9
//
// Every hub here is anonymous, so the conversation does not depend on who is
// public: an anonymous hub is reached by everybody and owned by nobody.
const char* AnonymousJson() {
  return R"([
    {"type":"hub","conversation_id":0,"text":"start","tags":"start","flags":0},
    {"type":"hub","conversation_id":5,"text":"anonymous","flags":0},
    {"type":"hub","conversation_id":9,"text":"the end","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":5},
    {"type":"connection","from_id":5,"from_port":0,"to_id":9}
  ])";
}

// The same conversation, except that hub 5 asks who would be speaking it and
// turns itself away from that actor.
const char* GatedJson() {
  return R"([
    {"type":"hub","conversation_id":0,"text":"start","tags":"start","flags":0},
    {"type":"hub","conversation_id":5,"text":"gated","expressions":"%speaker != \"A\"","flags":0},
    {"type":"hub","conversation_id":9,"text":"the end","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":5},
    {"type":"connection","from_id":5,"from_port":0,"to_id":9}
  ])";
}

// The same conversation again, except that hub 5 remembers who would be
// speaking it.
const char* RememberedJson() {
  return R"([
    {"type":"hub","conversation_id":0,"text":"start","tags":"start","flags":0},
    {"type":"hub","conversation_id":5,"text":"remembered","expressions":"last = %speaker","flags":0},
    {"type":"hub","conversation_id":9,"text":"the end","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":5},
    {"type":"connection","from_id":5,"from_port":0,"to_id":9}
  ])";
}

// And again, except that hub 5 says who it is speaking as, whatever the actor
// walking onto it would have said.
const char* AssignedJson() {
  return R"([
    {"type":"hub","conversation_id":0,"text":"start","tags":"start","flags":0},
    {"type":"hub","conversation_id":5,"text":"assigned","expressions":"%speaker = \"Narrator\"","flags":0},
    {"type":"hub","conversation_id":9,"text":"the end","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":5},
    {"type":"connection","from_id":5,"from_port":0,"to_id":9}
  ])";
}

void TestAnAnonymousHubSpeaksAsWhoeverWalksOntoIt() {
  std::printf("an anonymous hub speaks as whoever walks onto it\n");

  LouderName conversation = Declare("anonymous conversation");
  std::string json = AnonymousJson();
  CHECK(load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]),
                               json.size()) == 0,
        "loading the anonymous conversation returned %d", 0);

  LouderName speak_a[1] = {Declare("A")};
  LouderName speak_b[1] = {Declare("B")};
  LouderActor actor_a = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  LouderActor actor_b = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_b, 1, nullptr, 0);

  // Put the two actors in their own channels, so that neither is dragged along
  // by the other and each of them walks onto the hub on its own.
  force_position(g_ctx, conversation, actor_a, 0, Declare("left"), 0);
  force_position(g_ctx, conversation, actor_b, 0, Declare("right"), 0);
  flush_events(g_ctx);

  // The hub carries no speaker of its own, which is what makes it anonymous.
  CHECK(get_hub_speaker(g_ctx, conversation, 5) == LOUDER_NAME_NONE,
        "hub 5 should carry no speaker of its own");

  // The voice it answers with is the one of the actor asking, so the same hub
  // answers differently for each of them.
  CHECK(ReadName(get_hub_speaker_for_actor(g_ctx, conversation, 5, actor_a)) == "A",
        "the hub should answer as A for the first actor, saw %s",
        ReadName(get_hub_speaker_for_actor(g_ctx, conversation, 5, actor_a)).c_str());
  CHECK(ReadName(get_hub_speaker_for_actor(g_ctx, conversation, 5, actor_b)) == "B",
        "and as B for the second, saw %s",
        ReadName(get_hub_speaker_for_actor(g_ctx, conversation, 5, actor_b)).c_str());

  // Standard is the first matching choice rule: each actor is taking a hub
  // projected into its own voice, not exercising a skill or speaking for nobody.
  CHECK(get_hub_choice_rule(g_ctx, conversation, 0, actor_a, 0) == static_cast<int32_t>(LouderRule::STANDARD),
        "A's anonymous choice should use the standard rule");
  CHECK(get_hub_choice_rule(g_ctx, conversation, 0, actor_b, 0) == static_cast<int32_t>(LouderRule::STANDARD),
        "B's anonymous choice should use the standard rule");

  // The first actor walks onto the hub, which speaks in that actor's voice.
  CHECK(select_choice(g_ctx, conversation, actor_a, 0) == 0,
        "the first actor should be able to walk onto the anonymous hub");
  CHECK(get_actor_position(g_ctx, conversation, actor_a) == 5,
        "and should land on it, saw %d", get_actor_position(g_ctx, conversation, actor_a));

  std::vector<LouderEvent> events = DrainEvents();
  Motion motion{};
  CHECK(FindMotion(events, actor_a, 5, &motion), "the move onto the hub should be announced");
  CHECK(ReadName(motion.speaker) == "A", "the move should be spoken by A, saw %s",
        ReadName(motion.speaker).c_str());

  // The second actor walks onto the same hub later, and the hub speaks in that
  // actor's voice instead: the voice is projected onto the hub for the length
  // of the move, and no longer.
  CHECK(select_choice(g_ctx, conversation, actor_b, 0) == 0,
        "the second actor should be able to walk onto the same hub");
  CHECK(get_actor_position(g_ctx, conversation, actor_b) == 5,
        "and should land on it too, saw %d", get_actor_position(g_ctx, conversation, actor_b));

  events = DrainEvents();
  CHECK(FindMotion(events, actor_b, 5, &motion), "the second move onto the hub should be announced");
  CHECK(ReadName(motion.speaker) == "B", "the second move should be spoken by B, saw %s",
        ReadName(motion.speaker).c_str());

  // And the hub itself is left as it was: it is still anonymous, and it still
  // answers for each actor in that actor's own voice.
  CHECK(get_hub_speaker(g_ctx, conversation, 5) == LOUDER_NAME_NONE,
        "the hub should still carry no speaker of its own");
  CHECK(ReadName(get_hub_speaker_for_actor(g_ctx, conversation, 5, actor_a)) == "A",
        "the hub should still answer as A for the first actor, saw %s",
        ReadName(get_hub_speaker_for_actor(g_ctx, conversation, 5, actor_a)).c_str());
  CHECK(ReadName(get_hub_speaker_for_actor(g_ctx, conversation, 5, actor_b)) == "B",
        "and as B for the second, saw %s",
        ReadName(get_hub_speaker_for_actor(g_ctx, conversation, 5, actor_b)).c_str());

  remove_conversation(g_ctx, conversation);
}

void TestTheSpeakerVariableGatesAHubPerActor() {
  std::printf("the speaker variable gates a hub per actor\n");

  LouderName conversation = Declare("gated conversation");
  std::string json = GatedJson();
  CHECK(load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]),
                               json.size()) == 0,
        "loading the gated conversation returned %d", 0);

  LouderName speak_a[1] = {Declare("A")};
  LouderName speak_b[1] = {Declare("B")};
  LouderActor actor_a = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  LouderActor actor_b = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_b, 1, nullptr, 0);

  flush_events(g_ctx);

  // The hub asks who would be speaking it, and turns itself away from that
  // actor: it is there for the other one.
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_a) == 0,
        "the hub should not be offered to the actor it would speak as, saw %d",
        get_hub_choice_count(g_ctx, conversation, 0, actor_a));
  CHECK(get_hub_choice_count(g_ctx, conversation, 0, actor_b) == 1,
        "and should be offered to the other one, saw %d",
        get_hub_choice_count(g_ctx, conversation, 0, actor_b));

  // The actor it was offered to takes it, and lands on the hub.
  CHECK(select_choice(g_ctx, conversation, actor_b, 0) == 0,
        "the second actor should be able to take the gated hub");
  CHECK(get_actor_position(g_ctx, conversation, actor_b) == 5,
        "and should land on it, saw %d", get_actor_position(g_ctx, conversation, actor_b));
  CHECK(get_actor_position(g_ctx, conversation, actor_a) == 0,
        "while the actor it turned away from stays put, saw %d",
        get_actor_position(g_ctx, conversation, actor_a));

  remove_conversation(g_ctx, conversation);
}

void TestTheSpeakerVariableCanBeStoredInStoryState() {
  std::printf("the speaker variable can be stored in story state\n");

  LouderName conversation = Declare("remembered speaker conversation");
  std::string json = RememberedJson();
  CHECK(load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]),
                               json.size()) == 0,
        "loading the remembered speaker conversation returned %d", 0);

  LouderName speak_a[1] = {Declare("A")};
  LouderActor actor_a = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);

  flush_events(g_ctx);

  // The hub remembers who would be speaking it, which is the actor's own voice.
  CHECK(select_choice(g_ctx, conversation, actor_a, 0) == 0,
        "the actor should be able to walk onto the hub");
  CHECK(get_actor_position(g_ctx, conversation, actor_a) == 5,
        "and should land on it, saw %d", get_actor_position(g_ctx, conversation, actor_a));

  LouderVariant stored = get_story_state(g_ctx, conversation, Declare("last"));
  CHECK(stored.tag == LouderVariant::Tag::String, "the stored value should be a string");
  if (stored.tag == LouderVariant::Tag::String) {
    CHECK(ReadName(stored.string._0) == "A",
          "the hub should have stored the voice it would have been spoken by, saw %s",
          ReadName(stored.string._0).c_str());
  }

  remove_conversation(g_ctx, conversation);
}

void TestAssigningTheSpeakerChangesTheHubForThatArrivalOnly() {
  std::printf("assigning the speaker changes the hub for that arrival only\n");

  LouderName conversation = Declare("assigned speaker conversation");
  std::string json = AssignedJson();
  CHECK(load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]),
                               json.size()) == 0,
        "loading the assigned speaker conversation returned %d", 0);

  LouderName speak_a[1] = {Declare("A")};
  LouderName speak_b[1] = {Declare("B")};
  LouderActor actor_a = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  LouderActor actor_b = add_actor(g_ctx, conversation, 0, ActorFlags(), speak_b, 1, nullptr, 0);

  force_position(g_ctx, conversation, actor_a, 0, Declare("left"), 0);
  force_position(g_ctx, conversation, actor_b, 0, Declare("right"), 0);
  flush_events(g_ctx);

  // The hub says who it is speaking as, and the move it belongs to is spoken
  // in that voice rather than in the actor's.
  CHECK(select_choice(g_ctx, conversation, actor_a, 0) == 0,
        "the first actor should be able to walk onto the hub");
  std::vector<LouderEvent> events = DrainEvents();
  Motion motion{};
  CHECK(FindMotion(events, actor_a, 5, &motion), "the move onto the hub should be announced");
  CHECK(ReadName(motion.speaker) == "Narrator",
        "the move should be spoken by the voice the hub chose, saw %s",
        ReadName(motion.speaker).c_str());

  // The hub itself is left as it was: the assignment belongs to the arrival, so
  // the hub is still anonymous and still answers in each actor's own voice.
  CHECK(get_hub_speaker(g_ctx, conversation, 5) == LOUDER_NAME_NONE,
        "the hub should still carry no speaker of its own");
  CHECK(ReadName(get_hub_speaker_for_actor(g_ctx, conversation, 5, actor_a)) == "A",
        "the hub should still answer as A for the first actor, saw %s",
        ReadName(get_hub_speaker_for_actor(g_ctx, conversation, 5, actor_a)).c_str());

  // And the assignment happens on every arrival rather than only the first: a
  // later actor walking onto the same hub is spoken by the same voice.
  CHECK(select_choice(g_ctx, conversation, actor_b, 0) == 0,
        "the second actor should be able to walk onto the same hub");
  events = DrainEvents();
  CHECK(FindMotion(events, actor_b, 5, &motion), "the second move onto the hub should be announced");
  CHECK(ReadName(motion.speaker) == "Narrator",
        "the second move should be spoken by the same voice, saw %s",
        ReadName(motion.speaker).c_str());

  remove_conversation(g_ctx, conversation);
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

void TestTraces() {
  std::printf("traces\n");

  flush_traced_events(g_ctx);
  flush_traced_asks(g_ctx);

  CHECK(get_traced_event(g_ctx, 0).tag == LouderOption<LouderEvent>::Tag::None,
        "the event trace should start empty");
  CHECK(get_traced_ask(g_ctx, 0).tag == LouderOption<LouderAsk>::Tag::None,
        "the ask trace should start empty");

  LouderEvent event = BareEvent(make_event_data_actor_added());
  event.actor = 7;
  CHECK(test_push_event(g_ctx, &event) == LouderError::Ok, "pushing a built event should succeed");

  // Flushing the queue is what processing an event looks like, and the trace is
  // the history of that event rather than a second view of the queue.
  flush_events(g_ctx);
  LouderOption<LouderEvent> traced = get_traced_event(g_ctx, 0);
  CHECK(traced.tag == LouderOption<LouderEvent>::Tag::Some, "flushing events should leave the event trace");
  CHECK(traced.some._0.actor == 7, "the traced event should be a copy of the event that was pushed");
  CHECK(traced.some._0.data.tag == EventData::Tag::ActorAdded,
        "the traced event should keep its kind");

  LouderAsk ask{};
  ask.ask = Declare("traced callback");
  size_t ask_index = 0;
  while (get_ask(g_ctx, ask_index).tag == LouderOption<LouderAsk>::Tag::Some) {
    ++ask_index;
  }
  CHECK(push_ask(g_ctx, &ask) == LouderError::Ok, "pushing an ask should succeed");
  CHECK(answer_ask(g_ctx, ask_index, 1) == LouderError::Ok, "answering an ask should succeed");

  LouderOption<LouderAsk> traced_ask = get_traced_ask(g_ctx, 0);
  CHECK(traced_ask.tag == LouderOption<LouderAsk>::Tag::Some, "answering should leave the ask trace");
  CHECK(ReadName(traced_ask.some._0.ask) == "traced callback",
        "the traced ask should keep its callback name, saw %s",
        ReadName(traced_ask.some._0.ask).c_str());

  std::vector<uint8_t> buffer(65536, 0);
  int32_t written = read_trace_json(g_ctx, buffer.data(), buffer.size() - 1);
  CHECK(written > 0, "the trace JSON should be written");
  std::string json(reinterpret_cast<char*>(buffer.data()), written > 0 ? written : 0);
  CHECK(json.find("\"events\":[") != std::string::npos, "the trace should carry events, got %s", json.c_str());
  CHECK(json.find("\"asks\":[") != std::string::npos, "the trace should carry asks, got %s", json.c_str());
  CHECK(json.find("\"kind\":12") != std::string::npos, "the trace should describe the traced event, got %s",
        json.c_str());

  flush_traced_events(g_ctx);
  CHECK(get_traced_event(g_ctx, 0).tag == LouderOption<LouderEvent>::Tag::None,
        "flushing the event trace should empty it");
  CHECK(get_traced_ask(g_ctx, 0).tag == LouderOption<LouderAsk>::Tag::Some,
        "flushing the event trace should leave the ask trace alone");

  flush_traced_asks(g_ctx);
  CHECK(get_traced_ask(g_ctx, 0).tag == LouderOption<LouderAsk>::Tag::None,
        "flushing the ask trace should empty it");

  CHECK(get_traced_event(nullptr, 0).tag == LouderOption<LouderEvent>::Tag::None,
        "get_traced_event should reject null");
  CHECK(get_traced_ask(nullptr, 0).tag == LouderOption<LouderAsk>::Tag::None,
        "get_traced_ask should reject null");
  CHECK(read_trace_json(nullptr, buffer.data(), buffer.size() - 1) == -1,
        "read_trace_json should reject null");
  flush_traced_events(nullptr);
  flush_traced_asks(nullptr);
}

void TestSerialization() {
  std::printf("serialization\n");

  LouderName conversation = Declare("serialized conversation");
  std::string json = ConversationJson();
  load_conversation_json(g_ctx, conversation, reinterpret_cast<uint8_t*>(&json[0]), json.size());

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

  LouderEvent motion = BareEvent(make_event_data_regular_motion(Motion{0, 5, 1, LOUDER_NAME_NONE, false}));
  CHECK(test_push_event(g_ctx, &motion) == LouderError::Ok, "pushing a built motion should succeed");

  LouderEvent failure = BareEvent(make_event_data_load_error(LoadFailure{9, LOUDER_LOAD_NOT_AN_OBJECT}));
  CHECK(test_push_event(g_ctx, &failure) == LouderError::Ok, "pushing a built failure should succeed");

  std::vector<LouderEvent> events = DrainEvents();
  CHECK(events.size() == 3, "all three pushed events should come back, saw %zu", events.size());
  CHECK(events.size() == 3 && events[0].data.tag == EventData::Tag::Callback,
        "the first event should still read as a callback");
  CHECK(events.size() == 3 && ReadName(events[0].data.callback._0) == "on_arrival",
        "the callback should still carry its name");
  CHECK(events.size() == 3 && events[1].data.tag == EventData::Tag::RegularMotion,
        "the second event should still read as a motion");
  CHECK(events.size() == 3 && events[1].data.regular_motion._0.target_position == 5,
        "the motion should still carry its target position");
  CHECK(events.size() == 3 && events[1].data.regular_motion._0.choice_index == 1,
        "the motion should still carry its choice index");
  CHECK(events.size() == 3 && events[2].data.tag == EventData::Tag::LoadError,
        "the third event should still read as a load error");
  CHECK(events.size() == 3 && events[2].data.load_error._0.position == 9,
        "the load error should still carry its position");
  CHECK(events.size() == 3 && events[2].data.load_error._0.error == LOUDER_LOAD_NOT_AN_OBJECT,
        "the load error should still carry its error code");
}

void TestNullSafety() {
  std::printf("null handles\n");

  CHECK(init_louder() != nullptr, "initialising should succeed");

  CHECK(get_event(nullptr, 0).tag == LouderOption<LouderEvent>::Tag::None, "get_event should reject null");
  CHECK(get_ask(nullptr, 0).tag == LouderOption<LouderAsk>::Tag::None, "get_ask should reject null");
  CHECK(get_hub_text(nullptr, 0, 0) == LOUDER_NAME_NONE, "get_hub_text should reject null");
  CHECK(get_actor_position(nullptr, 0, 0) == -1, "get_actor_position should reject null");
  CHECK(get_story_state(nullptr, 0, 0).tag == LouderVariant::Tag::Int, "get_story_state should reject null");
  CHECK(read_name(nullptr, 0, nullptr, 0) == 0, "read_name should reject null");
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
  TestConditionsUseTheGuidesTruthinessRules();
  TestActors();
  TestChoices();
  TestPrivateHubIsOwnedByItsRecipients();
  TestWarpRecursion();
  TestAutoCycle();
  TestChoiceCountAgreesWithProbing();
  TestAvailableChoiceIndicesCanHaveGaps();
  TestExampleFlow();
  TestAutoSplitOnPrivateVoice();
  TestSpokeRuleIsConfigurable();
  TestSingleplayerRule();
  TestAPrivateHubReachedByAMeaningfulChoice();
  TestAnUnownedHubWithOneChoiceIsEveryonesToTake();
  TestAnAnonymousHubSpeaksAsWhoeverWalksOntoIt();
  TestTheSpeakerVariableGatesAHubPerActor();
  TestTheSpeakerVariableCanBeStoredInStoryState();
  TestAssigningTheSpeakerChangesTheHubForThatArrivalOnly();
  TestLoadErrorsAreReportedAsEvents();
  TestConfigLoadErrorsAreReportedAsEvents();
  TestAsks();
  TestTraces();
  TestSerialization();
  TestBuiltEvents();
  TestNullSafety();

  deinit_louder(g_ctx);

  std::printf("\n%d checks, %d failures\n", g_checks, g_failures);
  return g_failures == 0 ? 0 : 1;
}