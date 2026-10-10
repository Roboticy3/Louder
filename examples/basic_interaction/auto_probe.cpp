// Throwaway: walk Benjamin through hub 26 -> 27 and report where the auto logic
// sends him, and what he is offered at each step.
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
  return e != LOUDER_NAME_NONE ? e : declare_name(ctx, reinterpret_cast<const uint8_t*>(s), n);
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

  LouderName ben_sp[1] = {Declare("Benjamin")};
  LouderName freya_sp[1] = {Declare("Freya")};
  LouderActor ben = add_actor(ctx, conv, 26, ActorFlags(), ben_sp, 1, nullptr, 0);
  LouderActor freya = add_actor(ctx, conv, 26, ActorFlags(), freya_sp, 1, nullptr, 0);
  flush_events(ctx);

  std::printf("hub 27 structure: speaker=%s auto=%d\n",
              Name(get_hub_speaker(ctx, conv, 27)).c_str(),
              (get_hub_flags(ctx, conv, 27).bits & HubFlags_AUTO.bits) ? 1 : 0);
  for (int i = 0;; ++i) {
    int link = get_hub_choice_link(ctx, conv, 27, ben, i);
    if (link == -1) break;
    std::printf("  benjamin at 27: [%d]->%d '%s'\n", i, link,
                Name(get_hub_text(ctx, conv, link)).substr(0, 46).c_str());
  }
  std::printf("  freya   at 27: count=%d\n", get_hub_choice_count(ctx, conv, 27, freya));

  std::printf("\nbenjamin's choices at 26: count=%d\n", get_hub_choice_count(ctx, conv, 26, ben));
  for (int i = 0;; ++i) {
    int link = get_hub_choice_link(ctx, conv, 26, ben, i);
    if (link == -1) break;
    std::printf("  [%d]->%d '%s'\n", i, link,
                Name(get_hub_text(ctx, conv, link)).substr(0, 46).c_str());
  }

  std::printf("\nselecting index 0 (the Lie, into auto hub 27)\n");
  int r = select_choice(ctx, conv, ben, 0);
  std::printf("  select_choice -> %d\n", r);
  std::printf("  benjamin lands on %d, speaker '%s'\n", get_actor_position(ctx, conv, ben),
              Name(get_hub_speaker(ctx, conv, get_actor_position(ctx, conv, ben))).c_str());
  std::printf("  expected 29 (Memory), got %d\n", get_actor_position(ctx, conv, ben));
  std::printf("  freya stays at %d\n", get_actor_position(ctx, conv, freya));

  std::printf("\nevents:\n");
  for (size_t i = 0;; ++i) {
    LouderOption<LouderEvent> o = get_event(ctx, i);
    if (o.tag == LouderOption<LouderEvent>::Tag::None) break;
    LouderEvent e = o.some._0;
    if (e.data.tag == EventData::Tag::RegularMotion || e.data.tag == EventData::Tag::Warp) {
      const Motion& m = e.data.tag == EventData::Tag::Warp ? e.data.warp._0 : e.data.regular_motion._0;
      std::printf("  actor=%d %d->%d ci=%d bg=%d\n", e.actor, m.source_position, m.target_position,
                  m.choice_index, m.background ? 1 : 0);
    }
  }
  return 0;
}