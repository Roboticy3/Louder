// Rendering tests for the split-column frame.
//
// The layout is a pure function of the view state and the console size, so all
// of it is checked here as plain text. No console is opened.
//
// The property that matters most is that nothing written can reach the last
// cell of a row. A character in the final column makes the cursor wrap, and a
// wrapped console scrolls the whole screen up one line, which is how a stale
// row ends up sitting where the header belongs. That is a rendering bug this
// suite exists to prevent.
#include <cstdio>
#include <fstream>
#include <memory>
#include <sstream>
#include <string>
#include <vector>

#include "dialogue_view.h"
#include "engine_bridge.h"
#include "screen.h"

namespace {

using louder_demo::ActorId;
using louder_demo::DialogueView;
using louder_demo::Engine;
using louder_demo::Frame;
using louder_demo::NameId;
using louder_demo::Position;
using louder_demo::Style;
using louder_demo::Viewport;

int g_checks = 0;
int g_failures = 0;

#define CHECK(condition, ...)                             \
  do {                                                    \
    ++g_checks;                                           \
    if (!(condition)) {                                   \
      ++g_failures;                                       \
      std::printf("  FAIL %s:%d: ", __FILE__, __LINE__);  \
      std::printf(__VA_ARGS__);                           \
      std::printf("\n");                                  \
    }                                                     \
  } while (0)

std::string read_file(const std::string& path) {
  std::ifstream file(path, std::ios::binary);
  if (!file) return "";
  std::ostringstream contents;
  contents << file.rdbuf();
  return contents.str();
}

// The column width render() uses for a given console width. Duplicated from
// screen.cpp deliberately: if the layout changes, this should stop matching and
// the width assertions below should be revisited.
int column_width_for(int width) {
  int usable = width - 2 < 20 ? 20 : width - 2;
  int half = usable / 2;
  if (half < 20) half = 20;
  if (half > 58) half = 58;
  return half;
}

// A loaded engine, two actors at the start hub, and a view over them.
struct Fixture {
  Engine engine;
  NameId conversation = LOUDER_NAME_NONE;
  Position start = -1;
  ActorId freya = LOUDER_ACTOR_NONE;
  ActorId benjamin = LOUDER_ACTOR_NONE;
  std::unique_ptr<DialogueView> owned_view;

  Fixture() {
    engine.load_config(read_file("speakers.json"));
    engine.load_conversation("jail", read_file("jail.json"));
    conversation = engine.name("jail");
    start = engine.find_tag("start");
    freya = engine.add_actor(conversation, start, "Freya");
    benjamin = engine.add_actor(conversation, start, "Benjamin");

    Viewport left;
    left.actor = freya;
    left.display_name = "Freya";
    left.key = louder_demo::kLeftKeys[0];
    Viewport right;
    right.actor = benjamin;
    right.display_name = "REDACTED";
    right.key = louder_demo::kRightKeys[0];

    engine.drain_events();
    owned_view = std::make_unique<DialogueView>(&engine, left, right);
    owned_view->consume({});
  }

