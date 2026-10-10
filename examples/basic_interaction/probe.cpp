// Throwaway diagnostic: does the engine accept the editor's speaker config
// schema, and what does the jail conversation look like once loaded?
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

static LouderName Lookup(const char* s) {
  return get_named_id(ctx, reinterpret_cast<const uint8_t*>(s), std::strlen(s));
}

// Names are interned by hash, so anything the loader already touched needs a
// lookup rather than a declare.
static LouderName Declare(const char* s) {
  LouderName existing = Lookup(s);
  return existing != LOUDER_NAME_NONE ? existing : declare_name(ctx, reinterpret_cast<const uint8_t*>(s), std::strlen(s));
}

static std::string ReadFile(const char* path, bool* ok) {
  std::ifstream f(path, std::ios::binary);
  if (!f) { *ok = false; return ""; }
  std::ostringstream ss; ss << f.rdbuf(); *ok = true; return ss.str();
}

int main(int argc, char** argv) {
  ctx = init_louder();
  if (!ctx) { std::printf("init failed\n"); return 1; }

  const char* convPath = argc > 1 ? argv[1] : "jail.json";
  const char* cfgPath = argc > 2 ? argv[2] : "speakers.json";
  const char* engineCfgPath = argc > 3 ? argv[3] : "none.json";

  bool ok = false;
  std::string cfg = ReadFile(cfgPath, &ok);
  std::printf("config %s readable=%d size=%zu\n", cfgPath, (int)ok, cfg.size());
  LoadError e = load_conversation_config(ctx,
      reinterpret_cast<uint8_t*>(&cfg[0]), cfg.size(), 0);
  std::printf("load_conversation_config(editor format) -> %d\n", e);
  std::printf("Freya id=%d  Benjamin id=%d\n", Lookup("Freya"), Lookup("Benjamin"));

  std::string ecfg = ReadFile(engineCfgPath, &ok);
  if (ok) {
    LoadError e2 = load_conversation_config(ctx,
        reinterpret_cast<uint8_t*>(&ecfg[0]), ecfg.size(), 0);
    std::printf("load_conversation_config(engine format) -> %d from %s\n", e2, engineCfgPath);
    for (const char* s : {"Freya", "Benjamin", "Note", "Structure", "Stars", "Memory", "Sight", "Volition"}) {
      LouderName n = Lookup(s);
      std::printf("  speaker %-10s id=%d public=%d\n", s, n,
                  (n != LOUDER_NAME_NONE && (get_speaker_flags(ctx, n).bits & SpeakerFlags_PUBLIC.bits)) ? 1 : 0);
    }
    std::string listen;
    for (LouderName l = get_next_speaker_listen_to(ctx, Declare("Freya"), LOUDER_NAME_NONE);
         l != LOUDER_NAME_NONE; l = get_next_speaker_listen_to(ctx, Declare("Freya"), l)) {
      listen += Name(l) + " ";
    }
    std::printf("  Freya listening: %s\n", listen.c_str());
  } else {
    std::printf("no engine-format config at %s\n", engineCfgPath);
  }

  std::string doc = ReadFile(convPath, &ok);
  std::printf("conversation %s readable=%d size=%zu\n", convPath, (int)ok, doc.size());
  LouderName conv = Declare("jail");
  LoadError ce = load_conversation_json(ctx, conv, reinterpret_cast<uint8_t*>(&doc[0]), doc.size(), 0);
  std::printf("load_conversation_json -> %d\n", ce);

  LouderName start = Declare("start");
  LouderPosition sp = next_hub_from_tags(ctx, conv, &start, 1, 0, -1);
  std::printf("start hub = %d\n", sp);

  LouderName mainCh = Declare("main");
  std::printf("\n--- hubs ---\n");
  int hubCount = 0;
  for (LouderPosition p = next_hub_by_channel(ctx, conv, mainCh, -1); p != -1; p = next_hub_by_channel(ctx, conv, mainCh, p)) {
    HubFlags fl = get_hub_flags(ctx, conv, p);
    std::printf("hub %2d speaker=%-10s auto=%d ask=%d prio=%d text=%.44s\n", p,
        Name(get_hub_speaker(ctx, conv, p)).c_str(), (fl.bits & HubFlags_AUTO.bits) ? 1 : 0,
        (fl.bits & HubFlags_ASK.bits) ? 1 : 0, get_hub_priority(ctx, conv, p),
        Name(get_hub_text(ctx, conv, p)).c_str());
    std::string recips;
    for (LouderName r = next_hub_recipient(ctx, conv, p, LOUDER_NAME_NONE); r != LOUDER_NAME_NONE; r = next_hub_recipient(ctx, conv, p, r)) {
      recips += Name(r) + " ";
    }
    if (!recips.empty()) std::printf("        recipients: %s\n", recips.c_str());
    std::string tags;
    for (LouderName t = next_hub_tag(ctx, conv, p, LOUDER_NAME_NONE); t != LOUDER_NAME_NONE; t = next_hub_tag(ctx, conv, p, t)) {
      tags += Name(t) + " ";
    }
    if (!tags.empty()) std::printf("        tags: %s\n", tags.c_str());
    hubCount++;
  }
  std::printf("total hubs: %d\n", hubCount);

  flush_events(ctx);
  LouderActor dbg = add_actor(ctx, conv, sp, ActorFlags_SPECTATOR | ActorFlags_DEBUG, nullptr, 0, nullptr, 0);
  std::printf("\n--- raw choices (debug actor) ---\n");
  for (LouderPosition p = next_hub_by_channel(ctx, conv, mainCh, -1); p != -1; p = next_hub_by_channel(ctx, conv, mainCh, p)) {
    std::printf("hub %2d:", p);
    for (int i = 0; i < 12; ++i) {
      int link = get_hub_choice_link(ctx, conv, p, dbg, i);
      if (link == -1) break;
      std::printf(" [%d]->%d", i, link);
    }
    std::printf("\n");
  }

  std::printf("\n--- actors at start %d ---\n", sp);
  LouderName freya = Declare("Freya");
  LouderName benj = Declare("Benjamin");
  LouderName fSpeak[1] = {freya};
  LouderName bSpeak[1] = {benj};
  LouderActor a0 = add_actor(ctx, conv, sp, ActorFlags(), fSpeak, 1, nullptr, 0);
  LouderActor a1 = add_actor(ctx, conv, sp, ActorFlags(), bSpeak, 1, nullptr, 0);
  std::printf("actor freya=%d benjamin=%d\n", a0, a1);
  std::printf("freya choices=%d benjamin choices=%d\n",
      get_hub_choice_count(ctx, conv, sp, a0), get_hub_choice_count(ctx, conv, sp, a1));
  for (int i = 0; i < 8; ++i) {
    int f = get_hub_choice_link(ctx, conv, sp, a0, i);
    int b = get_hub_choice_link(ctx, conv, sp, a1, i);
    if (f == -1 && b == -1) break;
    std::printf("  idx %d: freya->%d '%s' | benjamin->%d '%s'\n", i, f,
        Name(get_hub_choice_text(ctx, conv, sp, a0, i)).c_str(), b,
        Name(get_hub_choice_text(ctx, conv, sp, a1, i)).c_str());
  }

  std::printf("\n--- events ---\n");
  for (size_t i = 0;; ++i) {
    LouderOption<LouderEvent> o = get_event(ctx, i);
    if (o.tag == LouderOption<LouderEvent>::Tag::None) break;
    LouderEvent ev = o.some._0;
    std::printf("[%zu] actor=%d channel=%s tag=%d", i, ev.actor, Name(ev.channel).c_str(), (int)ev.data.tag);
    if (ev.data.tag == EventData::Tag::RegularMotion || ev.data.tag == EventData::Tag::Warp || ev.data.tag == EventData::Tag::ForceMotion) {
      const Motion& m = ev.data.tag == EventData::Tag::RegularMotion ? ev.data.regular_motion._0
        : ev.data.tag == EventData::Tag::Warp ? ev.data.warp._0 : ev.data.force_motion._0;
      std::printf(" motion %d->%d ci=%d bg=%d", m.source_position, m.target_position, m.choice_index, m.background ? 1 : 0);
    } else if (ev.data.tag == EventData::Tag::ChoiceAppearance) {
      std::printf(" choice pos=%d idx=%d", ev.data.choice_appearance._0.position, ev.data.choice_appearance._0.choice_index);
    } else if (ev.data.tag == EventData::Tag::Callback) {
      std::printf(" callback=%s", Name(ev.data.callback._0).c_str());
    } else if (ev.data.tag == EventData::Tag::AdditionalDataChange) {
      std::printf(" addl=%s", Name(ev.data.additional_data_change._0.key).c_str());
    }
    std::printf("\n");
  }

  deinit_louder(ctx);
  return 0;
}