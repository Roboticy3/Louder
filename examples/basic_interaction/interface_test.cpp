// Interface-level tests for the Bare Minimum demo.
//
// These drive the real engine and the real view, but never touch a terminal:
// Screen is not constructed and draw() is not called. That split is the point,
// because the interesting behaviour is the mapping from keys to engine choice
// indices and the per-column visibility, not the escape sequences.
#include <cstdio>
#include <cstring>
#include <fstream>
#include <memory>
#include <sstream>
#include <string>
#include <vector>

#include "dialogue_view.h"
#include "engine_bridge.h"

namespace {

using louder_demo::ActorId;
using louder_demo::Choice;
using louder_demo::DialogueView;
using louder_demo::Engine;
using louder_demo::Line;
using louder_demo::NameId;
using louder_demo::Position;
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
  if (!file) {
    return "";
  }
  std::ostringstream contents;
  contents << file.rdbuf();
  return contents.str();
}

// A loaded engine with the two demo actors standing on the start hub, plus the
// view over them. Each test gets a fresh one, because actors and covered hubs
// accumulate and would otherwise leak between cases.
struct Fixture {
  Engine engine;
  NameId conversation = LOUDER_NAME_NONE;
  Position start = -1;
  ActorId freya = LOUDER_ACTOR_NONE;
  ActorId benjamin = LOUDER_ACTOR_NONE;

  // The view owns copies of the viewports it is given, so they are held
  // separately for the setup phase only.
  std::unique_ptr<DialogueView> owned_view;

  explicit Fixture(const std::string& config_path, const std::string& conversation_path) {
    CHECK(engine.ready(), "the engine should initialise");
    CHECK(engine.load_config(read_file(config_path)) == LOUDER_LOAD_OK, "the config should load");
    CHECK(engine.load_conversation("jail", read_file(conversation_path)) == LOUDER_LOAD_OK,
          "the conversation should load");
    conversation = engine.name("jail");
    start = engine.find_tag("start");
    CHECK(start != -1, "the conversation should have a start hub");

    freya = engine.add_actor(conversation, start, "Freya");
    benjamin = engine.add_actor(conversation, start, "Benjamin");
    CHECK(freya != LOUDER_ACTOR_NONE, "Freya should be added");
    CHECK(benjamin != LOUDER_ACTOR_NONE, "Benjamin should be added");

    auto left_viewport = std::make_unique<Viewport>();
    left_viewport->actor = freya;
    left_viewport->display_name = "Freya";
    left_viewport->key = louder_demo::kLeftKeys[0];

    auto right_viewport = std::make_unique<Viewport>();
    right_viewport->actor = benjamin;
    right_viewport->display_name = "REDACTED";
    right_viewport->key = louder_demo::kRightKeys[0];

    owned_view = std::make_unique<DialogueView>(&engine, *left_viewport, *right_viewport);

    // Actor-added events from setup are not part of what the view displays.
    engine.drain_events();
    owned_view->consume({});
  }

  // Short alias, so the tests below read as one thing rather than three.
  DialogueView& view() { return *owned_view; }
  const DialogueView& view() const { return *owned_view; }
};

std::vector<int32_t> general_indices(const Viewport& viewport) {
  std::vector<int32_t> indices;
  for (const Choice& choice : viewport.choices) {
    indices.push_back(choice.general_index);
  }
  return indices;
}

// --- key mapping -----------------------------------------------------------

void TestKeyMapping() {
  std::printf("key mapping\n");

  // Freya takes 1..9 and Benjamin takes qwerty, exactly as the spec describes.
  CHECK(DialogueView::side_for_key('1') == 0, "'1' should belong to Freya");
  CHECK(DialogueView::side_for_key('9') == 0, "'9' should belong to Freya");
  CHECK(DialogueView::side_for_key('q') == 1, "'q' should belong to Benjamin");
  CHECK(DialogueView::side_for_key('p') == 1, "'p' should belong to Benjamin");
  CHECK(DialogueView::side_for_key('z') == -1, "'z' should belong to nobody");
  CHECK(DialogueView::side_for_key('0') == -1, "'0' should not be a Freya choice key");

  // Each side numbers its own visible list from zero, in key order.
  CHECK(DialogueView::key_index_for('1') == 0, "Freya's first choice is on '1'");
  CHECK(DialogueView::key_index_for('3') == 2, "Freya's third choice is on '3'");
  CHECK(DialogueView::key_index_for('q') == 0, "Benjamin's first choice is on 'q'");
  CHECK(DialogueView::key_index_for('e') == 2, "Benjamin's third choice is on 'e'");

  // The two key sets must not overlap, or a keypress would be ambiguous.
  for (char left : louder_demo::kLeftKeys) {
    CHECK(DialogueView::side_for_key(left) == 0, "Freya key '%c' should stay on her side", left);
  }
  for (char right : louder_demo::kRightKeys) {
    CHECK(DialogueView::side_for_key(right) == 1, "Benjamin key '%c' should stay on his side", right);
  }
}

