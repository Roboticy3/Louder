// The two-column dialogue view.
//
// This is deliberately free of terminal and engine concerns. It takes key
// presses and engine events in, and produces a plain description of what each
// column should show. Everything testable about the interface is testable here
// without a terminal.
#pragma once

#include <string>
#include <vector>

#include "engine_bridge.h"

namespace louder_demo {

// The keys each side of the screen answers to.
//
// Freya takes 1..9 and Benjamin takes the qwerty block, as the spec describes.
// Both are indexed from 0, so Freya's first choice is on "1" and Benjamin's is
// on "q".
constexpr char kLeftKeys[] = {'1', '2', '3', '4', '5', '6', '7', '8', '9'};
constexpr char kRightKeys[] = {'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'};

// A side with exactly one choice available is not making a decision, so the spec
// asks for that choice to be presented as CONTINUE and bound to both of that
// side's keys: 1 and ` for Freya, q and Tab for Benjamin.
//
// These keys only mean anything in that state. Pressing them while the side has
// several choices is ignored rather than reported, since they are context
// sensitive and not really mistyped.
constexpr char kLeftContinueKey = '`';
constexpr char kRightContinueKey = '\t';

// What the single available choice is labelled on screen, and how the extra key
// is named to the player. The backtick and the word "backtick" both read badly,
// so it is spelled out.
constexpr const char* kContinueLabel = "CONTINUE";
constexpr const char* kLeftContinueKeyName = "backtick (`)";
constexpr const char* kRightContinueKeyName = "Tab";

// What a column needs to know about its owner, which the engine supplies.
struct Viewport {
  // The engine actor id behind this column.
  ActorId actor = LOUDER_ACTOR_NONE;

  // The name shown in the column header. This is the alias, not the speaker
  // name: Benjamin is displayed as REDACTED everywhere, including to Freya.
  std::string display_name;

  // The key this side presses, for display.
  char key = '?';

  // Set when the last hub this actor saw was a background appearance, meaning
  // another actor's choice happened elsewhere in the channel.
  bool background = false;

  // The hub the actor is standing on, and whether it can currently be read.
  bool has_position = false;
  Position position = kNoPosition;

  std::string speaker;
  std::string text;

  // The choices this actor may take, in visible order.
  std::vector<Choice> choices;

// Set when the engine refused the last choice this actor tried to make.
  std::string error;

  // The extra key bound to this side's CONTINUE choice, and its name for the
  // player. Empty when the side is not in the single-choice state.
  char continue_key = '\0';
  std::string continue_key_name;

  // True when this side has exactly one choice available and is therefore
  // showing CONTINUE rather than the choice's own text.
  bool is_continue() const { return choices.size() == 1; }

  // This side's number keys, so that rendering knows which block to label
  // choices with. Freya's column uses 1..9 and Benjamin's uses q..p.
  const char* number_keys = kLeftKeys;
  size_t number_key_count = sizeof(kLeftKeys) - 1;

  // The key that selects visible choice `index`, numbering the visible list.
  char key_for(size_t index) const {
    return index < number_key_count ? number_keys[index] : '?';
  }
};

// One thing that happened, in a form the view can react to without knowing the
// ABI.
struct Line {
  ActorId actor = LOUDER_ACTOR_NONE;

  enum class Kind {
    // The actor arrived at a hub.
    Arrival,
    // The actor is being shown a hub in the background.
    Background,
    // The actor's channel changed.
    Channel,
    // A callback fired. Not displayed in the columns; surfaced in the footer.
    Callback,
    // Something the engine could not do.
    Error,
  };

  Kind kind = Kind::Arrival;
  Position position = kNoPosition;
  std::string text;
};

// The whole screen. Two columns plus the shared status lines.
class DialogueView {
 public:
  DialogueView(Engine* engine, Viewport left, Viewport right);

  // Fold a batch of engine events into the view state and return the lines they
  // produced, in order.
  std::vector<Line> consume(const std::vector<LouderEvent>& events);

  // Re-read both columns from the engine: position, speaker, text, and the
  // choices currently available. Called after every batch of events, because the
  // engine decides visibility itself and this only reports on its answer.
  void refresh();

  // Feed one keypress. Returns true when it belonged to one of the two columns
  // and was acted on.
  bool handle_key(char key);

  const Viewport& left() const { return left_; }
  const Viewport& right() const { return right_; }

  // The engine errors worth showing the user, most recent last.
  const std::vector<std::string>& errors() const { return errors_; }

  // The most recent callback name, or "" if none has fired.
  const std::string& last_callback() const { return last_callback_; }

  // The conversation this view is showing.
  NameId conversation() const { return conversation_; }

  // Which side a key belongs to, if any.
  static int side_for_key(char key);
  static int key_index_for(char key);

 private:
  // The engine actor behind a column, searched by id so that a ChoiceAppearance
  // event, which names an actor rather than a side, can be routed.
  Viewport* viewport_for(ActorId actor);

  // Record an error against a column and the shared error list.
  void fail(Viewport& viewport, const std::string& message);

  Engine* engine_ = nullptr;
  NameId conversation_ = LOUDER_NAME_NONE;
  Viewport left_;
  Viewport right_;
  std::vector<std::string> errors_;
  std::string last_callback_;
};

}  // namespace louder_demo
