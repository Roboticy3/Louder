#include "screen.h"

#include <windows.h>

// _getch and _kbhit, which give keypresses without waiting for Enter.
#include <conio.h>

#include <algorithm>
#include <cstdio>
#include <cstring>
#include <vector>

namespace louder_demo {

namespace {

// Column sizing bounds.
constexpr int kMinColumnWidth = 24;
constexpr int kMaxColumnWidth = 64;

// Escape sequences. A console that refused virtual terminal processing ignores
// them harmlessly as literal text only in the worst case, which the Screen
// constructor avoids by gating all of them behind owns_console_.
constexpr char kReset[] = "\x1b[0m";
constexpr char kBold[] = "\x1b[1m";
constexpr char kDim[] = "\x1b[90m";
constexpr char kIdle[] = "\x1b[33m";
constexpr char kError[] = "\x1b[31m";
constexpr char kRule[] = "\x1b[90m|\x1b[0m";

std::string repeat(char character, int count) {
  return count > 0 ? std::string(static_cast<size_t>(count), character) : std::string();
}

// Break `text` into runs no wider than `width`, on word boundaries where
// possible. A word longer than the column is left to overflow rather than being
// cut in half, since cutting dialogue mid-word reads as a bug.
std::vector<std::string> wrap(const std::string& text, int width) {
  std::vector<std::string> rows;
  if (width <= 0) {
    return rows;
  }

  std::string line;
  std::string word;
  auto flush_word = [&]() {
    if (word.empty()) {
      return;
    }
    if (line.empty()) {
      line = word;
    } else if (static_cast<int>(line.size() + 1 + word.size()) <= width) {
      line += " " + word;
    } else {
      rows.push_back(line);
      line = word;
    }
    word.clear();
  };

  for (char character : text) {
    if (character == ' ' || character == '\n') {
      flush_word();
    } else {
      word += character;
    }
  }
  flush_word();

  if (!line.empty()) {
    rows.push_back(line);
  }
  return rows;
}

// Render one column into rows of text, without the header.
std::vector<std::string> column_body(const Viewport& viewport, const Viewport& other, int width) {
  std::vector<std::string> rows;

  if (!viewport.has_position) {
    rows.push_back("(not in the conversation)");
  } else {
    std::string who = viewport.speaker.empty() ? std::string("(no speaker)") : viewport.speaker;
    std::string label = viewport.background ? who + " (background)" : who;
    for (const std::string& row : wrap(label + ":", width)) {
      rows.push_back(row);
    }
    if (viewport.text.empty()) {
      rows.push_back("(no text)");
    } else {
      for (const std::string& row : wrap(viewport.text, width)) {
        rows.push_back(row);
      }
    }
  }

  rows.push_back("");

  if (!viewport.error.empty()) {
    for (const std::string& row : wrap("! " + viewport.error, width)) {
      rows.push_back(kError + row + kReset);
    }
    rows.push_back("");
  }

  if (!viewport.choices.empty()) {
    for (size_t i = 0; i < viewport.choices.size(); ++i) {
      // The key on screen numbers the visible list, which is not the engine's
      // index. Printing both keeps that distinction visible while testing.
      std::string prefix = std::string(1, viewport.key) + ") ";
      std::string entry = prefix + viewport.choices[i].text;
      std::vector<std::string> wrapped = wrap(entry, width);
      for (size_t row = 0; row < wrapped.size(); ++row) {
        // Continuation rows are indented to line up under the choice text.
        std::string indented = row == 0 ? wrapped[row] : std::string(prefix.size(), ' ') + wrapped[row];
        rows.push_back(indented);
      }
    }
  } else {
    // The spec's "X is thinking" line: this side has nothing to decide, so say
    // who does. When nobody has choices the conversation has stalled, which is
    // a different message.
    std::string message = other.choices.empty()
                              ? std::string("Nobody has choices. The conversation has stalled.")
                              : other.display_name + " is thinking...";
    for (const std::string& row : wrap(message, width)) {
      rows.push_back(kIdle + row + kReset);
    }
  }

  return rows;
}

}  // namespace

Screen::Screen() {
  HANDLE output = GetStdHandle(STD_OUTPUT_HANDLE);
  HANDLE input = GetStdHandle(STD_INPUT_HANDLE);
  out_ = output;
  in_ = input;

  DWORD mode = 0;
  if (GetConsoleMode(output, &mode)) {
    // Virtual terminal processing gives cursor moves and colours. Older consoles
    // refuse it, in which case the demo still runs, just without redraws.
    if (SetConsoleMode(output, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING)) {
      owns_console_ = true;
    }
    saved_out_mode_ = reinterpret_cast<void*>(static_cast<uintptr_t>(mode));
  }

  // Raw input: keys arrive one at a time without waiting for Enter, which is
  // what makes 1..9 and q..p usable as immediate commands.
  DWORD in_mode = 0;
  if (GetConsoleMode(input, &in_mode)) {
    DWORD raw = in_mode & ~(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
    raw |= ENABLE_EXTENDED_FLAGS;
    SetConsoleMode(input, raw);
  }

  if (owns_console_) {
    std::fputs("\x1b[?25l", stdout);  // hide the cursor
  }
  std::fflush(stdout);
}

Screen::~Screen() {
  if (owns_console_) {
    std::fputs("\x1b[?25h", stdout);  // show it again
    std::fflush(stdout);
  }
  HANDLE input = static_cast<HANDLE>(in_);
  DWORD in_mode = 0;
  if (input != nullptr && GetConsoleMode(input, &in_mode)) {
    SetConsoleMode(input, in_mode | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
  }
}

int Screen::width() const {
  CONSOLE_SCREEN_BUFFER_INFO info;
  if (GetConsoleScreenBufferInfo(static_cast<HANDLE>(out_), &info)) {
    int usable = info.srWindow.Right - info.srWindow.Left + 1;
    return usable > 0 ? usable : 80;
  }
  return 80;
}

int Screen::height() const {
  CONSOLE_SCREEN_BUFFER_INFO info;
  if (GetConsoleScreenBufferInfo(static_cast<HANDLE>(out_), &info)) {
    int usable = info.srWindow.Bottom - info.srWindow.Top + 1;
    return usable > 0 ? usable : 24;
  }
  return 24;
}

void Screen::begin_frame() {
  if (owns_console_) {
    std::fputs("\x1b[H", stdout);  // home
  }
  std::fflush(stdout);
}

void Screen::put(int row, int column, const std::string& text) {
  std::printf("\x1b[%d;%dH%s", row + 1, column + 1, text.c_str());
}

void Screen::clear_row(int row, int from_column, int to_column) {
  put(row, from_column, repeat(' ', to_column - from_column + 1));
}

void Screen::end_frame() {
  std::fflush(stdout);
}

bool Screen::key_pending() {
  return _kbhit() != 0;
}

char Screen::read_key() {
  int key = _getch();
  if (key == 0 || key == 0xE0) {
    // Extended key. Consume its scan code so it does not surface as a command.
    _getch();
    return '\0';
  }
  return static_cast<char>(key);
}

std::string format_lines(const std::vector<Line>& lines) {
  std::string text;
  for (const Line& line : lines) {
    switch (line.kind) {
      case Line::Kind::Arrival:
        text += "arrived at " + std::to_string(line.position);
        if (!line.text.empty()) {
          text += " (" + line.text + ")";
        }
        break;
      case Line::Kind::Background:
        text += "saw " + std::to_string(line.position) + " in the background";
        break;
      case Line::Kind::Channel:
        text += "changed channel to " + line.text;
        break;
      case Line::Kind::Callback:
        text += "callback " + line.text;
        break;
      case Line::Kind::Error:
        text += "error: " + line.text;
        break;
    }
    text += "\n";
  }
  return text;
}

void draw(Screen& screen, const DialogueView& view, const Viewport& left, const Viewport& right) {
  const int total = screen.width();
  const int rows_available = screen.height();

  // Three columns of content: left, the rule, right. The rule takes one
  // character plus a space either side.
  int column_width = (total - 3) / 2;
  column_width = std::min(std::max(column_width, kMinColumnWidth), kMaxColumnWidth);
  const int left_x = 0;
  const int rule_x = column_width + 1;
  const int right_x = column_width + 2;

  std::vector<std::string> left_rows = column_body(left, right, column_width);
  std::vector<std::string> right_rows = column_body(right, left, column_width);

  screen.begin_frame();

  screen.put(0, left_x, kBold + left.display_name + kReset + kDim + "  [" +
                            std::string(1, left.key) + "-key]" + kReset);
  screen.put(0, rule_x, kRule);
  screen.put(0, right_x, kBold + right.display_name + kReset + kDim + "  [" +
                             std::string(1, right.key) + "-key]" + kReset);
  screen.clear_row(1, 0, total - 1);

  // The taller of the two columns decides how many rows are needed; the shorter
  // one gets blank rows below it so a redraw never leaves stale text.
  size_t needed = std::max(left_rows.size(), right_rows.size());
  size_t visible = needed;
  if (rows_available > 6) {
    visible = std::min(visible, static_cast<size_t>(rows_available - 6));
  }

  // Show the tail of each column, so the newest dialogue is always on screen.
  size_t left_start = left_rows.size() > visible ? left_rows.size() - visible : 0;
  size_t right_start = right_rows.size() > visible ? right_rows.size() - visible : 0;

  for (size_t i = 0; i < visible; ++i) {
    int row = static_cast<int>(2 + i);
    screen.clear_row(row, 0, total - 1);
    screen.put(row, left_x, i + left_start < left_rows.size() ? left_rows[i + left_start] : "");
    screen.put(row, rule_x, kRule);
    screen.put(row, right_x, i + right_start < right_rows.size() ? right_rows[i + right_start] : "");
  }

  // Anything below the drawn body is cleared, in case the last frame was taller.
  for (size_t i = visible; i < needed; ++i) {
    int row = static_cast<int>(2 + i);
    if (row >= rows_available - 3) {
      break;
    }
    screen.clear_row(row, 0, total - 1);
    screen.put(row, rule_x, kRule);
  }

  int footer_row = rows_available - 1;
  screen.clear_row(footer_row, 0, total - 1);
  std::string footer = kDim + std::string("ESC to quit") + kReset;
  if (!view.errors().empty()) {
    footer += std::string(kError) + "  last error: " + view.errors().back() + kReset;
  } else if (!view.last_callback().empty()) {
    footer += std::string(kDim) + "  last callback: " + view.last_callback() + kReset;
  }
  screen.put(footer_row, 0, footer);

  screen.end_frame();
}

}  // namespace louder_demo