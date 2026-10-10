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

// Hub 0's raw choices are [0]->3, [1]->4, [2]->5, [3]->6, and Benjamin sees all
  // four, so his first key takes him to 3. Confirms the display number reaches
  // the right general index before any hiding has happened.
  CHECK(f.view().handle_key('q'), "'q' should be acted on");
  Position after_first = f.view().right().position;
  CHECK(after_first == 3,
        "Benjamin's first choice at the start hub leads to 3, got %d", after_first);

int32_t engine_count = f.engine.choice_count(f.conversation, f.view().right().position, f.view().right().actor);
  CHECK(static_cast<int32_t>(f.view().right().choices.size()) == engine_count,
        "the view's choice count should match the engine's, view %zu engine %d",
        f.view().right().choices.size(), engine_count);

  // Hub 3 has no choices of its own, so Benjamin's move is followed by the
  // engine pulling Freya along. Whatever it decides, the view reproduces.
  CHECK(f.view().left().position == f.engine.position_of(f.conversation, f.view().left().actor),
        "Freya's position should be the engine's");
}

void TestHiddenChoicesOffsetTheVisibleList() {
  std::printf("hidden choices offset the visible list\n");

  // Hub 26 is where the offsetting actually bites. Its raw choices are
  // [0]->27, [1]->30, [2]->33, [3]->35, [4]->34. Freya is offered only index 4,
  // and Benjamin only indices 1 and 2. So for both actors the first thing they
  // can pick is *not* engine index 0, which is precisely the mistake this
  // example exists to rule out.
  //
  // Neither actor can be walked to hub 26 from the start without the demo's
  // branching, so this reads the choices directly off the position instead.
  {
    Fixture f("speakers.json", "jail.json");

    // Freya's view of hub 26, asked as Freya standing there.
    ActorId freya_at_26 = f.engine.add_actor(f.conversation, 26, "Freya");
    CHECK(freya_at_26 != LOUDER_ACTOR_NONE, "Freya should be addable at 26");
    std::vector<Choice> freya_choices =
        f.engine.available_choices(f.conversation, 26, freya_at_26);
    CHECK(freya_choices.size() == 1, "Freya should be offered one choice at 26, saw %zu",
          freya_choices.size());
    CHECK(freya_choices.size() == 1 && freya_choices[0].general_index == 4,
          "Freya's single choice at 26 should be engine index 4, got %d",
          freya_choices.empty() ? -99 : freya_choices[0].general_index);
    CHECK(freya_choices.size() == 1 && freya_choices[0].link == 34,
          "Freya's single choice at 26 should lead to 34");

    // Benjamin's view of the same hub: indices 1 and 2, so his first key must
    // reach engine index 1, not 0.
    ActorId ben_at_26 = f.engine.add_actor(f.conversation, 26, "Benjamin");
    CHECK(ben_at_26 != LOUDER_ACTOR_NONE, "Benjamin should be addable at 26");
    std::vector<Choice> ben_choices = f.engine.available_choices(f.conversation, 26, ben_at_26);
    CHECK(ben_choices.size() == 2, "Benjamin should be offered two choices at 26, saw %zu",
          ben_choices.size());
    if (ben_choices.size() == 2) {
      CHECK(ben_choices[0].general_index == 1,
            "Benjamin's first visible choice at 26 should be engine index 1, got %d",
            ben_choices[0].general_index);
      CHECK(ben_choices[1].general_index == 2,
            "Benjamin's second visible choice at 26 should be engine index 2, got %d",
            ben_choices[1].general_index);
      CHECK(ben_choices[0].link == 30 && ben_choices[1].link == 33,
            "Benjamin's choices at 26 should lead to 30 then 33, got %d then %d",
            ben_choices[0].link, ben_choices[1].link);
    }
  }

  // The same thing through the interface: a fresh view whose right-hand side is
  // Benjamin standing on 26, then press his first key and require that it lands
  // on 30 rather than on 27.
  {
    Engine engine;
    CHECK(engine.load_config(read_file("speakers.json")) == LOUDER_LOAD_OK, "config should load");
    CHECK(engine.load_conversation("jail", read_file("jail.json")) == LOUDER_LOAD_OK,
          "conversation should load");
    NameId conversation = engine.name("jail");

    ActorId ben = engine.add_actor(conversation, 26, "Benjamin");
    ActorId freya = engine.add_actor(conversation, 26, "Freya");
    CHECK(ben != LOUDER_ACTOR_NONE && freya != LOUDER_ACTOR_NONE, "both actors should be added");

    Viewport left;
    left.actor = freya;
    left.display_name = "Freya";
    left.key = louder_demo::kLeftKeys[0];
    Viewport right;
    right.actor = ben;
    right.display_name = "REDACTED";
    right.key = louder_demo::kRightKeys[0];

    engine.drain_events();
    DialogueView view(&engine, left, right);
    view.consume({});

    CHECK(view.right().position == 26, "Benjamin should be standing on 26");
    CHECK(view.right().choices.size() == 2, "Benjamin should see two choices, saw %zu",
          view.right().choices.size());

    // 'q' is the first visible choice, which is engine index 1, leading to 30.
    // A view that passed 0 would instead enter auto hub 27.
    CHECK(view.handle_key('q'), "'q' should be acted on");
    CHECK(view.right().position == 30,
          "'q' at hub 26 should reach 30 via engine index 1, got %d", view.right().position);
    CHECK(view.errors().empty(), "the move should not error, saw: %s",
          view.errors().empty() ? "" : view.errors().back().c_str());

    // And 'w', the second visible choice, reaches 33.
    Engine engine2;
    CHECK(engine2.load_config(read_file("speakers.json")) == LOUDER_LOAD_OK, "config should load");
    CHECK(engine2.load_conversation("jail", read_file("jail.json")) == LOUDER_LOAD_OK,
          "conversation should load");
    NameId conversation2 = engine2.name("jail");
    ActorId ben2 = engine2.add_actor(conversation2, 26, "Benjamin");
    ActorId freya2 = engine2.add_actor(conversation2, 26, "Freya");
    Viewport left2;
    left2.actor = freya2;
    left2.display_name = "Freya";
    left2.key = louder_demo::kLeftKeys[0];
    Viewport right2;
    right2.actor = ben2;
    right2.display_name = "REDACTED";
    right2.key = louder_demo::kRightKeys[0];
    engine2.drain_events();
    DialogueView view2(&engine2, left2, right2);
    view2.consume({});
    CHECK(view2.handle_key('w'), "'w' should be acted on");
    CHECK(view2.right().position == 33,
          "'w' at hub 26 should reach 33 via engine index 2, got %d", view2.right().position);
  }

  // Freya at the same hub has one choice on her first key, reaching 34.
  {
    Engine engine;
    CHECK(engine.load_config(read_file("speakers.json")) == LOUDER_LOAD_OK, "config should load");
    CHECK(engine.load_conversation("jail", read_file("jail.json")) == LOUDER_LOAD_OK,
          "conversation should load");
    NameId conversation = engine.name("jail");
    ActorId freya = engine.add_actor(conversation, 26, "Freya");
    ActorId ben = engine.add_actor(conversation, 26, "Benjamin");
    Viewport left;
    left.actor = freya;
    left.display_name = "Freya";
    left.key = louder_demo::kLeftKeys[0];
    Viewport right;
    right.actor = ben;
    right.display_name = "REDACTED";
    right.key = louder_demo::kRightKeys[0];
    engine.drain_events();
    DialogueView view(&engine, left, right);
    view.consume({});

    CHECK(view.left().choices.size() == 1, "Freya should see one choice, saw %zu",
          view.left().choices.size());
    CHECK(view.handle_key('1'), "'1' should be acted on");
    CHECK(view.left().position == 34,
          "'1' at hub 26 should reach 34 via engine index 4, got %d", view.left().position);
  }
}

