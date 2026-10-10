// Rendering and key input for the split-column dialogue view.
//
// Kept apart from DialogueView so that all of the interesting behaviour is
// testable without a terminal. This file owns the console only.
#pragma once

#include <string>

#include "dialogue_view.h"

namespace louder_demo {

// Owns the console for the lifetime of the object: switches the window into
// virtual terminal mode, hides the cursor, and restores both on destruction.
class Screen {
 public:
  Screen();
  ~Screen();

  Screen(const Screen&) = delete;
  Screen& operator=(const Screen&) = delete;

  // The usable width of the console, and the number of rows.
  int width() const;
  int height() const;

  // Erase and redraw from the top left.
  void begin_frame();

  // Move to a row and column and write there. Coordinates are zero-based.
  void put(int row, int column, const std::string& text);

  // Fill a horizontal run with spaces, for clearing a column's old contents.
  void clear_row(int row, int from_column, int to_column);

  // Force the frame out to the terminal.
  void end_frame();

  // True when a key is waiting. Does not block.
  bool key_pending();

  // The next keypress. Blocks until one is available. Arrow keys and other
  // extended keys arrive as a leading 0 or 0xE0 and are reported as 0, so they
  // are ignored by the view rather than mistaken for a choice.
  char read_key();

 private:
  bool owns_console_ = false;
  void* out_ = nullptr;
  void* in_ = nullptr;
  void* saved_out_mode_ = nullptr;
};

// Draw both columns, the header, and the footer.
//
// `left` and `right` are the two viewports. Each is drawn in its own half of the
// terminal with a vertical rule between them. When a side has no choices but the
// other does, the idle side says so, which is the "X is thinking" line the spec
// asks for.
void draw(Screen& screen, const DialogueView& view, const Viewport& left, const Viewport& right);

// Draw the single column of lines a batch of events produced, for the log.
std::string format_lines(const std::vector<Line>& lines);

}  // namespace louder_demo