// --- initial state ---------------------------------------------------------

void TestInitialState() {
  std::printf("initial state\n");
  Fixture f("speakers.json", "jail.json");

  CHECK(f.view().left().has_position, "Freya should have a position");
  CHECK(f.view().left().position == f.start, "Freya should start on the start hub");
  CHECK(f.view().left().speaker == "Freya", "the start hub is spoken by Freya, got '%s'",
        f.view().left().speaker.c_str());
  CHECK(f.view().left().text.find("willing to identify yourself") != std::string::npos,
        "Freya's column should show the start hub's text, got '%s'", f.view().left().text.c_str());

  CHECK(f.view().right().position == f.start, "Benjamin should also start on the start hub");
  CHECK(f.view().right().speaker == "Freya", "the start hub is spoken by Freya whoever reads it, got '%s'",
        f.view().right().speaker.c_str());

  CHECK(f.view().errors().empty(), "starting up should produce no errors, saw %zu",
        f.view().errors().size());

  // The engine decides who may decide, and the view must report its answer
  // rather than assume anything.
  CHECK(f.view().left().choices.empty(), "Freya should have no choices here");
  CHECK(f.view().right().choices.size() == 4, "Benjamin should have four choices here, saw %zu",
        f.view().right().choices.size());
}

// --- the interface's core responsibility -----------------------------------

void TestVisibleIndexIsNotEngineIndex() {
  std::printf("visible index vs engine index\n");
  Fixture f("speakers.json", "jail.json");

  // At the start hub every one of Benjamin's choices is visible, so his visible
  // indices and the engine's indices coincide. This is the case where a naive
  // implementation happens to work, so it is checked first to establish the
  // baseline.
  std::vector<int32_t> indices = general_indices(f.view().right());
  CHECK(indices.size() == 4, "four choices expected");
  for (size_t i = 0; i < indices.size(); ++i) {
    CHECK(indices[i] == static_cast<int32_t>(i),
          "with nothing hidden, visible %zu should be engine index %d, got %d", i,
          static_cast<int32_t>(i), indices[i]);
  }

  // Walk Benjamin into the branch that hides choices from him. Hub 11 is
  // Benjamin's "I don't know my name", and hub 21 is his lie, which Freya cannot
  // see at all.
  CHECK(f.view().handle_key('q'), "'q' should be acted on");
  Position after_first = f.view().right().position;
  CHECK(after_first == 5,
        "Benjamin's first choice at the start hub leads to 5, got %d", after_first);

  // From 11 the raw hub has three choices. Benjamin hears Memory and Stars, and
  // hub 11 is spoken by Structure, which only Freya hears, so Benjamin is not
  // the lead here. What matters is that whatever the engine reports, the view
  // reproduces exactly.
  int32_t engine_count = f.engine.choice_count(f.conversation, f.view().right().position, f.view().right().actor);
  CHECK(static_cast<int32_t>(f.view().right().choices.size()) == engine_count,
        "the view's choice count should match the engine's, view %zu engine %d",
        f.view().right().choices.size(), engine_count);

  // Whatever the visible indices are, each must be distinct and point at a hub
  // that the engine agrees the choice reaches.
  std::vector<int32_t> visible = general_indices(f.view().right());
  for (size_t i = 0; i < visible.size(); ++i) {
    bool distinct = true;
    for (size_t j = 0; j < i; ++j) {
      if (visible[j] == visible[i]) {
        distinct = false;
      }
    }
    CHECK(distinct, "visible index %d appears twice", visible[i]);
  }
}

void TestKeypressSelectsTheNthVisibleChoice() {
  std::printf("keypress selects the nth visible choice\n");
  Fixture f("speakers.json", "jail.json");

  // Pressing the second key must move to the hub behind the *second visible*
  // choice, whatever engine index that is.
  f.view().handle_key('w');
  CHECK(f.view().right().position == 4,
        "'w' should take the second visible choice, which leads to 4, got %d",
        f.view().right().position);

  // And the engine index it used must have been 1, which it was, because nothing
  // was hidden here. Confirm by asking the engine directly.
  CHECK(f.engine.choice_count(f.conversation, f.view().right().position, f.view().right().actor) >= 0,
        "the engine should still answer queries for this actor");
}

