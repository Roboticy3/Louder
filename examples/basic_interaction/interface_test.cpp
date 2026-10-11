// Interface tests for the Bare Minimum demo.
//
// These drive the real engine and the real view, but never touch a terminal:
// Terminal is not constructed and View::draw is not called. That split is the
// point, because everything interesting about the interface is the mapping from
// keys to engine choice indices and the per-character visibility, not the escape
// sequences.
//
// What is left here is only what belongs to the interface. Anything about how
// the engine decides visibility, auto hubs, warping or private hubs is an engine
// rule and lives in engine/tests/engine_test.cpp.

#include <cstdio>
#include <fstream>
#include <sstream>
#include <string>

#include "dialogue.h"

namespace {

int g_checks = 0;
int g_failures = 0;

#define CHECK(condition, ...)                            \
  do {                                                   \
    ++g_checks;                                          \
    if (!(condition)) {                                  \
      ++g_failures;                                      \
      std::printf("  FAIL %s:%d: ", __FILE__, __LINE__); \
      std::printf(__VA_ARGS__);                          \
      std::printf("\n");                                 \
    }                                                    \
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

// A loaded conversation with two actors standing on a hub.
//
// `at` exists because two of the rules below are only visible from somewhere
// the demo does not naturally walk: hub 26 is private to Benjamin, and hub 28's
// single choice is the one that makes both sides continuing.
struct Fixture {
  LouderContext* ctx = init_louder();
  LouderName conversation = LOUDER_NAME_NONE;
  View view;
  LouderPosition start = -1;

  explicit Fixture(LouderPosition at = -1) {
    std::string config = read_file("speakers.json");
    std::string document = read_file("jail.json");
    // The loaders take a mutable buffer; the documents here are not modified.
    auto mutable_bytes = [](std::string& text) {
      return reinterpret_cast<uint8_t*>(const_cast<char*>(text.data()));
    };

    CHECK(config.empty() == false, "the speaker config should be readable");
    CHECK(load_conversation_config(ctx, mutable_bytes(config), config.size(),
                                   LOUDER_ENCODING_UTF8) == LOUDER_LOAD_OK,
          "the speaker config should load");

    conversation = intern(ctx, "jail");
    CHECK(load_conversation_json(ctx, conversation, mutable_bytes(document), document.size(),
                                 LOUDER_ENCODING_UTF8) == LOUDER_LOAD_OK,
          "the conversation should load");

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

  ~Fixture() { deinit_louder(ctx); }

  const Column& left() const { return view.left; }
  const Column& right() const { return view.right; }
};

// --- the choice index the interface must not confuse with the engine's ------

// A choice index in the UI is not the index the engine wants. Hub 26 carries
// five raw choices and offers Benjamin only four of them, so his visible list
// and the engine's indices agree only by accident; hub 26 is also private to
// Benjamin, whose column owns it while Freya's does not.
//
// This is the one rule the example exists to demonstrate, so it is checked at
// the hub where it bites rather than at a hub where nothing is hidden.
void TestVisibleChoiceIndicesAreNotEngineIndices() {
  std::printf("visible index vs engine index\n");
  Fixture f(26);

  CHECK(f.left().choices.empty(),
        "the speaker of a private hub does not own it, so Freya should get nothing, saw %zu",
        f.left().choices.size());
  CHECK(f.right().choices.size() == 4, "Benjamin should see four choices at 26, saw %zu",
        f.right().choices.size());

  // Each displayed choice must carry the engine's own link for the engine index
  // it names, and that index is not simply the position on screen. Four of the
  // hub's five raw choices are offered; the fifth is not.
  for (size_t i = 0; i < f.right().choices.size(); ++i) {
    const Choice& choice = f.right().choices[i];
    const LouderPosition link =
        get_hub_choice_link(f.ctx, f.conversation, 26, f.right().actor, choice.general_index);
    CHECK(choice.link == link, "visible choice %zu should lead to %d, got %d", i, link,
          choice.link);
  }
  CHECK(get_hub_choice_link(f.ctx, f.conversation, 26, f.right().actor, 4) < 0,
        "the fifth raw choice leads to a hub that is not Benjamin's, so it is refused");

  // His first visible choice is engine index 0, which leads into auto hub 27.
  // Auto hubs resolve themselves, so he lands beyond it: at hub 29, which only
  // he hears.
  CHECK(f.view.press('q'), "'q' should be acted on");
  CHECK(f.right().position == 29, "'q' should enter auto hub 27 and come out at 29, got %d",
        f.right().position);
  CHECK(f.right().error.empty(), "the move should not error, got '%s'", f.right().error.c_str());

  // Freya is carried through the same auto hub, but she does not hear hub 29,
  // so auto logic sends her to the other branch: hub 28. The two actors end up
  // apart, which is how his private thought stays private.
  CHECK(f.left().position == 28, "Freya should diverge onto hub 28, got %d", f.left().position);
}

// --- keys ------------------------------------------------------------------

// Freya takes 1..9 and Benjamin the qwerty block, and no key may answer to both,
// or one press would be ambiguous.
void TestKeysBelongToOneColumn() {
  std::printf("keys belong to one column\n");
  Fixture f;

  for (int i = 0; kFreyaKeys[i] != '\0'; ++i) {
    CHECK(f.left().key_for(static_cast<size_t>(i)) == kFreyaKeys[i],
          "Freya's choice %d should be on '%c'", i, kFreyaKeys[i]);
  }
  for (int i = 0; kBenjaminKeys[i] != '\0'; ++i) {
    CHECK(f.right().key_for(static_cast<size_t>(i)) == kBenjaminKeys[i],
          "Benjamin's choice %d should be on '%c'", i, kBenjaminKeys[i]);
  }

  // Pressing a key past the end of what is on offer is reported, not ignored:
  // the player typed something and deserves to know it did nothing.
  CHECK(!f.view.press('z'), "'z' belongs to neither column");
  CHECK(f.view.press('1'), "'1' is Freya's key even with nothing behind it");
  CHECK(!f.left().error.empty(), "so the refusal should be shown to her");
  CHECK(f.left().position == f.start, "and she should not have moved, got %d", f.left().position);
}

// The spec binds a lone choice to two keys per side, and both must land in the
// same place. Hub 28's only choice is public and unowned, which is what leaves
// both sides continuing there.
void TestLoneChoiceAnswersToBothOfItsKeys() {
  std::printf("a lone choice answers to both keys\n");

  // Freya continues on '1' and on backtick.
  for (const char* key : {"1", "`"}) {
    Fixture f(28);
    CHECK(f.left().is_continue(), "the premise is one choice for Freya, saw %zu",
          f.left().choices.size());
    CHECK(f.view.press(key[0]), "Freya's '%s' should be acted on", key);
    CHECK(f.left().error.empty(), "'%s' should not error, got '%s'", key, f.left().error.c_str());
    CHECK(f.left().position == 31, "'%s' should land on 31, got %d", key, f.left().position);
  }

  // Benjamin continues on 'q' and on Tab.
  for (const char* key : {"q", "\t"}) {
    Fixture f(28);
    CHECK(f.right().is_continue(), "the premise is one choice for Benjamin, saw %zu",
          f.right().choices.size());
    CHECK(f.view.press(key[0]), "Benjamin's '%s' should be acted on", key);
    CHECK(f.right().error.empty(), "'%s' should not error, got '%s'", key, f.right().error.c_str());
    CHECK(f.right().position == 31, "'%s' should land on 31, got %d", key, f.right().position);
  }

  // The second key only means something while the side is continuing. Pressing
  // it when there is a real choice to make is context-sensitive, so it is
  // ignored rather than reported as a mistyped choice.
  Fixture f;
  CHECK(f.right().choices.size() > 1, "the premise is a real choice for Benjamin to make");
  CHECK(!f.view.press('\t'), "Tab should do nothing while Benjamin has several choices");
}

// --- aliasing --------------------------------------------------------------

// Benjamin is REDACTED to both parties even though the engine reports his real
// name. The substitution is the interface's job, so the two must stay separate.
void TestTheAliasIsTheInterfacesOwn() {
  std::printf("aliasing\n");
  Fixture f;

  CHECK(std::string(f.right().display_name) == "REDACTED",
        "Benjamin's column should be titled REDACTED");
  CHECK(std::string(f.left().display_name) == "Freya", "Freya's column should be titled Freya");

  // The start hub is Freya's speech, so both columns show the same speaker.
  CHECK(f.right().speaker == "Freya", "the start hub is spoken by Freya whoever reads it, got '%s'",
        f.right().speaker.c_str());

  // After a move of his own, the engine reports Benjamin, and only the engine
  // does: the column header is untouched by it.
  CHECK(f.view.press('q'), "Benjamin should be able to lead from the start hub");
  CHECK(f.right().speaker == "Benjamin", "the engine should report the real speaker, got '%s'",
        f.right().speaker.c_str());
  CHECK(std::string(f.right().display_name) == "REDACTED",
        "and the column header should still be the alias");
}

// --- the columns report the engine, never their own opinion ---------------

// The interface decides nothing about who may see or choose what. It asks, and
// shows the answer. Whatever the engine does to the two actors, the columns
// must agree with it afterwards, so this is checked after every single move.
void TestTheColumnsAlwaysAgreeWithTheEngine() {
  std::printf("the columns always agree with the engine\n");
  Fixture f;

  for (int step = 0; step < 12; ++step) {
    for (const Column* column : {&f.left(), &f.right()}) {
      CHECK(column->position == get_actor_position(f.ctx, f.conversation, column->actor),
            "step %d: %s is shown at %d but the engine says %d", step, column->display_name,
            column->position, get_actor_position(f.ctx, f.conversation, column->actor));

      const int32_t expected = get_hub_choice_count(f.ctx, f.conversation, column->position,
                                                    column->actor);
      CHECK(expected == static_cast<int32_t>(column->choices.size()),
            "step %d: %s is shown with %zu choices but the engine counts %d", step,
            column->display_name, column->choices.size(), expected);

      // Every displayed choice must lead where it says, by the engine's own
      // answer for that index.
      for (size_t i = 0; i < column->choices.size(); ++i) {
        const Choice& choice = column->choices[i];
        CHECK(choice.link == get_hub_choice_link(f.ctx, f.conversation, column->position,
                                                 column->actor, choice.general_index),
              "step %d: %s's choice %zu shows %d, the engine says %d", step, column->display_name,
              i, choice.link, get_hub_choice_link(f.ctx, f.conversation, column->position,
                                                  column->actor, choice.general_index));
      }
    }

    // Alternate sides, so both get driven, and stop when neither can move.
    const Column* column = (step % 2 == 0) ? &f.right() : &f.left();
    if (column->choices.empty()) {
      column = column == &f.left() ? &f.right() : &f.left();
    }
    if (column->choices.empty()) {
      break;
    }
    f.view.press(column->keys[0]);
  }
}

// --- the conversation can actually be played -------------------------------

// A demo nobody can finish is not a demo. Whatever either player picks first,
// the conversation must not stall, and a side with nothing to decide must be
// able to say so.
void TestEitherSideCanCarryTheConversation() {
  std::printf("the conversation does not stall\n");

  for (size_t first = 0; first < 4; ++first) {
    Fixture f;
    if (first >= f.right().choices.size()) {
      break;
    }

    f.view.press(f.right().keys[first]);
    CHECK(f.right().position != f.start, "the first move %zu should have gone somewhere", first);
    CHECK(!f.left().choices.empty() || !f.right().choices.empty(),
          "after first move %zu neither actor has any choices, so the conversation is dead", first);
    CHECK(f.left().error.empty() && f.right().error.empty(),
          "and nothing should have been refused, got '%s' / '%s'", f.left().error.c_str(),
          f.right().error.c_str());
  }

  // The concrete case that would otherwise dead-end: Benjamin's first choice
  // drags Freya to hub 3, whose only exit is Volition's hub 21. Volition is
  // Freya's, so the lead passes to her and she must be able to take it.
  Fixture f;
  f.view.press('q');
  CHECK(f.left().position == 3, "Freya should have been carried to 3, got %d", f.left().position);
  CHECK(f.right().choices.empty(), "hub 21 is not Benjamin's to lead, so he should have none, saw %zu",
        f.right().choices.size());
  CHECK(f.left().choices.size() == 1, "Freya should have exactly one way on, saw %zu",
        f.left().choices.size());
  if (f.left().choices.size() == 1) {
    CHECK(f.view.press('1'), "Freya should be able to take it");
    CHECK(f.left().position == 21, "and it should lead to 21, got %d", f.left().position);
  }
}

}  // namespace

int main() {
  std::printf("running interface tests\n");

  TestVisibleChoiceIndicesAreNotEngineIndices();
  TestKeysBelongToOneColumn();
  TestLoneChoiceAnswersToBothOfItsKeys();
  TestTheAliasIsTheInterfacesOwn();
  TestTheColumnsAlwaysAgreeWithTheEngine();
  TestEitherSideCanCarryTheConversation();

  std::printf("\n%d checks, %d failures\n", g_checks, g_failures);
  return g_failures == 0 ? 0 : 1;
}
