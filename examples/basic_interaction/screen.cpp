#include "screen.h"

#include <windows.h>

// _getch and _kbhit, which give keypresses without waiting for Enter.
#include <conio.h>

#include <algorithm>
#include <cstdio>
#include <cstring>

namespace louder_demo {

namespace {

// Column sizing bounds. The upper bound exists mostly so that dialogue is not
// stretched across an absurdly wide terminal.
constexpr int kMinColumnWidth = 20;
constexpr int kMaxColumnWidth = 58;

// Rows reserved below the body: one blank, then the footer.
constexpr int kReservedRows = 2;

// How many characters of styling to wrap a run in.
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

// Trim to `width` visible characters. Wrapping is deliberately not done here;
// column_body breaks lines on word boundaries beforehand, and a line that is
// still too long is cut rather than allowed to overrun the column.
std::string truncate(const std::string& text, int width) {
  if (width <= 0 || static_cast<int>(text.size()) <= width) {
    return width <= 0 ? "" : text;
  }
  return text.substr(0, static_cast<size_t>(width));
}

// Break `text` into runs no wider than `width`, on word boundaries where
// possible. A word longer than the column is left to be truncated rather than
// being broken, since cutting dialogue mid-word reads as a bug.
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

struct BodyLine {
  std::string text;
  Style style = Style::Normal;
};

// Render one column into lines of text, without the header.
//
// `keys` is the key column for that side, indexed by position in the visible
// choice list, so choice 0 gets keys[0] rather than the column's header key.
std::vector<BodyLine> column_body(const Viewport& viewport, const Viewport& other,
                              const char* keys, int width) {
  std::vector<BodyLine> lines;

  if (!viewport.has_position) {
    lines.push_back({"(not in the conversation)", Style::Dim});
  } else {
    std::string who = viewport.speaker.empty() ? std::string("(no speaker)") : viewport.speaker;
    if (viewport.background) {
      who += " (in the background)";
    }
    for (const std::string& row : wrap(who + ":", width)) {
      lines.push_back({row, Style::Dim});
    }
    if (viewport.text.empty()) {
      lines.push_back({"(no text)", Style::Dim});
    } else {
      for (const std::string& row : wrap(viewport.text, width)) {
        lines.push_back({row, Style::Normal});
      }
    }
  }

  if (!viewport.error.empty()) {
    lines.push_back({"", Style::Normal});
    for (const std::string& row : wrap("! " + viewport.error, width)) {
      lines.push_back({row, Style::Error});
    }
  }

  if (viewport.choices.empty()) {
    lines.push_back({"", Style::Normal});
    // The spec's "X is thinking" line: this side has nothing to decide, so say
    // who does. When nobody has choices the conversation has stalled, which is
    // a different message and a different colour.
    std::string message = other.choices.empty()
                              ? std::string("Nobody has any choices. The conversation has stalled.")
                              : other.display_name + " is thinking...";
    for (const std::string& row : wrap(message, width)) {
      lines.push_back({row, Style::Idle});
    }
    return lines;
  }

  lines.push_back({"", Style::Normal});
  for (size_t i = 0; i < viewport.choices.size(); ++i) {
    size_t key_index = i;
    char key = '?';
    for (size_t k = 0; kRightKeys[k] != '\0'; ++k) {
      if (k == key_index) {
        key = keys[k];
      }
    }

    std::string prefix = std::string(1, key) + ") ";
    std::vector<std::string> wrapped = wrap(prefix + viewport.choices[i].text, width);
    for (size_t row = 0; row < wrapped.size(); ++row) {
      // Continuation rows line up under the choice text rather than under the
      // key.
      std::string text = row == 0 ? wrapped[row] : std::string(prefix.size(), ' ') + wrapped[row];
      lines.push_back({truncate(text, width), Style::Normal});
    }
  }
  return lines;
}

}  // namespace

Frame render(const DialogueView& view, const Viewport& left, const Viewport& right,
             int width, int height) {
  Frame frame;

  // One column of content, one column of rule, one column of content. The rule
  // takes a single character, and the columns are kept to width - 2 so that
  // nothing is written into the last cell of a row: a character there makes the
  // cursor wrap, and a wrapped console scrolls the screen up a line, which is
  // what puts a stale row where the header should be.
  const int usable = std::max(width - 2, kMinColumnWidth);
  const int column_width = std::min(std::max(usable / 2, kMinColumnWidth), kMaxColumnWidth);
  const int rule_x = column_width;
  const int right_x = column_width + 1;

  // Header. Row 0 of the frame.
  FrameRow header;
  header.left = left.display_name + "  [" + std::string(1, left.key) + "]";
  header.right = right.display_name + "  [" + std::string(1, right.key) + "]";
  header.left_style = Style::Bold;
  header.right_style = Style::Bold;
  frame.rows.push_back(header);

  std::vector<BodyLine> left_lines = column_body(left, right, kLeftKeys, column_width);
  std::vector<BodyLine> right_lines = column_body(right, left, kRightKeys, column_width);

  // How many body rows fit between the header and the footer.
  const int body_capacity = std::max(height - 1 - kReservedRows, 0);
  const size_t needed = std::max(left_lines.size(), right_lines.size());
  const size_t visible = std::min(needed, static_cast<size_t>(body_capacity));

  // Show the tail of each column, so the newest dialogue is always on screen.
  const size_t left_start = left_lines.size() > visible ? left_lines.size() - visible : 0;
  const size_t right_start = right_lines.size() > visible ? right_lines.size() - visible : 0;

  for (size_t i = 0; i < visible; ++i) {
    FrameRow row;
    if (i + left_start < left_lines.size()) {
      row.left = truncate(left_lines[i + left_start].text, column_width);
      row.left_style = left_lines[i + left_start].style;
    }
    if (i + right_start < right_lines.size()) {
      row.right = truncate(right_lines[i + right_start].text, column_width);
      row.right_style = right_lines[i + right_start].style;
    }
    frame.rows.push_back(row);
  }

// The footer reports the newest engine error, else the newest callback, else
  // nothing. The quit hint is always present and always last, because a footer
  // that replaced it would leave a stuck player with no visible way out.
  std::string footer = "ESC to quit";
  Style footer_style = Style::Dim;
  if (!view.errors().empty()) {
    footer = "last error: " + view.errors().back() + "  --  ESC to quit";
    footer_style = Style::Error;
  } else if (!view.last_callback().empty()) {
    footer = "last callback: " + view.last_callback() + "  --  ESC to quit";
    footer_style = Style::Dim;
  }
  frame.footer = truncate(footer, usable);
  frame.footer_style = footer_style;

  // A frame must never be taller than the console, or the bottom of it lands on
  // a row that does not exist and scrolls the screen. The header always stays;
  // the body is what gets dropped when there is no room.
  const int max_rows = std::max(height - kReservedRows, 0);
  if (static_cast<int>(frame.rows.size()) > max_rows) {
    frame.rows.resize(static_cast<size_t>(max_rows));
  }

  (void)rule_x;
  (void)right_x;
  return frame;
}

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
  if (input != nullptr && input != INVALID_HANDLE_VALUE && GetConsoleMode(input, &in_mode)) {
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
    // Home, then clear the whole screen. Without the clear, a shorter frame
    // leaves the tail of a taller previous frame behind.
    std::fputs("\x1b[H\x1b[2J", stdout);
  }
  std::fflush(stdout);
}