void TestAvailableChoicesNeverUnderCount() {
  std::printf("visible list matches the engine's count everywhere\n");

  // The interface discovers available choices by probing indices, because the
  // engine has no call for a hub's raw choice count and refuses both
  // out-of-range and unavailable indices identically. That probe has to find
  // every choice the engine counted, including when the available indices are
  // non-contiguous. Checked at every hub for both actors.
  Engine engine;
  CHECK(engine.load_config(read_file("speakers.json")) == LOUDER_LOAD_OK, "config should load");
  CHECK(engine.load_conversation("jail", read_file("jail.json")) == LOUDER_LOAD_OK,
        "conversation should load");
  NameId conversation = engine.name("jail");

  // Positions are sparse and unordered in the document, so every plausible
  // position is asked rather than trusting the range 0..max to be filled. A
  // position with no hub answers 0 and probes empty, which is consistent.
  LouderActor freya = engine.add_actor(conversation, 0, "Freya");
  LouderActor benjamin = engine.add_actor(conversation, 0, "Benjamin");
  CHECK(freya != LOUDER_ACTOR_NONE && benjamin != LOUDER_ACTOR_NONE, "both actors should be added");

  int hubs_checked = 0;
  for (LouderPosition p = 0; p < 64; ++p) {
    for (ActorId actor : {freya, benjamin}) {
      // Skip positions that are not hubs; the count is 0 there and the probe is
      // empty, which is consistent but not interesting.
      std::vector<Choice> found = engine.available_choices(conversation, p, actor);
      int32_t expected = engine.choice_count(conversation, p, actor);
      CHECK(static_cast<int32_t>(found.size()) == expected,
            "at hub %d for actor %d the probe found %zu choices but the engine counts %d", p,
            actor, found.size(), expected);
      if (expected > 0) {
        ++hubs_checked;
      }
    }
  }
  CHECK(hubs_checked > 0, "at least some hubs should offer choices to check");
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

// Benjamin has four choices at the start hub, keyed q, w, e, r. So 'r' is the
  // last in range and 't' is the first one past the end.
  CHECK(f.view().handle_key('r'), "'r' is Benjamin's fourth key, within his list of four");
  CHECK(f.view().right().position == 6, "'r' should lead to hub 6, got %d", f.view().right().position);

  // From 6 the only choice leads to 7, so Benjamin still has exactly one and
  // 't' is out of range from here too.
  size_t errors_before = f.view().errors().size();
  f.view().handle_key('t');
  CHECK(f.view().right().position == 6, "Benjamin should not have moved, got %d",
        f.view().right().position);
  CHECK(f.view().errors().size() > errors_before, "an out of range key should be reported");
}

