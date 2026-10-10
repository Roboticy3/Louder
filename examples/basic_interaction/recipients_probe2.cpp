// Throwaway: why did Benjamin's walk into hub 26 fail, and does hub 26 actually
// appear to him?
//
// First probe found: a lone Benjamin at 12 is offered [0]->26, but with a Freya
// actor also present, select_choice returns 1 (refused). The two differ in one
// way that matters to the spoke rule: whether anyone is speaking as Freya, the
// speaker of hub 26.
//
// This isolates that, and separately checks whether 26 appears to Benjamin at
// all, by asking a question that does not go through choice_available.
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

static void load() {
  std::string cfg = ReadFile("speakers.json");
  load_conversation_config(ctx, const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(cfg.data())), cfg.size(), 0);
  std::string doc = ReadFile("jail.json");
  LouderName conv = Declare("jail");
  load_conversation_json(ctx, conv, reinterpret_cast<uint8_t*>(&doc[0]), doc.size(), 0);
}

static void choices_at(LouderName conv, LouderPosition pos, const char* who) {
  LouderName sp = Declare(who);
  LouderActor a = add_actor(ctx, conv, pos, ActorFlags(), &sp, 1, nullptr, 0);
  std::printf("      %-9s from %2d:", who, pos);
  int n = get_hub_choice_count(ctx, conv, pos, a);
  for (int i = 0; i < 16; ++i) {
    int link = get_hub_choice_link(ctx, conv, pos, a, i);
    if (link == -1) break;
    std::printf(" [%d]->%d", i, link);
  }
  if (n == 0) std::printf(" (none, count=%d)", n);
  std::printf("\n");
  remove_actor(ctx, conv, a);
}

// Does hub 26 appear to this actor? Ask through a hub whose only choice leads
// to 26, and see whether that choice is offered. hub 12 and 13 both do.
static void can_reach_26(const char* who) {
  LouderName conv = Declare("jail");
  LouderName sp = Declare(who);
  LouderActor a = add_actor(ctx, conv, 12, ActorFlags(), &sp, 1, nullptr, 0);
  int n = get_hub_choice_count(ctx, conv, 12, a);
  int link0 = get_hub_choice_link(ctx, conv, 12, a, 0);
  std::printf("      %-9s offered at 12: count=%d [0]->%d  => %s\n", who, n, link0,
              link0 == 26 ? "26 APPEARS to them" : "26 does NOT appear to them");
  remove_actor(ctx, Declare("jail"), a);
}

int main() {
  ctx = init_louder();
  std::setvbuf(stdout, nullptr, _IONBF, 0);
  load();
  LouderName conv = Declare("jail");

  std::printf("=== A. Benjamin alone: does 26 appear to him? ===\n");
  can_reach_26("Benjamin");

  std::printf("\n=== B. Now add a Freya actor. Does 26 still appear? ===\n");
  {
    LouderName f = Declare("Freya");
    add_actor(ctx, conv, 12, ActorFlags(), &f, 1, nullptr, 0);
    can_reach_26("Benjamin");

    std::printf("\n      hub 26's speaker is Freya, and someone now speaks as Freya.\n");
    std::printf("      next_actor_speaking_as(Freya) = %d (>=0 means the speaker is taken)\n",
                next_actor_speaking_as(ctx, conv, f, LOUDER_ACTOR_NONE));
  }

  std::printf("\n=== C. Choices out of 26, with and without a Freya actor ===\n");
  {
    choices_at(conv, 26, "Benjamin");
    std::printf("      -- with a Freya actor present --\n");
    choices_at(conv, 26, "Benjamin");
    choices_at(conv, 26, "Freya");
  }

  std::printf("\n=== D. And hub 34, whose only way in is [4] out of 26 ===\n");
  {
    LouderName conv = Declare("jail");
    LouderName f = Declare("Freya");
    LouderName b = Declare("Benjamin");
    // Both actors standing at 26, so index 4 is offered to whoever may take it.
    LouderActor ben = add_actor(ctx, conv, 26, ActorFlags(), &b, 1, nullptr, 0);
    add_actor(ctx, conv, 26, ActorFlags(), &f, 1, nullptr, 0);
    std::printf("      benjamin at 26, count=%d  (index 4 is the route to 34)\n",
              get_hub_choice_count(ctx, conv, 26, ben));
    for (int i = 0; i < 16; ++i) {
      int link = get_hub_choice_link(ctx, conv, 26, ben, i);
      if (link == -1) break;
      std::printf("        [%d]->%d '%s'\n", i, link,
                  Name(get_hub_choice_text(ctx, conv, 26, ben, i)).substr(0, 40).c_str());
    }
  }
  return 0;
}