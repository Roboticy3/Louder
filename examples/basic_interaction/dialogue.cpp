#include "dialogue.h"

#include <algorithm>
#include <cstdio>

// Column sizing bounds. The upper bound exists mostly so that dialogue is not
// stretched across an absurdly wide terminal.
namespace {
constexpr int kMinColumnWidth = 20;
constexpr int kMaxColumnWidth = 58;

// An upper bound on how far to probe for choices, so that a hub claiming more
// choices than any real document would carry cannot turn the walk into an
// unbounded loop.
constexpr int32_t kMaxChoices = 64;

// How a run of text is coloured, so that layout can be checked as plain text
// before any escape sequences exist.
enum class Style { Normal, Bold, Dim, Idle, Error };

const char* style_code(Style style) {
  switch (style) {
    case Style::Bold:
      return "\x1b[1m";
    case Style::Dim:
      return "\x1b[90m";
    case Style::Idle:
      return "\x1b[33m";
    case Style::Error:
      return "\x1b[31m";
    default:
      return "";
  }
}

// One laid-out row of a column.
struct Row {
  std::string text;
  Style style = Style::Normal;
};

std::string styled(const std::string& text, Style style) {
  return std::string(style_code(style)) + text + "\x1b[0m";
}

// Cut to `width` visible characters.
std::string truncate(const std::string& text, int width) {
  return static_cast<int>(text.size()) <= width ? text : text.substr(0, static_cast<size_t>(width));
}

// Split on spaces and newlines, dropping the empty pieces.
std::vector<std::string> words_of(const std::string& text) {
  std::vector<std::string> words;
  std::string current;
  for (char character : text) {
    if (character == ' ' || character == '\n') {
      if (!current.empty()) {
        words.push_back(current);
        current.clear();
      }
    } else {
      current += character;
    }
  }
  if (!current.empty()) {
    words.push_back(current);
  }
  return words;
}

// Break `text` into runs no wider than `width`, on word boundaries where
// possible. A word longer than the column is left to be truncated rather than
// broken, since cutting dialogue mid-word reads as a bug.
std::vector<std::string> wrap(const std::string& text, int width) {
  std::vector<std::string> rows;
  if (width <= 0) {
    return rows;
  }

  std::string line;
  for (const std::string& word : words_of(text)) {
    if (line.empty()) {
      line = word;
    } else if (static_cast<int>(line.size() + 1 + word.size()) <= width) {
      line += " " + word;
    } else {
      rows.push_back(line);
      line = word;
    }
  }
  if (!line.empty()) {
    rows.push_back(line);
  }
  return rows;
}

// The position of `key` in `keys`, or -1.
int index_of(const char* keys, char key) {
  for (int i = 0; keys[i] != '\0'; ++i) {
    if (keys[i] == key) {
      return i;
    }
  }
  return -1;
}

// Every choice on `position` that `actor` could actually take, in ascending
// index order.
//
// The engine has no call for a hub's raw choice count, and it reports an index
// it will not serve the same way whether that index is past the end of the list
// or merely not available to this actor. A refusal therefore cannot end the
// walk. The available indices are not contiguous either: hub 26 of the demo
// conversation carries five raw choices and offers Freya only index 4, and
// Benjamin only indices 1 and 2. So every index is probed, and the engine's own
// count is what says when the list is complete.
std::vector<Choice> available_choices(LouderContext* ctx, LouderName conversation,
                                      LouderPosition position, LouderActor actor) {
  std::vector<Choice> choices;
  const int32_t expected = get_hub_choice_count(ctx, conversation, position, actor);
  for (int32_t index = 0;
       index < kMaxChoices && static_cast<int32_t>(choices.size()) < expected;
       ++index) {
    LouderPosition link = get_hub_choice_link(ctx, conversation, position, actor, index);
    if (link < 0) {
      continue;
    }
    choices.push_back(Choice{index, link,
                             get_hub_choice_rule(ctx, conversation, position, actor, index),
                             text_of(ctx, get_hub_choice_text(ctx, conversation, position,
                                                              actor, index))});
  }
  return choices;
}

// Lay one column out as rows of plain text, no wider than `width`.
std::vector<Row> column_rows(const Column& column, const Column& other, int width) {
  std::vector<Row> rows;
  auto push = [&](const std::string& text, Style style) {
    for (const std::string& line : wrap(text, width)) {
      rows.push_back(Row{line, style});
    }
  };

  if (!column.in_conversation) {
    push("(not in the conversation)", Style::Dim);
  } else {
    push(column.speaker.empty() ? "(no speaker):" : column.speaker + ":", Style::Dim);
    push(column.text.empty() ? "(no text)" : column.text, Style::Normal);
  }

  if (!column.error.empty()) {
    rows.push_back(Row{"", Style::Normal});
    push("! " + column.error, Style::Error);
  }

  if (column.choices.empty()) {
    rows.push_back(Row{"", Style::Normal});
    // The spec's "X is thinking" line: this side has nothing to decide, so say
    // who does. When nobody has choices the conversation has stalled, which is
    // a different message.
    push(other.choices.empty() ? "Nobody has any choices. The conversation has stalled."
                               : std::string(other.display_name) + " is thinking...",
         Style::Idle);
    return rows;
  }

  rows.push_back(Row{"", Style::Normal});
  for (size_t i = 0; i < column.choices.size(); ++i) {
    // "[1 or backtick (`)] CONTINUE" in the single-choice state, and just
    // "[1] <choice text>" otherwise. Naming the keys is what makes the CONTINUE
    // binding discoverable without a manual.
    const bool lone = column.is_continue();
    std::string keys(1, column.key_for(i));
    if (lone) {
      keys += std::string(" or ") + column.continue_key_name;
    }
    const std::string prefix = "[" + keys + "] ";
    std::string body = lone ? kContinueLabel : column.choices[i].text;

    // The rule that opened this choice, as reported by the engine. Shown in
    // brackets so it is visible which rule let the choice through.
    const char* rule_name = "unknown";
    switch (static_cast<ChoiceRule>(column.choices[i].rule)) {
      case ChoiceRule::Standard:
        rule_name = "standard";
        break;
      case ChoiceRule::Skill:
        rule_name = "skill";
        break;
      case ChoiceRule::Spoke:
        rule_name = "spoke";
        break;
      case ChoiceRule::Debug:
        rule_name = "debug";
        break;
      case ChoiceRule::Invisible:
        rule_name = "invisible";
        break;
      case ChoiceRule::Held:
        rule_name = "held";
        break;
    }
    body += "  (" + std::string(rule_name) + ")";

    const std::vector<std::string> lines = wrap(prefix + body, width);
    for (size_t row = 0; row < lines.size(); ++row) {
      // Wrapped rows line up under the label rather than under the keys.
      rows.push_back(Row{truncate(row == 0 ? lines[row]
                                          : std::string(prefix.size(), ' ') + lines[row],
                                  width),
                         Style::Normal});
    }
  }
  return rows;
}

}  // namespace

