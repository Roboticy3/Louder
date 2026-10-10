// Rendering and key input for the split-column dialogue view.
//
// Kept apart from DialogueView so that all of the interesting behaviour is
// testable without a terminal. This file owns the console only.
#pragma once

#include <string>
#include <vector>

#include "dialogue_view.h"

namespace louder_demo {

// How a run of text is coloured. Applied per line rather than inline, so the
// layout can be computed and inspected as plain text before any escape
// sequences exist.
enum class Style {
  Normal,
  Bold,
  Dim,
  Idle,   // the "the other side is thinking" line
  Error,  // an engine refusal
};

// One row of the body, with a cell per column.
struct FrameRow {
  std::string left;
  std::string right;
  Style left_style = Style::Normal;
  Style right_style = Style::Normal;
};

// A whole frame, already laid out and truncated to fit.
//
// Row 0 is the header. The body follows. The footer is drawn separately on the
// last console row so that a long conversation cannot push it off screen.
//
// Every string here is plain text no wider than the column it will be drawn in,
// which is what keeps the frame from writing past the last cell of a row. A
// console that wraps at the right margin scrolls the whole screen up by one,
// which is how a stale line ends up where the header belongs.
struct Frame {
  std::vector<FrameRow> rows;
  std::string footer;
  Style footer_style = Style::Dim;
};

// Lay out both columns for a console of the given size.
//
// `width` and `height` are the usable console dimensions. The result never
// addresses a row at or beyond `height`, and never produces a line wider than
// the column, so drawing it cannot scroll the screen.
Frame render(const DialogueView& view, const Viewport& left, const Viewport& right,
             int width, int height);

// Owns the console for the lifetime of the object: switches the window into
// virtual terminal mode, hides the cursor, and restores both on destruction.
class Screen {
 public:
  Screen();
  ~Screen();

  Screen(const Screen&) = delete;
  Screen& operator=(const Screen&) = delete;

  int width() const;
  int height() const;

  // Move the cursor to the top left. Everything is drawn from there in one go.
  void begin_frame();

  // Write `text` at a zero-based row and column, then erase the rest of that
  // line. Nothing is written past the last column, so the cursor cannot wrap.
  void put(int row, int column, const std::string& text);

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
};

// Paint a laid-out frame. Separated from render so the layout can be checked
// against plain text in tests.
void draw(Screen& screen, const Frame& frame, int width, int height);

}  // namespace louder_demo