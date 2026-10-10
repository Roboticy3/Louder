// Throwaway: drive the view through real keypresses and print the frame after
// each one, so the rendered output can be read without a terminal.
#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>

#include "dialogue_view.h"
#include "engine_bridge.h"
#include "screen.h"

using namespace louder_demo;

static std::string read_file(const std::string& path) {
  std::ifstream file(path, std::ios::binary);
  if (!file) return "";
  std::ostringstream contents;
  contents << file.rdbuf();
  return contents.str();
}

static void print_frame(const Frame& frame, int width, int height) {
  const int column_width = std::min(std::max(std::max(width - 2, 20) / 2, 20), 58);
  std::printf("    +%s+%s+\n", std::string(column_width, '-').c_str(),
              std::string(column_width, '-').c_str());
  for (size_t i = 0; i < frame.rows.size(); ++i) {
    const FrameRow& row = frame.rows[i];
    std::string left = row.left;
    if (static_cast<int>(left.size()) < column_width) {
      left += std::string(column_width - left.size(), ' ');
    }
    std::string right = row.right;
    if (static_cast<int>(right.size()) < column_width) {
      right += std::string(column_width - right.size(), ' ');
    }
    std::printf("    |%-*s|%*s|\n", column_width, left.c_str(), column_width, right.c_str());
  }
  std::printf("    footer: %s\n", frame.footer.c_str());
  std::printf("    rows=%zu height=%d last_row_would_be=%zu\n\n", frame.rows.size(), height,
              frame.rows.size());
}

int main(int argc, char** argv) {
  std::setvbuf(stdout, nullptr, _IONBF, 0);
  int width = argc > 1 ? std::atoi(argv[1]) : 80;
  int height = argc > 2 ? std::atoi(argv[2]) : 24;

  Engine engine;
  engine.load_config(read_file("speakers.json"));
  engine.load_conversation("jail", read_file("jail.json"));
  NameId conv = engine.name("jail");
  Position start = engine.find_tag("start");

  ActorId freya = engine.add_actor(conv, start, "Freya");
  ActorId ben = engine.add_actor(conv, start, "Benjamin");

  Viewport left;
  left.actor = freya;
  left.display_name = "Freya";
  left.key = kLeftKeys[0];
  Viewport right;
  right.actor = ben;
  right.display_name = "REDACTED";
  right.key = kRightKeys[0];

  engine.drain_events();
  DialogueView view(&engine, left, right);
  view.consume({});

  std::printf("=== initial (console %dx%d) ===\n", width, height);
  print_frame(render(view, view.left(), view.right(), width, height), width, height);

  // A plausible play sequence. Benjamin leads from the start hub with q, which
  // leaves Freya a single CONTINUE into her Volition thought; she takes it with
  // her backtick key rather than 1, to exercise the second binding.
  const char* keys = "q`ww1";
  for (int i = 0; keys[i] != '\0'; ++i) {
    std::printf("=== after '%c' ===\n", keys[i]);
    bool handled = view.handle_key(keys[i]);
    std::printf("    handled=%d freya=%d/%zu benjamin=%d/%zu errors=%zu\n", (int)handled,
                view.left().position, view.left().choices.size(), view.right().position,
                view.right().choices.size(), view.errors().size());
    for (const std::string& e : view.errors()) {
      std::printf("      err: %s\n", e.c_str());
    }
    print_frame(render(view, view.left(), view.right(), width, height), width, height);
  }
  return 0;
}