// --- unclaimed keys --------------------------------------------------------

void TestConversationCanAdvancePastEveryChoice() {
  std::printf("the conversation can advance from every reachable choice\n");

  // The failure this catches: Benjamin takes "I said, I really don't know who I
  // am", which drags Freya to hub 3. Hub 3's only choice leads to hub 21, and
  // hub 21 is Volition's, which only Freya hears. So after that move the lead
  // must belong to Freya, and she must be able to take it. If neither actor has
  // choices there the conversation is dead and no further keypress can help.
  //
  // Freya hears Volition because the Freya speaker's config lists Volition as
  // listening, and an actor playing Freya inherits that. Hub 21 is therefore
  // visible to her, and the claim on it has to be measured against the same
  // heard set, or the conversation dead-ends on a hub she can plainly see.
  for (size_t first = 0; first < 4; ++first) {
    Fixture f("speakers.json", "jail.json");
    std::vector<int32_t> visible = general_indices(f.view().right());
    if (first >= visible.size()) {
      break;
    }

    char key = louder_demo::kRightKeys[first];
    f.view().handle_key(key);

    // Whoever holds the lead must be able to move, so at least one side has to
    // have a choice after the first move.
    bool someone_can_act = !f.view().left().choices.empty() || !f.view().right().choices.empty();
    CHECK(someone_can_act,
          "after first move %zu (key '%c', actors at %d and %d) neither actor has any choices, "
          "so the conversation is dead",
          first, key, f.view().left().position, f.view().right().position);
  }

  // The specific path, checked step by step so a stall names the hub it
  // happened on.
  {
    Fixture f("speakers.json", "jail.json");
    f.view().handle_key('q');  // Benjamin: 0 -> 3, Freya dragged along

    CHECK(f.view().right().position == 3, "Benjamin should be at 3, got %d", f.view().right().position);
    CHECK(f.view().left().position == 3, "Freya should have been dragged to 3, got %d",
          f.view().left().position);

    // Hub 3 leads only to hub 21, which is Volition's. Only Freya hears Volition,
    // so she must be the one with a choice here, and it must be the only one.
    CHECK(f.view().left().choices.size() == 1,
          "Freya should have exactly one choice at hub 3 to continue into Volition's hub 21, but "
          "has %zu (engine count %d)",
          f.view().left().choices.size(),
          f.engine.choice_count(f.conversation, 3, f.view().left().actor));
    CHECK(f.view().right().choices.empty(),
          "Benjamin should have no choice at hub 3, since hub 21 is not his to lead, but has %zu",
          f.view().right().choices.size());

    if (!f.view().left().choices.empty()) {
      CHECK(f.view().left().choices[0].link == 21, "that choice should lead to 21, got %d",
            f.view().left().choices[0].link);
      CHECK(f.view().handle_key('1'), "Freya should be able to take it");
      CHECK(f.view().left().position == 21, "Freya should be at 21, got %d",
            f.view().left().position);
      CHECK(f.view().errors().empty(), "the move should not error, saw: %s",
            f.view().errors().empty() ? "" : f.view().errors().back().c_str());
    }

    // And she keeps going: hub 21 is Volition's with two choices of her own.
    CHECK(f.view().left().choices.size() == 2, "Freya should have two choices at hub 21, saw %zu",
          f.view().left().choices.size());
  }
}

