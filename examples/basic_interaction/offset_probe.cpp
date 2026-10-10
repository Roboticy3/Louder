// Throwaway: walk the jail conversation and report every LouderPosition where the
// visible choice indices differ from the engine's general indices. Those are
// the cases where a naive UI that passes its display number straight to
// select_choice would pick the wrong branch.
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
  std::ostringstream ss; ss << f.rdbuf(); return ss.str();
}

int main() {
  ctx = init_louder();
  std::setvbuf(stdout, nullptr, _IONBF, 0);

  load_conversation_config(ctx, const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(ReadFile("speakers.json").data())),
                           ReadFile("speakers.json").size(), 0);
  std::string doc = ReadFile("jail.json");
  LouderName conv = Declare("jail");
  load_conversation_json(ctx, conv, reinterpret_cast<uint8_t*>(&doc[0]), doc.size(), 0);

  LouderName startTag = Declare("start");
  LouderPosition start = next_hub_from_tags(ctx, conv, &startTag, 1, 0, -1);

  LouderName freyaS = Declare("Freya");
  LouderName benjS = Declare("Benjamin");

  // Enumerate, for each actor and each hub, the raw choice list and the
  // available one. A spectator+debug actor sees every raw choice.
  LouderActor dbg = add_actor(ctx, conv, start, ActorFlags_SPECTATOR | ActorFlags_DEBUG, nullptr, 0, nullptr, 0);

  // Hub 26 reported a nonzero choice count but an empty walk, which means the
  // available indices are not contiguous from zero. Probe every index.
  std::printf("\n--- hub 26, every index 0..7, per actor ---\n");
  for (const char* who : {"Freya", "Benjamin"}) {
    LouderName sp = Declare(who);
    LouderActor a = add_actor(ctx, conv, 26, ActorFlags(), &sp, 1, nullptr, 0);
    std::printf("%-10s count=%d", who, get_hub_choice_count(ctx, conv, 26, a));
    for (int i = 0; i < 8; ++i) {
      int link = get_hub_choice_link(ctx, conv, 26, a, i);
      LouderName text = get_hub_choice_text(ctx, conv, 26, a, i);
      std::printf("  [%d] link=%d text=%s", i, link,
                  text == LOUDER_NAME_NONE ? "<none>" : Name(text).substr(0, 18).c_str());
    }
    std::printf("\n");
    remove_actor(ctx, conv, a);
  }

  LouderName mainCh = Declare("main");
  std::printf("%-4s %-10s | %-28s | %-28s\n", "hub", "speaker", "raw choices (idx->pos)", "per-actor visible");
  for (LouderPosition p = next_hub_by_channel(ctx, conv, mainCh, -1); p != -1; p = next_hub_by_channel(ctx, conv, mainCh, p)) {
    std::string raw;
    for (int i = 0; i < 16; ++i) {
      int link = get_hub_choice_link(ctx, conv, p, dbg, i);
      if (link == -1) break;
      if (!raw.empty()) raw += " ";
      raw += std::to_string(i) + "->" + std::to_string(link);
    }

    // Fresh engines would be needed to test each actor per hub in isolation;
    // instead just ask with a temporarily added actor at that LouderPosition.
    auto visible_for = [&](const char* speaker) {
      LouderName sp = Declare(speaker);
      LouderActor a = add_actor(ctx, conv, p, ActorFlags(), &sp, 1, nullptr, 0);
      if (a == LOUDER_ACTOR_NONE) return std::string("<actor refused>");
      int n = get_hub_choice_count(ctx, conv, p, a);
      std::string out;
      for (int i = 0; i < 16; ++i) {
        int link = get_hub_choice_link(ctx, conv, p, a, i);
        if (link == -1) break;
        if (!out.empty()) out += " ";
        out += std::to_string(i) + "->" + std::to_string(link);
      }
      remove_actor(ctx, conv, a);
      std::string res = out.empty() ? ("(none, count=" + std::to_string(n) + ")") : out;
      return res;
    };

    std::printf("%-4d %-10s | %-28s | F:%-24s B:%s\n", p,
                Name(get_hub_speaker(ctx, conv, p)).substr(0, 10).c_str(),
                raw.empty() ? "-" : raw.c_str(),
                visible_for("Freya").c_str(),
                visible_for("Benjamin").c_str());
  }
  return 0;
}