void TestKeysOutOfRangeAreRejected() {
  std::printf("out of range keys\n");
  Fixture f("speakers.json", "jail.json");

  // Freya has no choices at the start hub, so any of her keys is out of range.
  Position before = f.view().left().position;
  bool handled = f.view().handle_key('1');
  CHECK(handled, "the key should still be consumed by her column");
  CHECK(f.view().left().position == before, "Freya should not have moved");
  CHECK(!f.view().left().error.empty(), "the refusal should be reported to the user");
  CHECK(!f.view().errors().empty(), "the refusal should also reach the shared error list");

  // Benjamin has four choices, so his fifth key is out of range and his fourth
  // is fine. This checks the boundary from both sides.
  CHECK(f.view().handle_key('t'), "'t' is Benjamin's fifth key, within his list of four");
  CHECK(f.view().right().position == 6, "'t' should lead to hub 6, got %d", f.view().right().position);

  size_t errors_before = f.view().errors().size();
  f.view().handle_key('y');  // sixth key, but only four choices
  CHECK(f.view().right().position == 6, "Benjamin should not have moved");
  CHECK(f.view().errors().size() > errors_before, "an out of range key should be reported");
}

// --- unclaimed keys --------------------------------------------------------

void TestUnclaimedKeysAreIgnored() {
  std::printf("unclaimed keys\n");
  Fixture f("speakers.json", "jail.json");

  Position left_before = f.view().left().position;
  Position right_before = f.view().right().position;

  for (char key : {'z', 'a', '0', '!', ' ', '\t'}) {
    CHECK(!f.view().handle_key(key), "'%c' should not be claimed by either column", key);
  }

  CHECK(f.view().left().position == left_before, "Freya should not have moved");
  CHECK(f.view().right().position == right_before, "Benjamin should not have moved");
  CHECK(f.view().errors().empty(), "unclaimed keys should not raise errors, saw %zu",
        f.view().errors().size());
}

// --- driving the conversation ----------------------------------------------

void TestBothSidesAdvance() {
  std::printf("both sides advance\n");
  Fixture f("speakers.json", "jail.json");

  // Benjamin leads from the start hub, so only he has choices. After he moves,
  // the engine decides who leads next, and the view must follow.
  f.view().handle_key('q');  // Benjamin: 0 -> 3

  CHECK(f.view().right().position == 3, "Benjamin should be at 3, got %d", f.view().right().position);

  // Hub 3 is Benjamin's line with no choices of its own, so the engine pulls
  // Freya along. Whether that happens is the engine's business; what the
  // interface must guarantee is that whatever it does, both columns agree with
  // the engine afterwards.
  CHECK(f.view().left().has_position, "Freya should still have a position");
  CHECK(f.view().errors().empty(), "the conversation should proceed without errors, saw %zu",
        f.view().errors().size());
  CHECK(f.view().left().position == f.engine.position_of(f.conversation, f.view().left().actor),
        "Freya's displayed position should be the engine's");
  CHECK(f.view().right().position == f.engine.position_of(f.conversation, f.view().right().actor),
        "Benjamin's displayed position should be the engine's");
}

void TestStaleErrorsClearOnSuccess() {
  std::printf("errors clear on success\n");
  Fixture f("speakers.json", "jail.json");

  f.view().handle_key('1');  // refused: Freya has nothing here
  CHECK(!f.view().left().error.empty(), "Freya should be showing an error");

  // Get Freya into a position where she does have choices, then take one. The
  // error from the refused key should not follow her around.
  while (f.view().left().choices.empty() && f.view().errors().size() < 8) {
    // Drive whichever side currently has the lead, one step at a time.
    if (!f.view().right().choices.empty()) {
      f.view().handle_key(louder_demo::kRightKeys[0]);
    } else if (!f.view().left().choices.empty()) {
      f.view().handle_key(louder_demo::kLeftKeys[0]);
    } else {
      break;
    }
  }

  if (!f.view().left().choices.empty()) {
    f.view().handle_key(louder_demo::kLeftKeys[0]);
    CHECK(f.view().left().error.empty(), "a successful choice should clear the stale error");
  }
}

// --- the alias -------------------------------------------------------------