void Screen::put(int row, int column, const std::string& text) {
  // Erase to end of line after writing, so a shorter line does not leave the
  // remnants of a longer one from the previous frame. Doing this instead of
  // padding with spaces is what keeps the cursor out of the last cell.
  std::printf("\x1b[%d;%dH%s\x1b[K", row + 1, column + 1, text.c_str());
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

void draw(Screen& screen, const Frame& frame, int width, int height) {
  const int usable = std::max(width - 2, kMinColumnWidth);
  const int column_width = std::min(std::max(usable / 2, kMinColumnWidth), kMaxColumnWidth);
  const int rule_x = column_width;
  const int right_x = column_width + 1;

  screen.begin_frame();

  for (size_t i = 0; i < frame.rows.size(); ++i) {
    const int row = static_cast<int>(i);
    if (row >= height - 1) {
      break;
    }
    const FrameRow& line = frame.rows[i];
    const char* left_code = style_code(line.left_style);
    const char* right_code = style_code(line.right_style);

    std::string text = std::string(left_code) + line.left + "\x1b[0m";
    screen.put(row, 0, text);
    screen.put(row, rule_x, "\x1b[90m|\x1b[0m");
    screen.put(row, right_x, std::string(right_code) + line.right + "\x1b[0m");
  }

  const int footer_row = height - 1;
  screen.put(footer_row, 0, std::string(style_code(frame.footer_style)) + frame.footer + "\x1b[0m");

  screen.end_frame();
}

}  // namespace louder_demo
