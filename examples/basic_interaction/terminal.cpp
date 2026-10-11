#include "terminal.h"

#include <windows.h>

// _getch and _kbhit, which deliver keypresses without waiting for Enter.
#include <conio.h>

#include <cstdio>

namespace {

HANDLE stdout_handle() {
  return GetStdHandle(STD_OUTPUT_HANDLE);
}

HANDLE stdin_handle() {
  return GetStdHandle(STD_INPUT_HANDLE);
}

// Console dimensions, or 0 when stdout is not a console at all.
bool visible_size(int* width, int* height) {
  CONSOLE_SCREEN_BUFFER_INFO info;
  if (!GetConsoleScreenBufferInfo(stdout_handle(), &info)) {
    return false;
  }
  *width = info.srWindow.Right - info.srWindow.Left + 1;
  *height = info.srWindow.Bottom - info.srWindow.Top + 1;
  return true;
}

}  // namespace

Terminal::Terminal() {
  HANDLE out = stdout_handle();
  DWORD out_mode = 0;
  if (GetConsoleMode(out, &out_mode)) {
    colour_ = SetConsoleMode(out, out_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0;
  }

  // Raw input: keys arrive one at a time without waiting for Enter, which is
  // what makes 1..9 and q..p usable as immediate commands. ENABLE_PROCESSED_INPUT
  // goes too, so Ctrl+C reaches the program rather than being swallowed.
  HANDLE in = stdin_handle();
  DWORD in_mode = 0;
  if (GetConsoleMode(in, &in_mode)) {
    DWORD raw = in_mode & ~(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
    raw |= ENABLE_EXTENDED_FLAGS;
    SetConsoleMode(in, raw);
  }

  if (colour_) {
    std::fputs("\x1b[?25l", stdout);  // hide the cursor
    std::fflush(stdout);
  }
}

Terminal::~Terminal() {
  if (colour_) {
    std::fputs("\x1b[?25h", stdout);  // show it again
    std::fflush(stdout);
  }

  HANDLE in = stdin_handle();
  DWORD in_mode = 0;
  if (in != INVALID_HANDLE_VALUE && GetConsoleMode(in, &in_mode)) {
    SetConsoleMode(in, in_mode | ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);
  }
}

int Terminal::width() const {
  int width = 0;
  int height = 0;
  return visible_size(&width, &height) && width > 0 ? width : 80;
}

int Terminal::height() const {
  int width = 0;
  int height = 0;
  return visible_size(&width, &height) && height > 0 ? height : 24;
}

void Terminal::begin_frame() {
  if (colour_) {
    std::fputs("\x1b[H\x1b[2J", stdout);
  }
  std::fflush(stdout);
}

void Terminal::put(int row, int column, const std::string& text) {
  // Erase to end of line rather than padding with spaces, which is what keeps
  // the cursor out of the last cell.
  std::printf("\x1b[%d;%dH%s\x1b[K", row + 1, column + 1, text.c_str());
}

void Terminal::end_frame() {
  std::fflush(stdout);
}

char Terminal::read_key() {
  int key = _getch();
  if (key == 0 || key == 0xE0) {
    // Extended key. Consume its scan code so it does not surface as a command.
    _getch();
    return '\0';
  }
  return static_cast<char>(key);
}