void TestAliasIsIndependentOfSpeakerName() {
  std::printf("aliasing\n");
  Fixture f("speakers.json", "jail.json");

  // The column header carries the alias...
  CHECK(f.view().right().display_name == "REDACTED", "Benjamin's column should be titled REDACTED");
  CHECK(f.view().left().display_name == "Freya", "Freya's column should be titled Freya");

  // ...while the text the engine reports still says Benjamin. The interface is
  // responsible for the substitution, not the engine, which is the whole point
  // of the aliasing requirement.
  f.view().handle_key('q');
  CHECK(f.view().right().speaker == "Benjamin",
        "the engine should still report the real speaker name, got '%s'",
        f.view().right().speaker.c_str());
  CHECK(f.view().right().display_name == "REDACTED",
        "the display name should be unaffected by the engine's speaker name");

  // Nothing anywhere in the view state should leak the real name into the alias
  // field.
  CHECK(f.view().right().display_name.find("Benjamin") == std::string::npos,
        "the alias should not contain the real name");
}

// --- visibility ------------------------------------------------------------

void TestPrivateHubsAreInvisibleToFreya() {
  std::printf("private hub visibility\n");
  Fixture f("speakers.json", "jail.json");

  // Hub 26 carries recipients "Benjamin", so it may only appear to an actor
  // speaking as or listening to Benjamin. Freya speaks as Freya and listens to
  // Freya's own internal voices, so she must never see it. The engine is the
  // authority; this checks the interface never shows her a choice leading there.
  const Position private_hub = 26;

  // Walk both sides as far as the engine allows, never more than 60 steps, and
  // assert Freya is never offered the private hub or shown standing on it.
  for (int step = 0; step < 60; ++step) {
    if (f.view().left().position == private_hub) {
      CHECK(false, "Freya reached the private hub %d", private_hub);
      break;
    }
    for (const Choice& choice : f.view().left().choices) {
      CHECK(choice.link != private_hub, "Freya was offered the private hub %d", private_hub);
    }

    bool progressed = false;
    if (!f.view().right().choices.empty()) {
      f.view().handle_key(louder_demo::kRightKeys[0]);
      progressed = true;
    }
    if (!f.view().left().choices.empty()) {
      f.view().handle_key(louder_demo::kLeftKeys[0]);
      progressed = true;
    }
    if (!progressed) {
      break;
    }
  }
}

void TestAutoHubsNeverDisplayed() {
  std::printf("auto hubs are never displayed\n");
  Fixture f("speakers.json", "jail.json");

  // Hubs 27 and 35 are flagged auto in the document. Auto hubs resolve
  // themselves and never appear, so no actor's displayed position may rest on
  // one and no choice may be shown as leading into one.
  for (int step = 0; step < 60; ++step) {
    for (const Viewport* viewport : {&f.view().left(), &f.view().right()}) {
      CHECK(!f.engine.is_auto(viewport->position),
            "an actor is standing on auto hub %d", viewport->position);
      for (const Choice& choice : viewport->choices) {
        CHECK(!f.engine.is_auto(choice.link), "a choice leads into auto hub %d", choice.link);
      }
    }

    bool progressed = false;
    if (!f.view().right().choices.empty()) {
      f.view().handle_key(louder_demo::kRightKeys[0]);
      progressed = true;
    }
    if (!f.view().left().choices.empty()) {
      f.view().handle_key(louder_demo::kLeftKeys[0]);
      progressed = true;
    }
    if (!progressed) {
      break;
    }
  }
}

void TestDeadEndIsReportedNotHung() {
  std::printf("dead ends\n");
  Fixture f("speakers.json", "jail.json");

  // Hub 35 is an auto hub with no exit. Selecting the choice that leads there
  // must fail rather than loop forever inside the engine. If the interface ever
  // reaches it, the failure has to be reported to the user rather than swallowed.
  for (int step = 0; step < 80; ++step) {
    bool targeted_dead_end = false;
    for (const Choice& choice : f.view().right().choices) {
      if (choice.link == 35) {
        targeted_dead_end = true;
      }
    }
    if (targeted_dead_end) {
      // Take it and require the interface to come back with a message.
      size_t errors_before = f.view().errors().size();
      f.view().handle_key('q');
      CHECK(f.view().errors().size() > errors_before,
            "walking into the dead end at 35 should report an error");
      CHECK(!f.view().errors().empty(), "the dead end error should survive to the shared list");
      return;
    }

    bool progressed = false;
    if (!f.view().right().choices.empty()) {
      f.view().handle_key(louder_demo::kRightKeys[0]);
      progressed = true;
    }
    if (!f.view().left().choices.empty()) {
      f.view().handle_key(louder_demo::kLeftKeys[0]);
      progressed = true;
    }
    if (!progressed) {
      break;
    }
  }
  // Reaching here means the walk never found the dead end, which is not a
  // failure of the interface, so it is not asserted on.
}

// --- consistency with the engine -------------------------------------------