LouderName intern(LouderContext* ctx, const std::string& text) {
  const uint8_t* bytes = reinterpret_cast<const uint8_t*>(text.data());
  LouderName existing = get_named_id(ctx, bytes, text.size());
  return existing != LOUDER_NAME_NONE ? existing : declare_name(ctx, bytes, text.size());
}

std::string text_of(LouderContext* ctx, LouderName name) {
  if (name == LOUDER_NAME_NONE) {
    return "";
  }
  // Enough for any name in these documents, with room to spare. The engine
  // writes a terminator, so this is passed as size - 1.
  std::vector<uint8_t> buffer(8192, 0);
  size_t written = read_name(ctx, name, buffer.data(), buffer.size() - 1);
  return std::string(reinterpret_cast<const char*>(buffer.data()), written);
}

void View::setup(LouderContext* engine, LouderName conversation_name,
                 LouderActor left_actor, LouderActor right_actor) {
  ctx = engine;
  conversation = conversation_name;

  left = Column();
  left.actor = left_actor;
  left.display_name = "Freya";
  left.keys = kFreyaKeys;
  left.continue_key = kFreyaContinueKey;
  left.continue_key_name = kFreyaContinueKeyName;

  right = Column();
  right.actor = right_actor;
  right.display_name = "REDACTED";
  right.keys = kBenjaminKeys;
  right.continue_key = kBenjaminContinueKey;
  right.continue_key_name = kBenjaminContinueKeyName;

  refresh();
}

void View::refresh() {
  Column* columns[2] = {&left, &right};
  for (Column* column : columns) {
    column->position = get_actor_position(ctx, conversation, column->actor);
    column->in_conversation = column->position >= 0;
    column->choices.clear();
    if (!column->in_conversation) {
      continue;
    }

    column->speaker = text_of(ctx, get_hub_speaker(ctx, conversation, column->position));
    column->text = text_of(ctx, get_hub_text(ctx, conversation, column->position));
    column->choices = available_choices(ctx, conversation, column->position, column->actor);

    // The engine's own count is the authority on how many choices exist, and
    // available_choices probes until it has found that many. A shortfall means
    // the probe hit its bound, which is worth surfacing rather than hiding
    // behind a silently short list.
    int32_t expected = get_hub_choice_count(ctx, conversation, column->position, column->actor);
    if (expected != static_cast<int32_t>(column->choices.size())) {
      column->error = "the engine reports " + std::to_string(expected) +
                      " choices here but " + std::to_string(column->choices.size()) +
                      " could be read";
    }
  }
}