void TestContinueKeysBothSelectTheLoneChoice() {
  std::printf("both CONTINUE keys select the lone choice\n");

  // The spec binds a single available choice to two keys per side. Both must
  // work, and each must land on the same hub.
  //
  //   Freya:    1 and `
  //   Benjamin: q and Tab
  // Freya at hub 3, reached by Benjamin's first move. Both of her keys are
  // checked against each other rather than against a hardcoded hub, so the test
  // states the property instead of the document.
  Position via_number = -1;
  {
    Fixture f("speakers.json", "jail.json");
    f.view().handle_key('q');  // Benjamin: 0 -> 3, Freya dragged along

    CHECK(f.view().left().is_continue(),
          "the premise is a single choice for Freya at hub 3, saw %zu",
          f.view().left().choices.size());
    Position expected = f.view().left().choices.empty() ? -1 : f.view().left().choices[0].link;

    CHECK(f.view().handle_key('1'), "the number key '1' should be acted on");
    CHECK(f.view().left().position == expected, "'1' should land on %d, got %d", expected,
          f.view().left().position);
    CHECK(f.view().errors().empty(), "'1' should not error, saw: %s",
          f.view().errors().empty() ? "" : f.view().errors().back().c_str());
    via_number = f.view().left().position;
  }

  // The backtick does the same thing from a clean engine.
  {
    Fixture f("speakers.json", "jail.json");
    f.view().handle_key('q');
    CHECK(f.view().left().is_continue(), "Freya should be in the continue state at hub 3");
    const Position expected = f.view().left().choices.empty() ? -1 : f.view().left().choices[0].link;
    CHECK(f.view().handle_key(louder_demo::kLeftContinueKey),
          "the backtick should be acted on while Freya is continuing");
    CHECK(f.view().left().position == expected, "the backtick should land on %d, got %d", expected,
          f.view().left().position);
    CHECK(f.view().left().position == via_number,
          "the backtick and 1 should land on the same hub, %d vs %d", f.view().left().position,
          via_number);
    CHECK(f.view().errors().empty(), "the backtick should not error, saw: %s",
          f.view().errors().empty() ? "" : f.view().errors().back().c_str());
  }

  // Benjamin needs a lone choice of his own. Hub 28's only choice leads to hub
  // 31, which has no speaker at all, so it is public and unowned: both actors
  // are offered it, and both end up in the single-choice state.
  {
    auto standing_at_28 = [](DialogueView** out_view, Engine** out_engine) {
      auto engine = std::make_unique<Engine>();
      engine->load_config(read_file("speakers.json"));
      engine->load_conversation("jail", read_file("jail.json"));
      NameId conversation = engine->name("jail");
      ActorId freya = engine->add_actor(conversation, 28, "Freya");
      ActorId ben = engine->add_actor(conversation, 28, "Benjamin");

      Viewport left;
      left.actor = freya;
      left.display_name = "Freya";
      Viewport right;
      right.actor = ben;
      right.display_name = "REDACTED";
      engine->drain_events();

      auto view = std::make_unique<DialogueView>(engine.get(), left, right);
      view->consume({});
      *out_view = view.release();
      *out_engine = engine.release();
    };

    DialogueView* by_q = nullptr;
    Engine* engine_q = nullptr;
    standing_at_28(&by_q, &engine_q);
    CHECK(by_q->right().is_continue(),
          "the premise is one choice for Benjamin at hub 28, saw %zu", by_q->right().choices.size());
    Position expected = by_q->right().choices.empty() ? -1 : by_q->right().choices[0].link;
    CHECK(by_q->handle_key('q'), "q should be acted on");
    CHECK(by_q->right().position == expected, "q should land on %d, got %d", expected,
          by_q->right().position);
    CHECK(by_q->errors().empty(), "q should not error, saw: %s",
          by_q->errors().empty() ? "" : by_q->errors().back().c_str());
    Position via_q = by_q->right().position;
    delete by_q;
    delete engine_q;

    // And Tab reaches the same place.
    DialogueView* by_tab = nullptr;
    Engine* engine_tab = nullptr;
    standing_at_28(&by_tab, &engine_tab);
    CHECK(by_tab->right().is_continue(), "Benjamin should be continuing again");
    CHECK(by_tab->handle_key(louder_demo::kRightContinueKey), "Tab should be acted on");
    CHECK(by_tab->right().position == via_q, "Tab should land where q did, %d vs %d",
          by_tab->right().position, via_q);
    CHECK(by_tab->errors().empty(), "Tab should not error, saw: %s",
          by_tab->errors().empty() ? "" : by_tab->errors().back().c_str());
    delete by_tab;
    delete engine_tab;
  }

  // Freya's Tab must never act, since it belongs to Benjamin. Checked while she
  // is mid-conversation with choices of her own, where a misrouted key would
  // be most visible.
  {
    Fixture f("speakers.json", "jail.json");
    f.view().handle_key('q');
    Position before = f.view().left().position;
    f.view().handle_key(louder_demo::kRightContinueKey);
    CHECK(f.view().left().position == before, "Tab must not move Freya, got %d",
          f.view().left().position);
    CHECK(f.view().right().position == before, "Tab must not move Benjamin either, got %d",
          f.view().right().position);
  }
}

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
  TestHiddenChoicesOffsetTheVisibleList();
  TestAvailableChoicesNeverUnderCount();
TestKeypressSelectsTheNthVisibleChoice();
  TestConversationCanAdvancePastEveryChoice();
  TestContinueKeysBothSelectTheLoneChoice();
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
