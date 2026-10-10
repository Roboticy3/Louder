// Throwaway: does the engine agree with itself about hub 3 and hub 21?
//
// Two questions, answered separately so the answers cannot contaminate each
// other:
//
//  1. A static query: an actor placed at a position, asking what choices it has.
//  2. A dynamic one: actors walked forward by select_choice, asking the same.
//
// If they disagree, the engine's state after a move differs from a freshly
// placed actor at the same position, which would be an engine bug. If they
// agree, the demo's reading of the state is at fault.
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

static void report(LouderName conv, LouderPosition pos, const char* label) {
  std::printf("  %s (hub %d):\n", label, pos);
  for (const char* who : {"Freya", "Benjamin"}) {
    LouderName sp = Declare(who);
    LouderActor a = add_actor(ctx, conv, pos, ActorFlags(), &sp, 1, nullptr, 0);
    std::printf("    %-10s count=%d", who, get_hub_choice_count(ctx, conv, pos, a));
    for (int i = 0; i < 6; ++i) {
      int link = get_hub_choice_link(ctx, conv, pos, a, i);
      if (link == -1) break;
      std::printf("  [%d]->%d", i, link);
    }
    std::printf("\n");
    remove_actor(ctx, conv, a);
  }
}

int main() {
  ctx = init_louder();
  std::setvbuf(stdout, nullptr, _IONBF, 0);

  std::string cfg = ReadFile("speakers.json");
  load_conversation_config(ctx, const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(cfg.data())), cfg.size(), 0);
  std::string doc = ReadFile("jail.json");
  LouderName conv = Declare("jail");
  load_conversation_json(ctx, conv, reinterpret_cast<uint8_t*>(&doc[0]), doc.size(), 0);

  std::printf("=== static queries ===\n");
  report(conv, 3, "hub 3");
  report(conv, 21, "hub 21");

  // Now walk there for real.
  std::printf("\n=== after walking 0 -> 3 with 'q' ===\n");
  LouderName f = Declare("Freya");
  LouderName b = Declare("Benjamin");
  LouderName fs[1] = {f};
  LouderName bs[1] = {b};
  LouderActor freya = add_actor(ctx, conv, 0, ActorFlags(), fs, 1, nullptr, 0);
  LouderActor ben = add_actor(ctx, conv, 0, ActorFlags(), bs, 1, nullptr, 0);
  flush_events(ctx);

  int r = select_choice(ctx, conv, ben, 0);
  std::printf("select_choice -> %d\n", r);
  std::printf("freya pos=%d  benjamin pos=%d\n",
              get_actor_position(ctx, conv, freya), get_actor_position(ctx, conv, ben));
  std::printf("  freya count at its own position = %d\n",
              get_hub_choice_count(ctx, conv, get_actor_position(ctx, conv, freya), freya));
  std::printf("  benjamin count at its own position = %d\n",
              get_hub_choice_count(ctx, conv, get_actor_position(ctx, conv, ben), ben));

  std::printf("\n=== can Freya take the choice from 3? ===\n");
  std::printf("  freya count at 3 = %d\n", get_hub_choice_count(ctx, conv, 3, freya));
  int rr = select_choice(ctx, conv, freya, 0);
  std::printf("  select_choice(freya, 0) -> %d\n", rr);
  std::printf("  freya pos=%d benjamin pos=%d\n",
              get_actor_position(ctx, conv, freya), get_actor_position(ctx, conv, ben));

  // And a freshly built context, for comparison.
  std::printf("\n=== same question, fresh actor in the same context ===\n");
  report(conv, 3, "hub 3 again");
  return 0;
}