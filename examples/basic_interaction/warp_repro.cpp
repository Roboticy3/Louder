// Minimal isolation of a suspected infinite recursion in the engine.
//
// One hub spoken by A with a single choice to hub 5, also spoken by A. Two
// actors: one speaking as A, one speaking as B. Only the A actor can take the
// choice (A is spoken by nobody else, and the B actor does not speak as A).
// The B actor is standing on the same hub and in the same channel.
//
// When A chooses, the engine's warp_followers finds B eligible and moves B to
// the leader's destination. That move calls travel on B, which calls
// warp_followers again, which now finds A eligible at the same destination...
#include <cstdio>
#include <cstring>
#include <string>

#include "louder_rust.h"

static LouderContext* ctx = nullptr;

static LouderName Declare(const char* text) {
  LouderName existing = get_named_id(ctx, reinterpret_cast<const uint8_t*>(text), std::strlen(text));
  if (existing != LOUDER_NAME_NONE) {
    return existing;
  }
  return declare_name(ctx, reinterpret_cast<const uint8_t*>(text), std::strlen(text));
}

static LouderName Lookup(const char* text) {
  return get_named_id(ctx, reinterpret_cast<const uint8_t*>(text), std::strlen(text));
}

static std::string ReadName(LouderName name) {
  if (name == LOUDER_NAME_NONE) return "<none>";
  std::string buffer(4096, '\0');
  size_t written = read_name(ctx, name, reinterpret_cast<uint8_t*>(&buffer[0]), buffer.size() - 1, 0);
  buffer.resize(written);
  return buffer;
}

// 0: the follower starts on the same hub as the leader.
// 1: the follower starts elsewhere, so the ping-pong would have to travel
//    between two positions instead.
static int VariantFollowerElsewhere = 0;

int main(int argc, char** argv) {
  if (argc > 1) {
    VariantFollowerElsewhere = std::atoi(argv[1]);
  }
  ctx = init_louder();
  std::setvbuf(stdout, nullptr, _IONBF, 0);

  const char* config = R"([
    {"type":"speaker","name":"A","flags":1},
    {"type":"speaker","name":"B","flags":1}
  ])";

  const char* conversation = R"([
    {"type":"hub","conversation_id":0,"speaker":"A","text":"hello","tags":"start","flags":0},
    {"type":"hub","conversation_id":5,"speaker":"A","text":"world","flags":0},
    {"type":"connection","from_id":0,"from_port":0,"to_id":5}
  ])";

  std::printf("config=%d\n", load_conversation_config(
      ctx, reinterpret_cast<uint8_t*>(const_cast<char*>(config)), std::strlen(config), 0));

  LouderName conv = Declare("repro");
  std::printf("conversation=%d\n", load_conversation_json(
      ctx, conv, reinterpret_cast<uint8_t*>(const_cast<char*>(conversation)),
      std::strlen(conversation), 0));

  LouderName a = Declare("A");
  LouderName b = Declare("B");

  // The A actor speaks as A, so only it may take the choice. The B actor is a
  // bystander on the same hub and in the same channel.
  LouderName speak_a[1] = {a};
  LouderName speak_b[1] = {b};
  LouderActor leader = add_actor(ctx, conv, 0, ActorFlags(), speak_a, 1, nullptr, 0);
  // Variant: the follower starts somewhere else, on hub 5 already.
  LouderActor follower = add_actor(ctx, conv, VariantFollowerElsewhere ? 5 : 0,
                                   ActorFlags(), speak_b, 1, nullptr, 0);
  std::printf("leader=%d follower=%d\n", leader, follower);
  std::printf("leader choices=%d follower choices=%d\n",
              get_hub_choice_count(ctx, conv, 0, leader),
              get_hub_choice_count(ctx, conv, 0, follower));

  flush_events(ctx);

  // Variant 2: put the follower on a different channel, so it is not in the
  // leader's channel and therefore cannot be dragged along. If the crash is in
  // the follower path, this should survive.
  if (VariantFollowerElsewhere == 2) {
    LouderName other = Declare("other");
    force_position(ctx, conv, follower, 5, other, 0);
    flush_events(ctx);
    std::printf("follower moved to channel=%s pos=%d, leader channel=%s\n",
                ReadName(get_actor_channel(ctx, conv, follower)).c_str(),
                get_actor_position(ctx, conv, follower),
                ReadName(get_actor_channel(ctx, conv, leader)).c_str());
  }

  // Variant 3: give the follower a higher priority, so it is shown the hub in
  // the background instead of being warped along.
  if (VariantFollowerElsewhere == 3) {
    force_position(ctx, conv, follower, VariantFollowerElsewhere == 3 ? 5 : 0,
                   LOUDER_NAME_NONE, 1);
    flush_events(ctx);
    std::printf("follower priority=%d leader priority=%d\n",
                get_actor_priority(ctx, conv, follower),
                get_actor_priority(ctx, conv, leader));
  }

  std::printf("selecting...\n");
  int32_t result = select_choice(ctx, conv, leader, 0);
  std::printf("returned %d\n", result);
  std::printf("leader pos=%d follower pos=%d\n",
              get_actor_position(ctx, conv, leader),
              get_actor_position(ctx, conv, follower));

  // Count the motions, to see whether this ever terminates.
  int arrivals = 0;
  for (size_t i = 0;; ++i) {
    LouderOption<LouderEvent> option = get_event(ctx, i);
    if (option.tag == LouderOption<LouderEvent>::Tag::None) break;
    LouderEvent event = option.some._0;
    if (event.data.tag == EventData::Tag::RegularMotion || event.data.tag == EventData::Tag::Warp) {
      const Motion& motion = event.data.tag == EventData::Tag::Warp ? event.data.warp._0
                                                                    : event.data.regular_motion._0;
      std::printf("  event %zu actor=%d %d->%d ci=%d\n", i, event.actor,
                  motion.source_position, motion.target_position, motion.choice_index);
      ++arrivals;
    }
  }
  std::printf("arrivals=%d\n", arrivals);

  deinit_louder(ctx);
  return 0;
}