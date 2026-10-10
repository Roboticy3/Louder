// Bare Minimum: two perspectives on one conversation, side by side in a single
// terminal.
//
// Freya is on the left and chooses with 1..9. Benjamin is on the right and
// chooses with q..p. Both are driven from one engine context holding one
// conversation and two actors, which is the arrangement the engine guide
// describes for synchronous dialogue.
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>

#include "dialogue_view.h"
#include "engine_bridge.h"
#include "screen.h"

namespace {

using louder_demo::ActorId;
using louder_demo::DialogueView;
using louder_demo::Engine;
using louder_demo::Position;
using louder_demo::Viewport;

// Benjamin is never named to either party. The demo aliases him the way the
// story does, so the interface has to carry the alias separately from the
// speaker name the engine reports.
constexpr char kBenjaminAlias[] = "REDACTED";

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

}  // namespace

int main(int argc, char** argv) {
  // Paths default to the copies beside the executable and can be overridden for
  // testing against other documents. Either way they are handed to the engine as
  // absolute-ish filesystem paths rather than as anything Godot-specific.
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

  Engine engine;
  if (!engine.ready()) {
    std::fprintf(stderr, "could not initialise the engine\n");
    return 1;
  }

  std::string config = read_file(config_path);
  if (config.empty()) {
    std::fprintf(stderr, "could not read the speaker config at %s\n", config_path.c_str());
    return 1;
  }
  int config_error = engine.load_config(config);
  if (config_error != LOUDER_LOAD_OK) {
    std::fprintf(stderr, "speaker config %s failed to load: %s\n", config_path.c_str(),
                 load_error_text(config_error));
    return 1;
  }

  std::string conversation = read_file(conversation_path);
  if (conversation.empty()) {
    std::fprintf(stderr, "could not read the conversation at %s\n", conversation_path.c_str());
    return 1;
  }
  int conversation_error = engine.load_conversation("jail", conversation);
  if (conversation_error != LOUDER_LOAD_OK) {
    std::fprintf(stderr, "conversation %s failed to load: %s\n", conversation_path.c_str(),
                 load_error_text(conversation_error));
    return 1;
  }

  Position start = engine.find_tag("start");
  if (start == -1) {
    std::fprintf(stderr, "no hub in %s is tagged \"start\"\n", conversation_path.c_str());
    return 1;
  }

  // Two actors in one conversation. Each speaks as one character and, through
  // the config's listening lists, hears that character's internal voices.
  ActorId freya = engine.add_actor(engine.name("jail"), start, "Freya");
  ActorId benjamin = engine.add_actor(engine.name("jail"), start, "Benjamin");
  if (freya == LOUDER_ACTOR_NONE || benjamin == LOUDER_ACTOR_NONE) {
    std::fprintf(stderr, "could not add both actors to the conversation\n");
    return 1;
  }

  Viewport left;
  left.actor = freya;
  left.display_name = "Freya";
  left.key = louder_demo::kLeftKeys[0];

  Viewport right;
  right.actor = benjamin;
  right.display_name = kBenjaminAlias;
  right.key = louder_demo::kRightKeys[0];

  DialogueView view(&engine, left, right);

  // Discard the actor-added events from setup; the initial state comes from
  // asking the engine where everyone is rather than from replaying events.
  engine.drain_events();
  view.consume({});

  louder_demo::Screen screen;

  // The console is asked for its size each frame rather than cached, so
  // resizing the window takes effect immediately.
  auto paint = [&]() {
    int width = screen.width();
    int height = screen.height();
    draw(screen, render(view, view.left(), view.right(), width, height), width, height);
  };

  paint();

  if (check_only) {
    std::printf("\nleft  : %s at %d, %zu choices\n", view.left().display_name.c_str(),
                view.left().position, view.left().choices.size());
    std::printf("right : %s at %d, %zu choices\n", view.right().display_name.c_str(),
                view.right().position, view.right().choices.size());
    for (const std::string& error : view.errors()) {
      std::printf("error : %s\n", error.c_str());
    }
    return 0;
  }

  // One blocking keypress per frame. ESC quits; every other key is offered to
  // the view, which ignores anything that is not one of its choice keys.
  while (true) {
    char key = screen.read_key();
    if (key == 27) {  // ESC
      break;
    }
    view.handle_key(key);
    paint();
  }

  return 0;
}