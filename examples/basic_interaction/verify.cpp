// Throwaway, written carefully because the previous probe reported two things
// that cannot be true: hub 7's speaker as "2Freya", and a choice on hub 20,
// which the document says has none.
//
// This adds exactly two actors, never adds or removes an actor while walking,
// and re-reads the document's own facts before trusting anything else.
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
  size_t n = std::strlen(s);
  LouderName e = get_named_id(ctx, reinterpret_cast<const uint8_t*>(s), n);
  if (e != LOUDER_NAME_NONE) return e;
  return declare_name(ctx, reinterpret_cast<const uint8_t*>(s), n);
}

static std::string ReadFile(const char* path) {
  std::ifstream f(path, std::ios::binary);
  if (!f) return "";
  std::ostringstream ss;
  ss << f.rdbuf();
  return ss.str();
}

int main() {
  ctx = init_louder();
  std::setvbuf(stdout, nullptr, _IONBF, 0);

  std::string cfg = ReadFile("speakers.json");
  load_conversation_config(ctx, const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(cfg.data())), cfg.size(), 0);
  std::string doc = ReadFile("jail.json");
  LouderName conv = Declare("jail");
  load_conversation_json(ctx, conv, reinterpret_cast<uint8_t*>(&doc[0]), doc.size(), 0);

  // Facts straight from the document, before any actor exists.
  std::printf("--- facts, no actors ---\n");
  std::printf("  hub  7 speaker = '%s'\n", Name(get_hub_speaker(ctx, conv, 7)).c_str());
  std::printf("  hub 20 speaker = '%s'\n", Name(get_hub_speaker(ctx, conv, 20)).c_str());
  std::printf("  hub 27 flags   = %d (auto = %d)\n", get_hub_flags(ctx, conv, 27).bits,
              (get_hub_flags(ctx, conv, 27).bits & HubFlags_AUTO.bits) ? 1 : 0);
  std::printf("  hub 35 flags   = %d (auto = %d)\n", get_hub_flags(ctx, conv, 35).bits,
              (get_hub_flags(ctx, conv, 35).bits & HubFlags_AUTO.bits) ? 1 : 0);

  // Exactly two actors, standing at the start, and never touched again.
  LouderName start_tag = Declare("start");
  LouderPosition start = next_hub_from_tags(ctx, conv, &start_tag, 1, 0, -1);
  LouderName fs[1] = {Declare("Freya")};
  LouderName bs[1] = {Declare("Benjamin")};
  LouderActor freya = add_actor(ctx, conv, start, ActorFlags(), fs, 1, nullptr, 0);
  LouderActor ben = add_actor(ctx, conv, start, ActorFlags(), bs, 1, nullptr, 0);
  std::printf("\n  start=%d freya=%d benjamin=%d\n", start, freya, ben);

  // A spectator+debug actor is the only way to see every raw choice. It sees
  // everything but takes no part, so it cannot perturb the two players.
  LouderActor dbg = add_actor(ctx, conv, start, ActorFlags_SPECTATOR | ActorFlags_DEBUG,
                              nullptr, 0, nullptr, 0);
  std::printf("  debug=%d\n", dbg);

  std::printf("\n--- hub 7 and 20, read again with two actors present ---\n");
  std::printf("  hub  7 speaker = '%s'\n", Name(get_hub_speaker(ctx, conv, 7)).c_str());
  std::printf("  hub 20 speaker = '%s'\n", Name(get_hub_speaker(ctx, conv, 20)).c_str());

  std::printf("\n--- hub 26 in detail ---\n");
  std::printf("  raw (debug actor):");
  for (int i = 0; i < 8; ++i) {
    int link = get_hub_choice_link(ctx, conv, 26, dbg, i);
    if (link == -1) break;
    std::printf(" [%d]->%d", i, link);
  }
  std::printf("\n  freya   count=%d", get_hub_choice_count(ctx, conv, 26, freya));
  for (int i = 0; i < 8; ++i) {
    int link = get_hub_choice_link(ctx, conv, 26, freya, i);
    if (link == -1) break;
    std::printf(" [%d]->%d", i, link);
  }
  std::printf("\n  benjamin count=%d", get_hub_choice_count(ctx, conv, 26, ben));
  for (int i = 0; i < 8; ++i) {
    int link = get_hub_choice_link(ctx, conv, 26, ben, i);
    if (link == -1) break;
    std::printf(" [%d]->%d", i, link);
  }
  std::printf("\n");

  std::printf("\n--- is hub 27 offered as a choice target? it is AUTO ---\n");
  std::printf("  hub 27 is_auto (engine) = %d\n",
              (get_hub_flags(ctx, conv, 27).bits & HubFlags_AUTO.bits) ? 1 : 0);
  std::printf("  benjamin: count at 26 = %d, [0]->%d\n",
              get_hub_choice_count(ctx, conv, 26, ben),
              get_hub_choice_link(ctx, conv, 26, ben, 0));

  std::printf("\n--- what a DEBUG actor sees at 27 (auto hubs appear only to debug) ---\n");
  std::printf("  debug count at 27 = %d\n", get_hub_choice_count(ctx, conv, 27, dbg));
  std::printf("  benjamin count at 27 = %d\n", get_hub_choice_count(ctx, conv, 27, ben));

  std::printf("\n--- walk to 26 for real and look again ---\n");
  flush_events(ctx);
  int r = select_choice(ctx, conv, ben, 0);   // from hub 12? no: from start
  std::printf("  (from start hub %d) select_choice(benjamin,0) -> %d\n", start, r);
  std::printf("  benjamin pos=%d freya pos=%d\n",
              get_actor_position(ctx, conv, ben), get_actor_position(ctx, conv, freya));
  std::printf("  benjamin count here = %d\n",
              get_hub_choice_count(ctx, conv, get_actor_position(ctx, conv, ben), ben));
  return 0;
}