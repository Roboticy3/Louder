// A thin, testable wrapper around the Louder C ABI.
//
// The engine interns strings as opaque integer ids, so anything that crosses
// this boundary as a LouderName is kept as an id here and only turned back into
// text at the edges. Everything above this header deals in std::string and
// plain integers.
#pragma once

#include <cstdint>
#include <string>
#include <vector>

#include "louder_rust.h"

namespace louder_demo {

// An interned string id. LOUDER_NAME_NONE (-1) means "no such name".
using NameId = int32_t;
using Position = int32_t;
using ActorId = int32_t;

// The engine's "no such position". The Rust side calls this BLANK_POSITION and
// returns it as -1 from every position query, but the generated header does not
// export the constant, so it is restated here rather than spelled out inline.
constexpr Position kNoPosition = -1;

// One choice as this actor sees it.
//
// `general_index` is the index into the hub's raw choice list, which is what the
// engine wants. It is usually not the same as the choice's position in the
// actor's visible list, which is what the interface numbers on screen.
struct Choice {
  int32_t general_index = 0;
  int32_t link = kNoPosition;
  std::string text;
};

// Reads ids back to text. Owns the engine handle for its lifetime.
class Engine {
 public:
  Engine();
  ~Engine();

  Engine(const Engine&) = delete;
  Engine& operator=(const Engine&) = delete;

  // True when the engine handle came up. Checked once, before anything else.
  bool ready() const { return ctx_ != nullptr; }

  // --- strings -------------------------------------------------------------

  // The id for `text`, creating it if the engine has not seen it. Never returns
  // LOUDER_NAME_NONE for a valid string: the loaders intern some names before
  // the host ever asks for them, and declare_name reports "already exists" by
  // returning none, so a lookup has to come first.
  NameId name(const std::string& text);

  // The text behind an id, or "" for none.
  std::string text_of(NameId name);

  // --- loading -------------------------------------------------------------

  // Load a conversation document under the name `name`. Returns the engine's
  // LoadError, where 0 is success.
  LoadError load_conversation(const std::string& name, const std::string& document);

  // Load the speaker config. Returns the engine's LoadError, where 0 is
  // success.
  LoadError load_config(const std::string& document);

  // --- the conversation graph ----------------------------------------------

  // The lowest position tagged `tag`, or -1 if no hub carries it.
  Position find_tag(const std::string& tag);

  std::string speaker_of(Position position);
  std::string text_at(Position position);
  bool is_auto(Position position);

  // Every choice on `position` that `actor` could actually take, as
  // (general choice index, text) in ascending index order.
  //
  // The engine indexes choices by their position in the hub's raw choice list,
  // and hides the ones this actor may not take. So the visible list is usually
  // shorter than the hub's real list and its indices do not start at zero. The
  // caller maps a 1-based display number onto this list; it must not pass the
  // display number to select_choice directly.
  std::vector<Choice> available_choices(NameId conversation, Position position, ActorId actor);

  // The number of choices the engine believes are available, for cross-checking
  // available_choices.
  int32_t choice_count(NameId conversation, Position position, ActorId actor);

  // The hub the choice at `general_index` leads to, or BLANK_POSITION if that
  // choice does not exist or is not available to this actor. Exposed so tests
  // can check the view's links against the engine's own answer.
  int32_t choice_link(NameId conversation, Position position, ActorId actor, int32_t general_index);

  // --- actors --------------------------------------------------------------

  // Add an actor speaking as `speaker`, starting at `start`. Returns its id, or
  // LOUDER_ACTOR_NONE on failure.
  ActorId add_actor(NameId conversation, Position start, const std::string& speaker);

  Position position_of(NameId conversation, ActorId actor);
  std::string channel_of(NameId conversation, ActorId actor);

// Select the choice at a *general* choice index. Returns true on success.
  bool select_choice(NameId conversation, ActorId actor, int32_t general_choice_index);

  // Move an actor without it choosing. Returns true on success.
  //
  // Used to reach positions the conversation will not walk to on its own, and
  // to confirm that a hub the engine considers impassable is refused here too
  // rather than only being refused through the choice rules.
  bool force_position(NameId conversation, ActorId actor, Position target);

  // --- events --------------------------------------------------------------

  // Everything the engine has queued since the last drain, then clears the
  // queue. Order is the order the engine produced them in, which matters: a
  // hub appearance is always followed by the appearances of its choices.
  std::vector<LouderEvent> drain_events();

 private:
  LouderContext* ctx_ = nullptr;

  // Small memo of ids already resolved, so the hot path (rendering a frame)
  // does not re-hash every string it touches.
  std::vector<std::pair<std::string, NameId>> name_cache_;

  NameId lookup_or_declare(const std::string& text);
};

}  // namespace louder_demo
