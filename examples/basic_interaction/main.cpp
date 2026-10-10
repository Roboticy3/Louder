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
  std::string conversation_path = argc > 1 ? argv[1] : "jail.json";
  std::string config_path = argc > 2 ? argv[2] : "speakers.json";

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
  draw(screen, view, view.left(), view.right());

  // One blocking keypress per frame. ESC quits; every other key is offered to
  // the view, which ignores anything that is not one of its choice keys.
  while (true) {
    char key = screen.read_key();
    if (key == 27) {  // ESC
      break;
    }
    view.handle_key(key);
    draw(screen, view, view.left(), view.right());
  }

  return 0;
}