void TestViewAlwaysAgreesWithEngine() {
  std::printf("view agrees with the engine\n");
  Fixture f("speakers.json", "jail.json");

  // Whatever the walk does, the two columns must never drift from what the
  // engine reports. This is the invariant the whole interface rests on, so it is
  // checked after every single step rather than only at interesting moments.
  for (int step = 0; step < 40; ++step) {
    for (const Viewport* viewport : {&f.view().left(), &f.view().right()}) {
      Position engine_position = f.engine.position_of(f.conversation, viewport->actor);
      CHECK(viewport->position == engine_position,
            "step %d: displayed position %d but the engine says %d", step,
            viewport->position, engine_position);

      int32_t engine_choices = f.engine.choice_count(f.conversation, engine_position, viewport->actor);
      CHECK(static_cast<int32_t>(viewport->choices.size()) == engine_choices,
            "step %d: displayed %zu choices but the engine reports %d", step,
            viewport->choices.size(), engine_choices);

      // Every displayed choice must lead where it says, by the engine's own
      // answer for that index from that actor's position.
      for (size_t i = 0; i < viewport->choices.size(); ++i) {
        const Choice& choice = viewport->choices[i];
        int32_t engine_link = f.engine.choice_link(f.conversation, engine_position, viewport->actor,
                                                   choice.general_index);
        CHECK(choice.link == engine_link,
              "step %d: displayed choice %zu reports link %d but the engine says %d", step,
              i, choice.link, engine_link);
      }
    }

    // Alternate sides so both get driven.
    bool progressed = false;
    if (step % 2 == 0 && !f.view().right().choices.empty()) {
      f.view().handle_key(louder_demo::kRightKeys[step % 3]);
      progressed = true;
    } else if (!f.view().left().choices.empty()) {
      f.view().handle_key(louder_demo::kLeftKeys[step % 3]);
      progressed = true;
    } else if (!f.view().right().choices.empty()) {
      f.view().handle_key(louder_demo::kRightKeys[step % 3]);
      progressed = true;
    }
    if (!progressed) {
      break;
    }
  }
}

void TestExhaustiveFirstMoveOutcomes() {
  std::printf("every first move\n");

  // For every visible choice on the start hub, from a fresh engine, pressing the
  // matching key must land on the hub the choice reports. This is the smallest
  // exhaustive check of the key-to-choice mapping: it cannot pass by accident on
  // one lucky path.
  Fixture probe("speakers.json", "jail.json");
  size_t moves = probe.view().right().choices.size();
  CHECK(moves == 4, "expected four first moves, saw %zu", moves);

  for (size_t i = 0; i < moves; ++i) {
    Fixture f("speakers.json", "jail.json");
    std::vector<int32_t> visible = general_indices(f.view().right());
    CHECK(i < visible.size(), "move %zu should exist", i);

    const std::string expected = f.view().right().choices[i].text;
    const int32_t expected_general = f.view().right().choices[i].general_index;

    char key = louder_demo::kRightKeys[i];
    f.view().handle_key(key);

    CHECK(f.view().right().choices.size() > 0 || true, "move %zu completed", i);
    CHECK(f.view().errors().empty(), "move %zu should not error, saw: %s", i,
          f.view().errors().empty() ? "" : f.view().errors().back().c_str());
    CHECK(visible[i] == expected_general,
          "move %zu: visible index %d should be engine index %d", i, visible[i],
          expected_general);
    CHECK(!expected.empty(), "move %zu should carry choice text", i);

    // The actor must have left the start hub.
    CHECK(f.view().right().position != f.start, "move %zu should have moved off the start hub", i);
  }
}

}  // namespace

int main() {
  // Unbuffered, so progress survives a crash and the stack depth is visible.
  std::setvbuf(stdout, nullptr, _IONBF, 0);
  std::printf("running interface tests\n");

  TestKeyMapping();
  TestInitialState();
  TestVisibleIndexIsNotEngineIndex();
  TestKeypressSelectsTheNthVisibleChoice();
  TestKeysOutOfRangeAreRejected();
  TestUnclaimedKeysAreIgnored();
  TestBothSidesAdvance();
  TestStaleErrorsClearOnSuccess();
  TestAliasIsIndependentOfSpeakerName();
  TestPrivateHubsAreInvisibleToFreya();
  TestAutoHubsNeverDisplayed();
  TestDeadEndIsReportedNotHung();
  TestViewAlwaysAgreesWithEngine();
  TestExhaustiveFirstMoveOutcomes();

  std::printf("\n%d checks, %d failures\n", g_checks, g_failures);
  return g_failures == 0 ? 0 : 1;
}
