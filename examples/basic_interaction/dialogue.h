// The two-column dialogue this demo shows.
//
// There is deliberately no wrapper object around the engine. `louder_rust.h` is
// the interface being demonstrated, so it is called directly and its ids and
// names are used as they are. What lives here is only what the engine does not
// do: which key belongs to which column, and how two actors' views of one
// conversation are laid out side by side.
//
// Everything is plain data plus four methods, so that interface_test.cpp can
// drive a whole session without a terminal.
#pragma once

#include <string>
#include <vector>

#include "louder_rust.h"
#include "terminal.h"

// Freya takes 1..9 and Benjamin the qwerty block, as the spec describes.
constexpr char kFreyaKeys[] = "123456789";
constexpr char kBenjaminKeys[] = "qwertyuiop";

// A side with exactly one choice is not deciding anything, so the spec asks for
// that choice to be presented as CONTINUE and bound to both of that side's
// keys: 1 and ` for Freya, q and Tab for Benjamin.
//
// The extra key only means anything in that state. Pressing it while the side
// has several choices is ignored rather than reported, since it is a
// context-sensitive key and not a mistyped one.
constexpr char kFreyaContinueKey = '`';
constexpr char kBenjaminContinueKey = '\t';
constexpr const char* kContinueLabel = "CONTINUE";
constexpr const char* kFreyaContinueKeyName = "backtick (`)";
constexpr const char* kBenjaminContinueKeyName = "Tab";

// The interned id for `text`, creating it if the engine has not seen it.
//
// declare_name reports "already exists" by returning none, and the loaders
// intern some names before the host ever asks for them, so a lookup has to come
// first. Never returns LOUDER_NAME_NONE for a string the engine can hold.
LouderName intern(LouderContext* ctx, const std::string& text);

// The text behind an id, or "" for none.
std::string text_of(LouderContext* ctx, LouderName name);

// One choice as this actor sees it.
struct Choice {
  // The index into the hub's full choice list, which is what select_choice
  // wants. It is usually not the position in the actor's visible list, which is
  // what the interface numbers on screen.
  int32_t general_index = 0;

  // The hub behind the choice.
  LouderPosition link = -1;

  // The rule that made this choice available, as a ChoiceRule value.
  int32_t rule = 0;

  std::string text;
};

// One column of the screen: everything shown for a single actor.
struct Column {
  // The engine actor behind this column.
  LouderActor actor = LOUDER_ACTOR_NONE;

  // The name in the column header. This is the alias, not the speaker: the demo
  // shows Benjamin as REDACTED to both parties even though the engine reports
  // his real name.
  const char* display_name = "";

  // The keys this side presses, and the second key that selects its lone choice
  // when it has one.
  const char* keys = kFreyaKeys;
  char continue_key = kFreyaContinueKey;
  const char* continue_key_name = kFreyaContinueKeyName;

  // The hub the actor is standing on, and whether it can be read at all.
  bool in_conversation = false;
  LouderPosition position = -1;

  // Who the engine says is speaking there, and what it says they said.
  std::string speaker;
  std::string text;

  // The choices this actor may take, in visible order.
  std::vector<Choice> choices;

  // The engine's last refusal on this side, shown until something works.
  std::string error;

  // True when this side has exactly one choice and is therefore showing
  // CONTINUE rather than the choice's own text.
  bool is_continue() const { return choices.size() == 1; }

  // The key shown beside visible choice `index`.
  char key_for(size_t index) const {
    return index < std::char_traits<char>::length(keys) ? keys[index] : '?';
  }
};

// The whole screen: two columns over one conversation.
struct View {
  LouderContext* ctx = nullptr;
  LouderName conversation = LOUDER_NAME_NONE;

  Column left;
  Column right;

  // The most recent callback name, or "" if none has fired. The footer reports
  // it so that a one-shot callback firing once is observable.
  std::string last_callback;

  // Point the view at a loaded conversation and the two actors on it.
  void setup(LouderContext* engine, LouderName conversation_name,
             LouderActor left_actor, LouderActor right_actor);

  // Re-read both columns from the engine: position, speaker, text, and the
  // choices available now. The engine decides who may decide; this reports its
  // answer rather than assuming anything.
  void refresh();

  // Offer one keypress to the screen. Returns true when the key belonged to one
  // of the columns and was acted on.
  bool press(char key);

  // Draw a frame of the given console size.
  void draw(Terminal& terminal) const;
};
