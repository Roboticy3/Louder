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

// Pull the key out of a line rendered as "[key] text". Only single-character
// keys count: the CONTINUE state renders "[1 or Tab]", which is a different
// shape and is checked separately.
bool bracketed_key(const std::string& line, std::string* key) {
  if (line.size() < 4 || line[0] != '[' || line[2] != ']') {
    return false;
  }
  if (line.find(" or ") != std::string::npos) {
    return false;
  }
  *key = line.substr(1, 1);
  return true;
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
  // Each choice renders as "[key] text".
  std::vector<std::string> keys_in_order;
  for (size_t row = 0; row < frame.rows.size(); ++row) {
    std::string key;
    if (bracketed_key(frame.rows[row].right, &key)) {
      keys_in_order.push_back(key);
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

  // Freya has none, so her column must show no keyed choice at all.
  for (size_t row = 0; row < frame.rows.size(); ++row) {
    std::string key;
    CHECK(!bracketed_key(frame.rows[row].left, &key),
          "Freya has no choices, but row %zu reads '%s'", row, frame.rows[row].left.c_str());
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

  // Hub 31 has no choices at all and no way out, so standing there leaves both
  // sides empty. Both columns must report the stall rather than accusing each
  // other of thinking.
  Engine engine;
  engine.load_config(read_file("speakers.json"));
  engine.load_conversation("jail", read_file("jail.json"));
  NameId conversation = engine.name("jail");
  ActorId freya = engine.add_actor(conversation, 31, "Freya");
  ActorId ben = engine.add_actor(conversation, 31, "Benjamin");

  Viewport left;
  left.actor = freya;
  left.display_name = "Freya";
  Viewport right;
  right.actor = ben;
  right.display_name = "REDACTED";
  engine.drain_events();
  DialogueView view(&engine, left, right);
  view.consume({});

  CHECK(view.left().choices.empty(), "hub 31 should offer Freya nothing");
  CHECK(view.right().choices.empty(), "hub 31 should offer Benjamin nothing");

  Frame frame = louder_demo::render(view, view.left(), view.right(), 80, 24);
  bool left_says = false;
  bool right_says = false;
  for (size_t row = 0; row < frame.rows.size(); ++row) {
    if (frame.rows[row].left.find("stalled") != std::string::npos) left_says = true;
    if (frame.rows[row].right.find("stalled") != std::string::npos) right_says = true;
  }
  CHECK(left_says, "the left column should report the stall");
  CHECK(right_says, "the right column should report the stall");
}

void TestSingleChoiceRendersAsContinue() {
  std::printf("a lone choice renders as CONTINUE with both keys\n");

  // Hub 3 offers Freya exactly one choice, into Volition's thought, after
  // Benjamin has moved them both there. That single choice has to read as
  // CONTINUE and name both of her keys.
  Engine engine;
  engine.load_config(read_file("speakers.json"));
  engine.load_conversation("jail", read_file("jail.json"));
  NameId conversation = engine.name("jail");
  Position start = engine.find_tag("start");
  ActorId freya = engine.add_actor(conversation, start, "Freya");
  ActorId ben = engine.add_actor(conversation, start, "Benjamin");
  engine.drain_events();

  Viewport left;
  left.actor = freya;
  left.display_name = "Freya";
  Viewport right;
  right.actor = ben;
  right.display_name = "REDACTED";
  DialogueView view(&engine, left, right);
  view.consume({});

  CHECK(view.right().choices.size() == 4, "Benjamin should lead with four choices, saw %zu",
        view.right().choices.size());
  CHECK(!view.left().is_continue(), "with four choices on the other side, Freya is not continuing");

  view.handle_key('q');  // Benjamin: 0 -> 3, Freya dragged along

  CHECK(view.left().position == 3, "Freya should be at 3, got %d", view.left().position);
  CHECK(view.left().choices.size() == 1, "Freya should have one choice at 3, saw %zu",
        view.left().choices.size());
  CHECK(view.left().is_continue(), "a side with one choice should be in the continue state");

  Frame frame = louder_demo::render(view, view.left(), view.right(), 80, 24);

  bool found = false;
  for (size_t row = 0; row < frame.rows.size(); ++row) {
    if (frame.rows[row].left.find(louder_demo::kContinueLabel) != std::string::npos) {
      found = true;
      CHECK(frame.rows[row].left.find("1") != std::string::npos,
            "the CONTINUE line should name her number key, got '%s'", frame.rows[row].left.c_str());
      CHECK(frame.rows[row].left.find("backtick") != std::string::npos,
            "the CONTINUE line should name her backtick, got '%s'", frame.rows[row].left.c_str());
      CHECK(frame.rows[row].left.find("Volition") == std::string::npos,
            "the choice's own text should be replaced by CONTINUE, got '%s'",
            frame.rows[row].left.c_str());
    }
  }
  CHECK(found, "Freya's column should show CONTINUE");

  // Benjamin has no choices here, so his column must not claim to be continuing
  // and must not name a CONTINUE key.
  for (size_t row = 0; row < frame.rows.size(); ++row) {
    CHECK(frame.rows[row].right.find(louder_demo::kContinueLabel) == std::string::npos,
          "a side with no choices should not show CONTINUE, got '%s'",
          frame.rows[row].right.c_str());
  }
}

void TestContinueKeyIsOnlyLiveInTheContinueState() {
  std::printf("the CONTINUE key is inert otherwise\n");

  // The backtick and Tab are context sensitive: with several choices they must
  // do nothing at all, not move an actor and not raise an error.
  Fixture f;
  CHECK(f.view().right().choices.size() == 4, "the premise is four choices on the right");

  size_t errors_before = f.view().errors().size();
  Position left_before = f.view().left().position;
  Position right_before = f.view().right().position;

  CHECK(!f.view().handle_key(louder_demo::kLeftContinueKey),
        "the backtick should not be claimed while Freya has several choices");
  CHECK(!f.view().handle_key(louder_demo::kRightContinueKey),
        "Tab should not be claimed while Benjamin has several choices");

  CHECK(f.view().errors().size() == errors_before,
        "an inert continue key should not be reported as an error, saw %zu new",
        f.view().errors().size() - errors_before);
  CHECK(f.view().left().position == left_before, "Freya should not have moved");
  CHECK(f.view().right().position == right_before, "Benjamin should not have moved");
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
  TestSingleChoiceRendersAsContinue();
  TestContinueKeyIsOnlyLiveInTheContinueState();
  TestStalledSideSaysSo();
  TestErrorsAreShownOnTheOffendingSide();
  TestFooterAlwaysOffersAWayOut();

  std::printf("\n%d checks, %d failures\n", g_checks, g_failures);
  return g_failures == 0 ? 0 : 1;
}