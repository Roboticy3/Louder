// Bare Minimum: two perspectives on one conversation, side by side in a single
// terminal.
//
// Freya is on the left and chooses with 1..9. Benjamin is on the right and
// chooses with q..p. Both are driven from one engine context holding one
// conversation and two actors, which is the arrangement the engine guide
// describes for synchronous dialogue.
//
// The whole demo is the handful of engine calls below, used as the C ABI hands
// them over. dialogue.h and dialogue.cpp add only what the engine does not do:
// which key belongs to which column, and how the two views are laid out.
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>

// louder_rust.h has no include guard of its own, so it is reached through
// dialogue.h rather than included a second time here.
#include "dialogue.h"
#include "terminal.h"

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

const char* load_error_text(int error) {
  switch (error) {
    case LOUDER_LOAD_MALFORMED_JSON:
      return "the document is not valid JSON";
    case LOUDER_LOAD_NOT_A_LIST:
      return "the document is not a list of records";
    case LOUDER_LOAD_NOT_AN_OBJECT:
      return "a record is not an object";
    case LOUDER_LOAD_ALREADY_LOADED:
      return "that conversation name is already loaded";
    case LOUDER_LOAD_BAD_ENCODING:
      return "unsupported encoding";
    default:
      return "unknown error";
  }
}

// Add an actor speaking as `speaker`, standing at `start`. Each speaks as one
// character and, through the config's listening lists, hears that character's
// internal voices.
LouderActor add_speaker(LouderContext* ctx, LouderName conversation, LouderPosition start,
                        const char* speaker) {
  LouderName name = intern(ctx, speaker);
  return add_actor(ctx, conversation, start, ActorFlags(), &name, 1, nullptr, 0);
}

}  // namespace

int main(int argc, char** argv) {
  // Paths default to the copies beside the executable and can be overridden for
  // testing against other documents. Either way they are handed to the engine as
  // filesystem paths rather than as anything Godot-specific.
  std::string conversation_path = "jail.json";
  std::string config_path = "speakers.json";

  // --check sets everything up, draws one frame and exits, so the whole startup
  // path can be exercised without a human at the keyboard.
  bool check_only = false;
  int first_positional = 1;
  if (argc > 1 && std::string(argv[1]) == "--check") {
    check_only = true;
    first_positional = 2;
  }
  if (argc > first_positional) {
    conversation_path = argv[first_positional];
  }
  if (argc > first_positional + 1) {
    config_path = argv[first_positional + 1];
  }

  LouderContext* ctx = init_louder();
  if (ctx == nullptr) {
    std::fprintf(stderr, "could not initialise the engine\n");
    return 1;
  }

  std::string config = read_file(config_path);
  if (config.empty()) {
    std::fprintf(stderr, "could not read the speaker config at %s\n", config_path.c_str());
    return 1;
  }
  int config_error = load_conversation_config(
      ctx, reinterpret_cast<uint8_t*>(const_cast<char*>(config.data())), config.size(),
      LOUDER_ENCODING_UTF8);
  if (config_error != LOUDER_LOAD_OK) {
    std::fprintf(stderr, "speaker config %s failed to load: %s\n", config_path.c_str(),
                 load_error_text(config_error));
    return 1;
  }

  std::string document = read_file(conversation_path);
  if (document.empty()) {
    std::fprintf(stderr, "could not read the conversation at %s\n", conversation_path.c_str());
    return 1;
  }

  // The conversation is held under a name, so every later query passes it back.
  LouderName conversation = intern(ctx, "jail");
  int conversation_error = load_conversation_json(
      ctx, conversation, reinterpret_cast<uint8_t*>(const_cast<char*>(document.data())),
      document.size(), LOUDER_ENCODING_UTF8);
  if (conversation_error != LOUDER_LOAD_OK) {
    std::fprintf(stderr, "conversation %s failed to load: %s\n", conversation_path.c_str(),
                 load_error_text(conversation_error));
    return 1;
  }

  // Start the conversation at the hub tagged "start", rather than assuming the
  // document numbers its hubs from zero.
  LouderName start_tag = intern(ctx, "start");
  LouderPosition start = next_hub_from_tags(ctx, conversation, &start_tag, 1, 0, -1);
  if (start < 0) {
    std::fprintf(stderr, "no hub in %s is tagged \"start\"\n", conversation_path.c_str());
    return 1;
  }

  View view;
  view.setup(ctx, conversation, add_speaker(ctx, conversation, start, "Freya"),
             add_speaker(ctx, conversation, start, "Benjamin"));
  if (view.left.actor == LOUDER_ACTOR_NONE || view.right.actor == LOUDER_ACTOR_NONE) {
    std::fprintf(stderr, "could not add both actors to the conversation\n");
    return 1;
  }

  // Discard the actor-added events from setup; the initial state comes from
  // asking the engine where everyone is rather than from replaying events.
  flush_events(ctx);

  Terminal terminal;
  view.draw(terminal);

  if (check_only) {
    std::printf("\nleft  : %s at %d, %zu choices\n", view.left.display_name, view.left.position,
                view.left.choices.size());
    std::printf("right : %s at %d, %zu choices\n", view.right.display_name, view.right.position,
                view.right.choices.size());
    return 0;
  }

  // One blocking keypress per frame. ESC quits; every other key is offered to
  // the view, which ignores anything that is not one of its choice keys.
  while (true) {
    char key = terminal.read_key();
    if (key == 27) {  // ESC
      break;
    }
    view.press(key);
    view.draw(terminal);
  }

  deinit_louder(ctx);
  return 0;
}
