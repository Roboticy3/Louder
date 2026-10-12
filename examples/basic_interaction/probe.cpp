// Scratch tool: ask the engine what the demo conversation does, so a change to
// it can be checked against interface_test.cpp without running the tests.
//
// The interface tests assert against hard coded hub numbers, so changing the
// conversation means working out which numbers moved before editing the tests.
// This prints the same facts the tests ask for and nothing else, so the answer
// comes before the assertion does.
//
// Not part of the build; build_and_test.ps1 does not compile this.
//
//   g++ -std=c++17 -O1 -I . probe.cpp dialogue.cpp terminal.cpp -o probe.exe louder_rust.dll
//
//   probe start            what each actor sees at the start hub
//   probe at <position>    what each actor sees when both start there
//   probe press q 1 q      replay keys from the start hub, reporting after each
//   probe hubs             every hub in the document, in position order

#include <cstdio>
#include <cstdlib>
#include <fstream>
#include <sstream>
#include <string>

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

// One line of dialogue, long enough to identify a hub by and no longer.
std::string brief(const std::string& text) {
  std::string out;
  for (char c : text) {
    out += (c == '\n' || c == '\r') ? ' ' : c;
  }
  if (out.size() > 40) {
    out = out.substr(0, 39) + "~";
  }
  return out;
}

// What one actor sees at one hub: the whole question the interface asks of the
// engine, and the whole answer the tests check.
void describe(const char* who, LouderContext* ctx, LouderName conversation, LouderActor actor,
              LouderPosition position) {
  const int32_t count = get_hub_choice_count(ctx, conversation, position, actor);
  std::printf("  %-9s at %-3d  %-10s %-41s  %d choice%s%s\n", who, position,
              text_of(ctx, get_hub_speaker(ctx, conversation, position)).c_str(),
              brief(text_of(ctx, get_hub_text(ctx, conversation, position))).c_str(), count,
              count == 1 ? "" : "s", count == 1 ? "  (CONTINUE)" : "");

  // Each choice under its own engine index, because that index is not the
  // number the interface prints. A gap in the numbers is the normal case.
  for (int32_t index = 0; index < 64; ++index) {
    const LouderPosition link = get_hub_choice_link(ctx, conversation, position, actor, index);
    if (link < 0) {
      continue;
    }
    std::printf("              [%2d] -> %-3d %s\n", index, link,
                text_of(ctx, get_hub_choice_text(ctx, conversation, position, actor, index))
                    .c_str());
  }
}

// A loaded conversation with both actors on one hub, which is the starting
// point every test in interface_test.cpp shares.
struct Session {
  LouderContext* ctx = init_louder();
  LouderName conversation = LOUDER_NAME_NONE;
  View view;
  LouderPosition start = -1;

  explicit Session(LouderPosition at = -1) {
    std::string config = read_file("speakers.json");
    std::string document = read_file("jail.json");
    // The loaders take a mutable buffer; neither document is modified.
    auto mutable_bytes = [](std::string& text) {
      return reinterpret_cast<uint8_t*>(const_cast<char*>(text.data()));
    };

    const int32_t config_result = load_conversation_config(
        ctx, mutable_bytes(config), config.size(), LOUDER_ENCODING_UTF8);
    if (config_result != LOUDER_LOAD_OK) {
      std::printf("speakers.json failed to load: %d\n", config_result);
    }

    conversation = intern(ctx, "jail");
    const int32_t document_result = load_conversation_json(
        ctx, conversation, mutable_bytes(document), document.size(), LOUDER_ENCODING_UTF8);
    if (document_result != LOUDER_LOAD_OK) {
      std::printf("jail.json failed to load: %d\n", document_result);
    }

    // A load reports records it could not read through the event queue rather
    // than by failing, so a silently short document is worth looking at here.
    for (size_t i = 0;; ++i) {
      LouderOption<LouderEvent> option = get_event(ctx, i);
      if (option.tag == LouderOption<LouderEvent>::Tag::None) {
        break;
      }
      if (option.some._0.data.tag == EventData::Tag::LoadError) {
        const LoadFailure& failure = option.some._0.data.load_error._0;
        std::printf("load error: position %d, error %d\n", failure.position, failure.error);
      }
    }
    flush_events(ctx);

    LouderName tag = intern(ctx, "start");
    start = next_hub_from_tags(ctx, conversation, &tag, 1, 0, -1);
    const LouderPosition here = at < 0 ? start : at;

    LouderName freya = intern(ctx, "Freya");
    LouderName benjamin = intern(ctx, "Benjamin");
    view.setup(ctx, conversation,
               add_actor(ctx, conversation, here, ActorFlags(), &freya, 1, nullptr, 0),
               add_actor(ctx, conversation, here, ActorFlags(), &benjamin, 1, nullptr, 0));
    flush_events(ctx);
  }

  ~Session() { deinit_louder(ctx); }
};

void report(const Session& session) {
  describe("Freya", session.ctx, session.conversation, session.view.left.actor,
           session.view.left.position);
  describe("REDACTED", session.ctx, session.conversation, session.view.right.actor,
           session.view.right.position);
  if (!session.view.left.error.empty()) {
    std::printf("  Freya error:    %s\n", session.view.left.error.c_str());
  }
  if (!session.view.right.error.empty()) {
    std::printf("  REDACTED error: %s\n", session.view.right.error.c_str());
  }
}

// The key a caller would type, given the spelling used on a command line.
char key_of(const std::string& name) {
  if (name == "tab") return '\t';
  if (name == "backtick") return '`';
  return name[0];
}

}  // namespace

int main(int argc, char** argv) {
  const std::string command = argc > 1 ? argv[1] : "start";

  if (command == "start") {
    Session session;
    std::printf("start hub is %d\n\n", session.start);
    report(session);
    return 0;
  }

  if (command == "at") {
    if (argc < 3) {
      std::printf("usage: probe at <position>\n");
      return 1;
    }
    Session session(std::atoi(argv[2]));
    std::printf("both actors placed at %s\n\n", argv[2]);
    report(session);
    return 0;
  }

  if (command == "press") {
    Session session;
    std::printf("start hub is %d\n", session.start);
    for (int i = 2; i < argc; ++i) {
      const std::string name = argv[i];
      const bool acted = session.view.press(key_of(name));
      std::printf("\npress '%s' -> %s\n", name.c_str(), acted ? "acted on" : "ignored");
      report(session);
    }
    return 0;
  }

  if (command == "hubs") {
    Session session;
    // Positions are sparse, so the walk asks the engine rather than trusting the
    // document to be dense. A hub with neither text nor speaker still counts,
    // so the flag iterator cannot be used either: every hub is matched by
    // asking for its own position back.
    std::printf("%4s  %-10s %s\n", "hub", "speaker", "text");
    for (LouderPosition position = 0; position < 256; ++position) {
      const LouderName text = get_hub_text(session.ctx, session.conversation, position);
      const LouderName speaker = get_hub_speaker(session.ctx, session.conversation, position);
      if (text == LOUDER_NAME_NONE && speaker == LOUDER_NAME_NONE) {
        continue;
      }
      std::printf("%4d  %-10s %s\n", position,
                  speaker == LOUDER_NAME_NONE ? "(none)" : text_of(session.ctx, speaker).c_str(),
                  text == LOUDER_NAME_NONE ? "(no text)" : brief(text_of(session.ctx, text)).c_str());
    }
    return 0;
  }

  std::printf("usage: probe start | at <position> | press <keys...> | hubs\n");
  return 1;
}