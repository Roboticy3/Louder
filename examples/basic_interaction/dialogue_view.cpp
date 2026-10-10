#include "dialogue_view.h"

#include <algorithm>

namespace louder_demo {

namespace {

// The motion a RegularMotion / Warp / ForceMotion event carries.
const Motion& motion_of(const LouderEvent& event) {
  switch (event.data.tag) {
    case EventData::Tag::Warp:
      return event.data.warp._0;
    case EventData::Tag::ForceMotion:
      return event.data.force_motion._0;
    default:
      return event.data.regular_motion._0;
  }
}

}  // namespace

DialogueView::DialogueView(Engine* engine, Viewport left, Viewport right)
    : engine_(engine), left_(std::move(left)), right_(std::move(right)) {
  conversation_ = engine_->name("jail");

  // Each column gets its own number keys and its own CONTINUE key, so the two
  // sides can never both answer to the same key.
  left_.number_keys = kLeftKeys;
  left_.number_key_count = sizeof(kLeftKeys) - 1;
  left_.continue_key = kLeftContinueKey;
  left_.continue_key_name = kLeftContinueKeyName;

  right_.number_keys = kRightKeys;
  right_.number_key_count = sizeof(kRightKeys) - 1;
  right_.continue_key = kRightContinueKey;
  right_.continue_key_name = kRightContinueKeyName;
}

int DialogueView::side_for_key(char key) {
  for (size_t i = 0; i < sizeof(kLeftKeys); ++i) {
    if (key == kLeftKeys[i]) {
      return 0;
    }
  }
  for (size_t i = 0; i < sizeof(kRightKeys); ++i) {
    if (key == kRightKeys[i]) {
      return 1;
    }
  }
  // The CONTINUE keys belong to a fixed side each, so a Tab is never ambiguous.
  if (key == kLeftContinueKey) {
    return 0;
  }
  if (key == kRightContinueKey) {
    return 1;
  }
  return -1;
}

int DialogueView::key_index_for(char key) {
  for (size_t i = 0; i < sizeof(kLeftKeys); ++i) {
    if (key == kLeftKeys[i]) {
      return static_cast<int>(i);
    }
  }
  for (size_t i = 0; i < sizeof(kRightKeys); ++i) {
    if (key == kRightKeys[i]) {
      return static_cast<int>(i);
    }
  }
  return -1;
}

Viewport* DialogueView::viewport_for(ActorId actor) {
  if (left_.actor == actor) {
    return &left_;
  }
  if (right_.actor == actor) {
    return &right_;
  }
  return nullptr;
}

void DialogueView::fail(Viewport& viewport, const std::string& message) {
  viewport.error = message;
  errors_.push_back(message);
}

std::vector<Line> DialogueView::consume(const std::vector<LouderEvent>& events) {
  std::vector<Line> lines;

  for (const LouderEvent& event : events) {
    switch (event.data.tag) {
      case EventData::Tag::RegularMotion:
      case EventData::Tag::Warp:
      case EventData::Tag::ForceMotion: {
        const Motion& motion = motion_of(event);
        Viewport* viewport = viewport_for(event.actor);
        if (viewport == nullptr) {
          // An event for an actor this view does not show, such as the debug
          // actor used to inspect raw choices. Not an error.
          break;
        }

        Line line;
        line.actor = event.actor;
        line.position = motion.target_position;

        if (motion.background) {
          viewport->background = true;
          line.kind = Line::Kind::Background;
          line.text = "background";
          lines.push_back(line);
        } else {
          // A normal arrival clears the background flag: the actor is leading
          // its own flow again.
          viewport->background = false;
          line.kind = Line::Kind::Arrival;
          line.text = engine_->speaker_of(motion.target_position);
          lines.push_back(line);
        }
        break;
      }

      case EventData::Tag::ChannelChange: {
        Viewport* viewport = viewport_for(event.actor);
        if (viewport == nullptr) {
          break;
        }
        Line line;
        line.actor = event.actor;
        line.kind = Line::Kind::Channel;
        line.text = engine_->text_of(event.channel);
        lines.push_back(line);
        break;
      }

      case EventData::Tag::Callback: {
        // Callbacks are not part of the dialogue text. The footer reports them
        // so that a one-shot callback firing once is observable.
        last_callback_ = engine_->text_of(event.data.callback._0);
        Line line;
        line.actor = event.actor;
        line.kind = Line::Kind::Callback;
        line.text = last_callback_;
        lines.push_back(line);
        break;
      }

      default:
        // ChoiceAppearance is reconciled by refresh() asking the engine what it
        // thinks is available, which cannot disagree with itself. Condition,
        // Statement and the load-time reports have nothing to show here.
        break;
    }
  }

  refresh();
  return lines;
}

void DialogueView::refresh() {
  Viewport* viewports[2] = {&left_, &right_};
  for (Viewport* viewport : viewports) {
    Position position = engine_->position_of(conversation_, viewport->actor);
    viewport->has_position = position != kNoPosition;
    viewport->position = position;

    if (!viewport->has_position) {
      viewport->choices.clear();
      continue;
    }

    viewport->speaker = engine_->speaker_of(position);
    viewport->text = engine_->text_at(position);
    viewport->choices = engine_->available_choices(conversation_, position, viewport->actor);

// The engine's own count is the authority on how many choices exist, and
    // available_choices probes until it has found that many. A mismatch means
    // the probe hit its bound, which is worth surfacing rather than hiding
    // behind a silently short list.
    int32_t expected = engine_->choice_count(conversation_, position, viewport->actor);
    if (expected != static_cast<int32_t>(viewport->choices.size())) {
      fail(*viewport, "engine reports " + std::to_string(expected) + " choices at position " +
                          std::to_string(position) + " but the view found " +
                          std::to_string(viewport->choices.size()));
    }
  }
}

bool DialogueView::handle_key(char key) {
  int side = side_for_key(key);
  if (side < 0) {
    return false;
  }

  Viewport& viewport = side == 0 ? left_ : right_;

  // The extra CONTINUE key is only meaningful while this side has exactly one
  // choice. Otherwise it is not a mistyped choice key but a context-sensitive
  // one, so it is ignored rather than reported as an error.
  if (key == viewport.continue_key && !viewport.is_continue()) {
    return false;
  }

  // The number on screen is the index into this actor's *visible* choices. The
  // engine wants the index into the hub's full choice list, which conversation
  // logic has offset by hiding some of them. Conflating the two is the classic
  // mistake this example exists to demonstrate, so the mapping is explicit.
  //
  // In the single-choice state the extra key also selects that one choice, so it
  // maps to visible index 0.
  int visible_index = key == viewport.continue_key ? 0 : key_index_for(key);
  if (visible_index < 0 || visible_index >= static_cast<int>(viewport.choices.size())) {
    fail(viewport, "no choice " + std::string(1, key) + " for " + viewport.display_name);
    return true;
  }

  int32_t general_index = viewport.choices[visible_index].general_index;

  if (!engine_->select_choice(conversation_, viewport.actor, general_index)) {
    fail(viewport, "the engine refused choice " + std::to_string(general_index) + " at position " +
                       std::to_string(viewport.position));
    return true;
  }

  // Selecting clears the stale error only on success, so the message stays up
  // until something works.
  viewport.error.clear();
  consume(engine_->drain_events());
  return true;
}

}  // namespace louder_demo