bool View::press(char key) {
  Column* columns[2] = {&left, &right};
  Column* column = nullptr;
  int visible = -1;

  for (Column* candidate : columns) {
    // The second CONTINUE key, which only means something while that side has
    // exactly one choice.
    if (key == candidate->continue_key && candidate->is_continue()) {
      column = candidate;
      visible = 0;
      break;
    }
    int index = index_of(candidate->keys, key);
    if (index >= 0) {
      column = candidate;
      visible = index;
      break;
    }
  }

  // Nobody claims it: not a choice key, and not a CONTINUE key while that side
  // has several choices. Context-sensitive, so ignored rather than reported.
  if (column == nullptr) {
    return false;
  }

  // The number on screen indexes this actor's *visible* choices. The engine
  // wants the index into the hub's full list, which conversation logic has
  // offset by hiding some of them. Conflating the two is the mistake this
  // example exists to rule out, so the mapping is explicit.
  if (visible >= static_cast<int>(column->choices.size())) {
    column->error = "there is no choice " + std::string(1, key) + " here";
    return true;
  }

  const int32_t general_index = column->choices[visible].general_index;
  if (select_choice(ctx, conversation, column->actor, general_index) != 0) {
    column->error = "the engine refused that move";
    return true;
  }

  // Success clears the stale error, so a message stays up until something works.
  column->error.clear();

  // Drain the queue, keeping the one thing this demo shows from it. Everything
  // else is either re-read from the engine by refresh() or not displayed.
  for (size_t i = 0;; ++i) {
    LouderOption<LouderEvent> option = get_event(ctx, i);
    if (option.tag == LouderOption<LouderEvent>::Tag::None) {
      break;
    }
    if (option.some._0.data.tag == EventData::Tag::Callback) {
      last_callback = text_of(ctx, option.some._0.data.callback._0);
    }
  }
  flush_events(ctx);

  refresh();
  return true;
}

void View::draw(Terminal& terminal) const {
  const int width = terminal.width();
  const int height = terminal.height();

  // One column of content, a rule, and another column. Everything is kept to
  // width - 2 so that nothing is written into the last cell of a row: see
  // Terminal::put for why that matters.
  const int usable = std::max(width - 2, kMinColumnWidth);
  const int column_width = std::min(std::max(usable / 2, kMinColumnWidth), kMaxColumnWidth);
  const int right_x = column_width + 1;

  std::vector<Row> left_rows = column_rows(left, right, column_width);
  std::vector<Row> right_rows = column_rows(right, left, column_width);

  // How many rows of each column fit between the header and the footer. The two
  // columns are usually different lengths, so each is shown from its own end and
  // the newest dialogue is what stays on screen.
  const size_t capacity = static_cast<size_t>(std::max(height - 3, 0));
  const size_t left_count = std::min(left_rows.size(), capacity);
  const size_t right_count = std::min(right_rows.size(), capacity);
  const size_t left_start = left_rows.size() - left_count;
  const size_t right_start = right_rows.size() - right_count;

  terminal.begin_frame();

  terminal.put(0, 0, styled(std::string(left.display_name) + "  [" + left.key_for(0) + "]",
                            Style::Bold));
  terminal.put(0, column_width, "\x1b[90m|\x1b[0m");
  terminal.put(0, right_x, styled(std::string(right.display_name) + "  [" + right.key_for(0) + "]",
                                 Style::Bold));

  for (size_t i = 0; i < std::max(left_count, right_count); ++i) {
    const int row = static_cast<int>(i) + 1;
    if (row >= height - 1) {
      break;  // the footer owns the last row
    }
    if (i < left_count) {
      terminal.put(row, 0, styled(left_rows[left_start + i].text,
                                  left_rows[left_start + i].style));
    }
    terminal.put(row, column_width, "\x1b[90m|\x1b[0m");
    if (i < right_count) {
      terminal.put(row, right_x, styled(right_rows[right_start + i].text,
                                        right_rows[right_start + i].style));
    }
  }

  // The footer reports the newest error, else the newest callback, else nothing.
  // The quit hint is always present and always last, because a footer that
  // replaced it would leave a stuck player with no visible way out.
  const Column* complaining = !left.error.empty() ? &left : (!right.error.empty() ? &right : nullptr);
  std::string footer = "ESC to quit";
  Style footer_style = Style::Dim;
  if (complaining != nullptr) {
    footer = std::string(complaining->display_name) + ": " + complaining->error + "  --  ESC to quit";
    footer_style = Style::Error;
  } else if (!last_callback.empty()) {
    footer = "last callback: " + last_callback + "  --  ESC to quit";
  }
  terminal.put(height - 1, 0, styled(truncate(footer, usable), footer_style));

  terminal.end_frame();
}
