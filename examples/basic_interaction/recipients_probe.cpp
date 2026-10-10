// Throwaway: does hub 26, which is private to Benjamin, actually appear to
// Benjamin?
//
// The recipients rule in docs/docs/engine/index.md is "the hub may only appear
// to actors speaking as OR listening to speakers in this list." So speaking as
// Benjamin should be enough, even though Benjamin does not listen to Benjamin.
//
// Visibility has no direct FFI call, so it is inferred two ways that do not rely
// on the same code path as the guess being tested:
//
//   1. Choice availability. choice_available requires the *linked* hub to appear
//      to the actor, so an offered choice out of 12 into 26 proves 26 appears.
//   2. Warping. warp_followers only drags a follower to a hub that appears to
//      them, so being dragged to 26 proves the same thing independently.
#include <cstdio>
#include <cstring>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "louder_rust.h"

static LouderContext* ctx = nullptr;

static std::string Name(LouderName n) {
  if (n == LOUDER_NAME_NONE) return "<none>";
  std::vector<uint8_t> buf(4096, 0);
  size_t w = read_name(ctx, n, buf.data(), buf.size() - 1, 0);
  return std::string(reinterpret_cast<char*>(buf.data()), w);
}

static LouderName Declare(const char* s) {
  LouderName e = get_named_id(ctx, reinterpret_cast<const uint8_t*>(s), std::strlen(s));
  return e != LOUDER_NAME_NONE ? e : declare_name(ctx, reinterpret_cast<const uint8_t*>(s), std::strlen(s));
}

static std::string ReadFile(const char* path) {
  std::ifstream f(path, std::ios::binary);
  if (!f) return "";
  std::ostringstream ss;
  ss << f.rdbuf();
  return ss.str();
}

// Add a throwaway actor at `pos` speaking as `speaker`, report what it can see
// from there, then remove it again.
static void probe_at(LouderName conv, LouderPosition pos, const char* speaker, const char* label) {
  LouderName sp = Declare(speaker);
  LouderActor a = add_actor(ctx, conv, pos, ActorFlags(), &sp, 1, nullptr, 0);
  if (a == LOUDER_ACTOR_NONE) {
    std::printf("  %-28s <could not add %s>\n", label, speaker);
    return;
  }
  std::printf("  %-28s as %-9s choices from %d:", label, speaker, pos);
  int n = get_hub_choice_count(ctx, conv, pos, a);
  for (int i = 0; i < 16; ++i) {
    int link = get_hub_choice_link(ctx, conv, pos, a, i);
    if (link == -1) break;
    std::printf(" [%d]->%d", i, link);
  }
  if (n == 0) std::printf(" (none)");
  std::printf("\n");
  remove_actor(ctx, conv, a);
}

int main() {
  ctx = init_louder();
  std::setvbuf(stdout, nullptr, _IONBF, 0);

  std::string cfg = ReadFile("speakers.json");
  load_conversation_config(ctx, const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(cfg.data())), cfg.size(), 0);
  std::string doc = ReadFile("jail.json");
  LouderName conv = Declare("jail");
  load_conversation_json(ctx, conv, reinterpret_cast<uint8_t*>(&doc[0]), doc.size(), 0);

  std::printf("hub 26: speaker=%s recipients=", Name(get_hub_speaker(ctx, conv, 26)).c_str());
  for (LouderName r = next_hub_recipient(ctx, conv, 26, LOUDER_NAME_NONE); r != LOUDER_NAME_NONE;
       r = next_hub_recipient(ctx, conv, 26, r)) {
    std::printf("%s ", Name(r).c_str());
  }
  std::printf("\n");
  std::printf("Benjamin is public=%d, Freya is public=%d, Volition is public=%d\n",
              (get_speaker_flags(ctx, Declare("Benjamin")).bits & SpeakerFlags_PUBLIC.bits) ? 1 : 0,
              (get_speaker_flags(ctx, Declare("Freya")).bits & SpeakerFlags_PUBLIC.bits) ? 1 : 0,
              (get_speaker_flags(ctx, Declare("Volition")).bits & SpeakerFlags_PUBLIC.bits) ? 1 : 0);

  std::printf("\n=== 1. Can each actor walk INTO 26 from 12? ===\n");
  std::printf("  (12 has one choice, [0]->26. If the engine offers it, 26 appears.)\n");
  probe_at(conv, 12, "Benjamin", "from 12");
  probe_at(conv, 12, "Freya", "from 12");
  probe_at(conv, 13, "Benjamin", "from 13");
  probe_at(conv, 13, "Freya", "from 13");

  std::printf("\n=== 2. Real walk: Benjamin at 12 takes the choice into 26 ===\n");
  {
    LouderName b = Declare("Benjamin");
    LouderName f = Declare("Freya");
    LouderActor ben = add_actor(ctx, conv, 12, ActorFlags(), &b, 1, nullptr, 0);
    LouderActor freya = add_actor(ctx, conv, 12, ActorFlags(), &f, 1, nullptr, 0);
    flush_events(ctx);

    int r = select_choice(ctx, conv, ben, 0);
    std::printf("  select_choice(benjamin, 0) -> %d\n", r);
    std::printf("  benjamin pos=%d   freya pos=%d\n",
                get_actor_position(ctx, conv, ben), get_actor_position(ctx, conv, freya));
    std::printf("  benjamin was dragged onto the private hub: %s\n",
                get_actor_position(ctx, conv, ben) == 26 ? "YES" : "no");
    std::printf("  freya was spared (26 is not hers):        %s\n",
                get_actor_position(ctx, conv, freya) == 26 ? "no, she was dragged too" : "YES");

    std::printf("  benjamin's choices at 26: count=%d", get_hub_choice_count(ctx, conv, 26, ben));
    for (int i = 0; i < 16; ++i) {
      int link = get_hub_choice_link(ctx, conv, 26, ben, i);
      if (link == -1) break;
      std::printf(" [%d]->%d", i, link);
    }
    std::printf("\n");
  }

  std::printf("\n=== 3. Can Freya ever stand on 26? ===\n");
  std::printf("  choices offered *from* 26 (only reachable if she can stand there):\n");
  probe_at(conv, 26, "Freya", "from 26");
  probe_at(conv, 26, "Benjamin", "from 26");

  std::printf("\n  her route would have to pass through 12 or 13, both of which\n"
              "  offer her nothing, so hub 26 is unreachable for her in practice.\n");

  std::printf("\n=== 4. Is hub 34 reachable for Freya? ===\n");
  std::printf("  34's only incoming choice is [4] out of 26.\n");
  std::printf("  34 appears to Freya? her choices AT 34: ");
  probe_at(conv, 34, "Freya", "at 34");
  return 0;
}