  DialogueView& view() { return *owned_view; }
};

// --- the invariants that stop the screen scrolling ------------------------

void TestNoLineReachesTheLastColumn() {
  std::printf("no line reaches the last column\n");
  Fixture f;

  // Every console width the demo is likely to meet, and a few it is not.
  for (int width : {40, 60, 80, 100, 120, 200}) {
    const int column_width = column_width_for(width);
    Frame frame = louder_demo::render(f.view(), f.view().left(), f.view().right(), width, 24);
    for (size_t row = 0; row < frame.rows.size(); ++row) {
      const auto& line = frame.rows[row];
      CHECK(static_cast<int>(line.left.size()) <= column_width,
            "at width %d row %zu the left cell is %zu chars, over the column width %d", width,
            row, line.left.size(), column_width);
      CHECK(static_cast<int>(line.right.size()) <= column_width,
            "at width %d row %zu the right cell is %zu chars, over the column width %d", width,
            row, line.right.size(), column_width);
    }
    CHECK(static_cast<int>(frame.footer.size()) <= width - 1,
          "at width %d the footer is %zu chars, which reaches the last column", width,
          frame.footer.size());
  }
}

void TestFrameNeverExceedsTheConsoleHeight() {
  std::printf("the frame never exceeds the console height\n");
  Fixture f;

  // Including heights too short to show anything, which must not produce a
  // negative capacity or a frame taller than the screen.
  for (int height : {1, 2, 3, 5, 10, 24, 60}) {
    Frame frame = louder_demo::render(f.view(), f.view().left(), f.view().right(), 80, height);
    // Rows 0..height-1 exist, and the footer is drawn on row height-1, so the
    // body has to stop at row height-2.
    CHECK(static_cast<int>(frame.rows.size()) <= height - 1,
          "at height %d the frame has %zu rows, which would be drawn past the last row", height,
          frame.rows.size());
  }
}

void TestLongTextIsWrappedNotOverflowed() {
  std::printf("long text is wrapped, not overflowed\n");
  Fixture f;

  // Hub 1 in the demo carries a long paragraph of Freya's internal voice. Walk
  // to it and check the wrapping holds at a narrow width.
  const int width = 50;
  const int column_width = column_width_for(width);
  Frame frame = louder_demo::render(f.view(), f.view().left(), f.view().right(), width, 40);

  bool saw_wrapped_line = false;
  for (size_t row = 0; row < frame.rows.size(); ++row) {
    if (static_cast<int>(frame.rows[row].left.size()) > column_width) {
      CHECK(false, "row %zu overflows: %zu > %d", row, frame.rows[row].left.size(), column_width);
    }
    if (frame.rows[row].left.find(' ') != std::string::npos &&
        frame.rows[row].left.size() > 20) {
      saw_wrapped_line = true;
    }
  }
  // The start hub's question is long enough to wrap at 50 columns.
  CHECK(saw_wrapped_line, "expected at least one wrapped line at width 50");
}

// --- the content ----------------------------------------------------------

void TestHeaderNamesBothColumnsWithTheirKeys() {
  std::printf("header names both columns\n");
  Fixture f;

  Frame frame = louder_demo::render(f.view(), f.view().left(), f.view().right(), 80, 24);
  CHECK(!frame.rows.empty(), "the frame should have a header row");
  if (frame.rows.empty()) return;

  CHECK(frame.rows[0].left.find("Freya") != std::string::npos,
        "the left header should name Freya, got '%s'", frame.rows[0].left.c_str());
  CHECK(frame.rows[0].left.find("[1]") != std::string::npos,
        "the left header should show her key, got '%s'", frame.rows[0].left.c_str());
  CHECK(frame.rows[0].right.find("REDACTED") != std::string::npos,
        "the right header should show the alias, got '%s'", frame.rows[0].right.c_str());
  CHECK(frame.rows[0].right.find("Benjamin") == std::string::npos,
        "the right header must not leak the real name, got '%s'", frame.rows[0].right.c_str());
  CHECK(frame.rows[0].right.find("[q]") != std::string::npos,
        "the right header should show his key, got '%s'", frame.rows[0].right.c_str());
}

void TestChoicesAreKeyedInOrder() {
  std::printf("choices are keyed in order\n");
  Fixture f;

  Frame frame = louder_demo::render(f.view(), f.view().left(), f.view().right(), 80, 24);

  // Benjamin has four choices at the start hub and they must be labelled q, w,
  // e, r in that order. Reading them in the frame is the only way to catch a
  // labelling bug, since the view state alone does not carry the keys.
  std::vector<std::string> keys_in_order;
  for (size_t row = 0; row < frame.rows.size(); ++row) {
    const std::string& text = frame.rows[row].right;
    if (text.size() >= 2 && text[1] == ')' &&
        (text[0] == 'q' || text[0] == 'w' || text[0] == 'e' || text[0] == 'r' ||
         text[0] == 't' || text[0] == 'y')) {
      keys_in_order.push_back(std::string(1, text[0]));
    }
  }
  CHECK(keys_in_order.size() == 4, "expected four keyed choices in the right column, saw %zu",
        keys_in_order.size());
  if (keys_in_order.size() == 4) {
    CHECK(keys_in_order[0] == "q" && keys_in_order[1] == "w" && keys_in_order[2] == "e" &&
              keys_in_order[3] == "r",
          "choices should be keyed q, w, e, r, got %s %s %s %s", keys_in_order[0].c_str(),
          keys_in_order[1].c_str(), keys_in_order[2].c_str(), keys_in_order[3].c_str());
  }
}

void TestIdleSideSaysWhoIsThinking() {
  std::printf("the idle side names who is thinking\n");
  Fixture f;

  Frame frame = louder_demo::render(f.view(), f.view().left(), f.view().right(), 80, 24);

  // Freya has no choices and Benjamin has four, so her column must say so, in
  // the idle style, using Benjamin's alias.
  bool found = false;
  for (size_t row = 0; row < frame.rows.size(); ++row) {
    if (frame.rows[row].left.find("is thinking") != std::string::npos) {
      found = true;
      CHECK(frame.rows[row].left.find("REDACTED") != std::string::npos,
            "the thinking line should use the alias, got '%s'", frame.rows[row].left.c_str());
      CHECK(frame.rows[row].left_style == Style::Idle,
            "the thinking line should be styled as idle");
    }
  }
  CHECK(found, "Freya's column should say Benjamin is thinking");

  // Benjamin's column must not claim Freya is thinking, since he has choices.
  for (size_t row = 0; row < frame.rows.size(); ++row) {
    CHECK(frame.rows[row].right.find("is thinking") == std::string::npos,
          "the side with choices should not claim the other is thinking");
  }
}

void TestStalledSideSaysSo() {
  std::printf("a stalled conversation says so\n");

  // Walk to hub 3, where neither actor has a choice. Both columns must report
  // the stall rather than accusing each other of thinking.
  Fixture f;
  f.view().handle_key('q');
  Frame frame = louder_demo::render(f.view(), f.view().left(), f.view().right(), 80, 24);

  bool left_says = false;
  bool right_says = false;
  for (size_t row = 0; row < frame.rows.size(); ++row) {
    if (frame.rows[row].left.find("stalled") != std::string::npos) left_says = true;
    if (frame.rows[row].right.find("stalled") != std::string::npos) right_says = true;
  }
  CHECK(left_says, "the left column should report the stall");
  CHECK(right_says, "the right column should report the stall");
}

void TestErrorsAreShownOnTheOffendingSide() {
  std::printf("errors show on the side that made them\n");
  Fixture f;

  // Freya has no choices at the start hub, so her key is refused.
  f.view().handle_key('1');
  Frame frame = louder_demo::render(f.view(), f.view().left(), f.view().right(), 80, 24);

  bool left_error = false;
  bool right_error = false;
  for (size_t row = 0; row < frame.rows.size(); ++row) {
    if (frame.rows[row].left.find("no choice") != std::string::npos) left_error = true;
    if (frame.rows[row].right.find("no choice") != std::string::npos) right_error = true;
  }
  CHECK(left_error, "the refusal should be shown in the column that caused it");
  CHECK(!right_error, "the other column should not show someone else's refusal");

  CHECK(frame.footer.find("last error") != std::string::npos,
        "the footer should carry the newest error, got '%s'", frame.footer.c_str());
  CHECK(frame.footer_style == Style::Error, "the footer should be styled as an error");
}

void TestFooterAlwaysOffersAWayOut() {
  std::printf("the footer always offers a way out\n");
  Fixture f;
  Frame clean = louder_demo::render(f.view(), f.view().left(), f.view().right(), 80, 24);
  CHECK(clean.footer.find("ESC") != std::string::npos, "the footer should mention ESC");

  f.view().handle_key('1');  // now there is an error
  Frame errored = louder_demo::render(f.view(), f.view().left(), f.view().right(), 80, 24);
  CHECK(errored.footer.find("ESC") != std::string::npos,
        "the quit hint should survive an error, got '%s'", errored.footer.c_str());
}

}  // namespace

int main() {
  std::setvbuf(stdout, nullptr, _IONBF, 0);
  std::printf("running render tests\n");

  TestNoLineReachesTheLastColumn();
  TestFrameNeverExceedsTheConsoleHeight();
  TestLongTextIsWrappedNotOverflowed();
  TestHeaderNamesBothColumnsWithTheirKeys();
  TestChoicesAreKeyedInOrder();
  TestIdleSideSaysWhoIsThinking();
  TestStalledSideSaysSo();
  TestErrorsAreShownOnTheOffendingSide();
  TestFooterAlwaysOffersAWayOut();

  std::printf("\n%d checks, %d failures\n", g_checks, g_failures);
  return g_failures == 0 ? 0 : 1;
}