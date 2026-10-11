// The Windows console: how big it is, and the two ways this demo talks to it.
//
// This is the only file that knows a console exists. Everything else lays out
// text and hands it over, so nothing but main() ever puts the terminal into raw
// mode, and the interface tests can drive a whole session without one.
#pragma once

#include <string>

// Owns the console for as long as it is alive: switches the window into virtual
// terminal mode, hides the cursor, and puts input back the way it found it on
// destruction. The console's input mode outlives the process, so restoring it is
// not optional.
struct Terminal {
  Terminal();
  ~Terminal();

  Terminal(const Terminal&) = delete;
  Terminal& operator=(const Terminal&) = delete;

  // The visible size. Asked fresh each frame so that resizing the window takes
  // effect immediately. Falls back to 80x24 when stdout is not a console.
  int width() const;
  int height() const;

  // Home the cursor and clear the screen. Without the clear, a shorter frame
  // leaves the tail of a taller previous frame behind.
  void begin_frame();

  // Write `text` at a zero-based row and column, then erase the rest of that
  // line. Nothing is written past the last column, because a character in the
  // last cell wraps the cursor and a wrapped console scrolls the screen up a
  // line, which is what puts a stale row where the header belongs.
  void put(int row, int column, const std::string& text);

  void end_frame();

  // The next keypress, blocking until one arrives. Arrow keys and other
  // extended keys arrive as a leading 0 or 0xE0 and are reported as '\0', so
  // the dialogue ignores them rather than mistaking one for a choice.
  char read_key();

 private:
  // Whether the console accepted virtual terminal processing, and so whether
  // cursor moves and colour do anything. An older console refuses it, and the
  // demo then runs without colour rather than not at all.
  bool colour_ = false;
};
