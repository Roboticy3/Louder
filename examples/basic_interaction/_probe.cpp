// Trace what the engine does on the demo conversation, so a change to it can be
// checked against the interface tests without running them.
//
// This is a scratch tool, not part of the build. It answers the same questions
// interface_test.cpp asks, one at a time, and prints the answer rather than
// asserting it.
//
//   probe start          what the start hub looks like
//   probe at <pos>       what each actor sees at that hub
//   probe press <keys>   replay a key sequence from the start hub
//   probe hubs           the whole conversation as a graph

#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>
#include <vector>

#include "dialogue.h"

namespace {

std::string read_file(const std::string& path) {
  std::ifstream file(path, std::ios::binary);
  if (!file) {
    return "";
  }
  std::ostringstream contents;
  contents << file.rdbuf();
  return contents.str();
}

std::string short_text(const std::string& text) {
  std::string out;
  for (char c : text) {
    if (c == '\n' || c == '\r') {
      out += ' ';
    } else {
      out += c;
    }
  }
  if (out.size() > 34) {
    out = out.substr(0, 33) + "~";
  }
  return out;
}

// What one actor sees at one hub, which is the whole question the interface
// asks of the engine.
void describe(const char* who, LouderContext* ctx, LouderName conversation,
              LouderActor actor, LouderPosition position) {
  const int32_t count = get_hub_choice_count(ctx, conversation, position, actor);
  std::printf("  %-10s at %-3d  %-12s %-35s  %d choice%s", who, position,
              text_of(ctx, get_hub_speaker(ctx, conversation, position)).c_str(),
              short_text(text_of(ctx, get_hub_text(ctx, conversation, position))).c_str(), count,
              count == 1 ? "" : "s");
  if (count == 1) {
    std::printf("  (CONTINUE)");
  }
  std::printf("\n");
  for (int32_t index = 0; index < 32; ++index) {
    LouderPosition link = get_hub_choice_link(ctx, conversation, position, actor, index);
    if (link < 0) {
      continue;
    }
    std::printf("               [%2d] -> %-3d %s\n", index, link,
                text_of(ctx, get_hub_choice_text(ctx, conversation, position, actor, index))
                    .c_str());
  }
}

struct Session {
  LouderContext* ctx = init_louder();
  LouderName conversation = LOUDER_NAME_NONE;
  View view;
  LouderPosition start = -1;

  explicit Session(LouderPosition at = -1) {
    std::string config = read_file("speakers.json");
    std::string document = read_file("jail.json");
    auto mutable_bytes = [](std::string& text) {
      return reinterpret_cast<uint8_t*>(const_cast<char*>(text.data()));
    };
    load_conversation_config(ctx, mutable_bytes(config), config.size(), LOUDER_ENCODING_UTF8);
    conversation = intern(ctx, "jail");
    load_conversation_json(ctx, conversation, mutable_bytes(document), document.size(),
                           LOUDER_ENCODING_UTF8);

    LouderName tag = intern(ctx, "start");
    start = next_hub_from_tags(ctx, conversation, &tag, 1, 0, -1);

    LouderName freya = intern(ctx, "Freya");
    LouderName benjamin = intern(ctx, "Benjamin");
    view.setup(ctx, conversation, add_actor(ctx, conversation, at < 0 ? start : at, ActorFlags(),
                                           &freya, 1, nullptr, 0),
               add_actor(ctx, conversation, at < 0 ? start : at, ActorFlags(), &benjamin, 1, nullptr,
                         0));
    flush_events(ctx);
  }

  ~Session() { deinit_louder(ctx); }
};

}  // namespace

int main(int argc, char** argv) {
  const std::string command = argc > 1 ? argv[1] : "start";

  if (command == "start") {
    Session s;
    std::printf("start hub is %d\n", s.start);
    describe("Freya", s.ctx, s.conversation, s.view.left.actor, s.view.left.position);
    describe("REDACTED", s.ctx, s.conversation, s.view.right.actor, s.view.right.position);
    return 0;
  }

  if (command == "at") {
    if (argc < 3) {
      std::printf("usage: probe at <position>\n");
      return 1;
    }
    Session s(std::atoi(argv[2]));
    std::printf("placed both actors at %s\n", argv[2]);
    describe("Freya", s.ctx, s.conversation, s.view.left.actor, s.view.left.position);
    describe("REDACTED", s.ctx, s.conversation, s.view.right.actor, s.view.right.position);
    return 0;
  }

  if (command == "press") {
    Session s;
    std::printf("start hub is %d\n", s.start);
    for (int i = 2; i < argc; ++i) {
      std::string key = argv[i];
      // A key may be written as a name so Tab and backtick can be given.
      char c = key == "tab" ? '\t' : (key == "backtick" ? '`' : key[0]);
      const bool acted = s.view.press(c);
      std::printf("\npress '%s' -> %s\n", key.c_str(), acted ? "acted on" : "ignored");
      describe("Freya", s.ctx, s.conversation, s.view.left.actor, s.view.left.position);
      describe("REDACTED", s.ctx, s.conversation, s.view.right.actor, s.view.right.position);
      if (!s.view.left.error.empty()) {
        std::printf("  Freya error:    %s\n", s.view.left.error.c_str());
      }
      if (!s.view.right.error.empty()) {
        std::printf("  REDACTED error: %s\n", s.view.right.error.c_str());
      }
    }
    return 0;
  }

  if (command == "hubs") {
    Session s;
    LouderPosition position = next_hub_from_tags(s.ctx, s.conversation, &s.view.left.position, 1,
                                                 0, -1);
    (void)position;
    // Walk every hub by probing the flag iterator, which needs a flag set. The
    // simplest complete walk is over every position the engine will answer for,
    // which the hub-from-tags iterator cannot give, so use the flag iterator with
    // an empty include: every hub has no flags set, so nothing matches. Instead
    // walk from 0 upwards and let the engine refuse the gaps.
    for (LouderPosition p = 0; p < 200; ++p) {
      if (get_hub_text(s.ctx, s.conversation, p) == LOUDER_NAME_NONE &&
          get_hub_speaker(s.ctx, s.conversation, p) == LOUDER_NAME_NONE) {
        continue;
      }
      std::printf("%3d %-12s %s\n", p,
                  text_of(s.ctx, get_hub_speaker(s.ctx, s.conversation, p)).c_str(),
                  short_text(text_of(s.ctx, get_hub_text(s.ctx, s.conversation, p))).c_str());
    }
    return 0;
  }

  std::printf("usage: probe start | at <pos> | press <keys...> | hubs\n");
  return 1;
}