#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

// Types mirroring louder_rust/src/lib.rs.
typedef int32_t LouderName;
typedef size_t LouderEncoding;
struct LouderContext;

#define LOUDER_NAME_NONE ((LouderName)-1)
#define LOUDER_SUCCESS 0
#define LOUDER_ERROR_NOT_FOUND (-1)

//basically the only reason I'm using cpp is because I want to see if the function passes through this "extern C" stuff I'm not too familiar with.
//otherwise, this should be a .c file.
extern "C" {
  int32_t __stdcall test_linkage() {
    return 123;
  }
}

// The engine's FFI surface, matching louder_rust/src/lib.rs.
extern "C" {
  LouderEncoding __stdcall get_next_encoding(LouderEncoding start);
  void __stdcall get_encoding_name(LouderEncoding index, uint8_t *buffer, size_t max_size);
  size_t __stdcall change_encoding(uint8_t *buffer, size_t max_size, LouderEncoding from_encoding, LouderEncoding to_encoding);
  LouderName __stdcall get_named_id(const LouderContext *ctx, const uint8_t *buffer, size_t max_size, LouderEncoding encoding);
  LouderName __stdcall declare_name(LouderContext *ctx, const uint8_t *buffer, size_t max_size, LouderEncoding encoding);
  int32_t __stdcall read_name(const LouderContext *ctx, LouderName name, uint8_t *buffer, size_t max_size, LouderEncoding encoding);
  LouderContext *__stdcall init_louder();
  int32_t __stdcall deinit_louder(LouderContext *ctx);
}

// Encodings the tests rely on, discovered by name at runtime so the tests
// don't depend on the crate's ordering of its encoding table.
static size_t g_utf8 = (size_t)-1;
static size_t g_windows_1252 = (size_t)-1;

static size_t find_encoding(const char *want) {
  char name[64];
  for (size_t e = 0; e < 512; e++) {
    memset(name, 0xAA, sizeof(name));
    get_encoding_name(e, (uint8_t *)name, sizeof(name) - 1);
    if (name[0] == '\0') {
      break;
    }
    if (strcmp(name, want) == 0) {
      return e;
    }
  }
  return (size_t)-1;
}

static size_t count_encodings() {
  size_t e = 0;
  size_t count = 1;
  while (true) {
    size_t next = get_next_encoding(e);
    if (next == 0) {
      break;
    }
    e = next;
    count++;
    if (count > 4096) {
      break;
    }
  }
  return count;
}

int test_test_linkage() {
  auto result = test_linkage();
  printf("test_linkage() = %d\n", result);
  
  if (result != 123) {
    printf("test_linkage() failed\n");
    return 1;
  }

  return 0;
}

int test_get_next_encoding() {
  char name[64];
  memset(name, 0xAA, sizeof(name));
  get_encoding_name(0, (uint8_t *)name, sizeof(name) - 1);
  if (name[0] == '\0' || name[0] == (char)0xAA) {
    printf("test_get_next_encoding: no encodings available\n");
    return 1;
  }

  // Walking from 0 stays contiguous until it wraps back to 0.
  size_t e = 0;
  size_t count = 1;
  while (true) {
    size_t next = get_next_encoding(e);
    if (next == 0) {
      break;
    }
    if (next != e + 1) {
      printf("test_get_next_encoding: jump %zu -> %zu, expected +1\n", e, next);
      return 1;
    }
    e = next;
    count++;
    if (count > 4096) {
      printf("test_get_next_encoding: iteration did not terminate\n");
      return 1;
    }
  }

  // Every index in [0, count) names something.
  for (size_t i = 0; i < count; i++) {
    memset(name, 0xAA, sizeof(name));
    get_encoding_name(i, (uint8_t *)name, sizeof(name) - 1);
    if (name[0] == '\0' || name[0] == (char)0xAA) {
      printf("test_get_next_encoding: index %zu has no name\n", i);
      return 1;
    }
  }

  // The first index past the last is empty.
  memset(name, 0xAA, sizeof(name));
  get_encoding_name(count, (uint8_t *)name, sizeof(name) - 1);
  if (name[0] != '\0') {
    printf("test_get_next_encoding: index %zu should be empty\n", count);
    return 1;
  }

  // Out-of-range start values return 0.
  if (get_next_encoding(count + 500) != 0) {
    printf("test_get_next_encoding: out-of-range start should return 0\n");
    return 1;
  }

  printf("test_get_next_encoding: %zu encodings\n", count);
  return 0;
}

int test_get_encoding_name() {
  if (g_utf8 == (size_t)-1) {
    printf("test_get_encoding_name: \"utf-8\" not found\n");
    return 1;
  }
  if (g_windows_1252 == (size_t)-1) {
    printf("test_get_encoding_name: \"windows-1252\" not found\n");
    return 1;
  }

  char name[64];
  memset(name, 0xAA, sizeof(name));
  get_encoding_name(g_utf8, (uint8_t *)name, sizeof(name) - 1);
  if (strcmp(name, "utf-8") != 0) {
    printf("test_get_encoding_name: index %zu = \"%s\", expected \"utf-8\"\n", g_utf8, name);
    return 1;
  }

  memset(name, 0xAA, sizeof(name));
  get_encoding_name(g_windows_1252, (uint8_t *)name, sizeof(name) - 1);
  if (strcmp(name, "windows-1252") != 0) {
    printf("test_get_encoding_name: index %zu = \"%s\", expected \"windows-1252\"\n", g_windows_1252, name);
    return 1;
  }

  // Truncated names stay NUL-terminated.
  char full[64];
  get_encoding_name(g_windows_1252, (uint8_t *)full, sizeof(full) - 1);
  char small[8];
  memset(small, 0, sizeof(small));
  get_encoding_name(g_windows_1252, (uint8_t *)small, 5);
  if (memcmp(small, full, 5) != 0 || small[5] != '\0') {
    printf("test_get_encoding_name: truncation produced wrong bytes\n");
    return 1;
  }

  // A zero-sized destination only gets a terminator.
  uint8_t one[1] = { 0x7F };
  get_encoding_name(g_utf8, one, 0);
  if (one[0] != '\0') {
    printf("test_get_encoding_name: zero-size buffer should get a NUL\n");
    return 1;
  }

  // Out-of-range indices write an empty string.
  uint8_t bucket[8];
  memset(bucket, 0x7F, sizeof(bucket));
  get_encoding_name((size_t)-1, bucket, sizeof(bucket) - 1);
  if (bucket[0] != '\0') {
    printf("test_get_encoding_name: invalid index should write \"\"\n");
    return 1;
  }

  printf("test_get_encoding_name: utf-8 @ %zu, windows-1252 @ %zu\n", g_utf8, g_windows_1252);
  return 0;
}

int test_change_encoding() {
  if (g_utf8 == (size_t)-1 || g_windows_1252 == (size_t)-1) {
    printf("test_change_encoding: required encodings unavailable\n");
    return 1;
  }

  // ASCII round-trips between utf-8 and windows-1252 unchanged.
  uint8_t ascii[8] = "Hello";
  size_t n = change_encoding(ascii, 5, g_utf8, g_windows_1252);
  if (n != 5 || memcmp(ascii, "Hello", 5) != 0 || ascii[5] != '\0') {
    printf("test_change_encoding: utf-8 -> windows-1252 of ASCII failed\n");
    return 1;
  }
  n = change_encoding(ascii, 5, g_windows_1252, g_utf8);
  if (n != 5 || memcmp(ascii, "Hello", 5) != 0 || ascii[5] != '\0') {
    printf("test_change_encoding: windows-1252 -> utf-8 of ASCII failed\n");
    return 1;
  }

  // "é" is 0xC3 0xA9 in utf-8 and 0xE9 in windows-1252.
  uint8_t accent[8] = { 0xC3, 0xA9, 0x00 };
  n = change_encoding(accent, 2, g_utf8, g_windows_1252);
  if (n != 1 || accent[0] != 0xE9 || accent[1] != '\0') {
    printf("test_change_encoding: utf-8 \"é\" -> windows-1252 failed\n");
    return 1;
  }
  // Out-of-place output larger than the buffer is truncated and terminated.
  n = change_encoding(accent, 1, g_windows_1252, g_utf8);
  if (n != 1 || accent[0] != 0xC3 || accent[1] != '\0') {
    printf("test_change_encoding: windows-1252 0xE9 -> utf-8 truncation failed\n");
    return 1;
  }

  // Invalid encodings fail without touching the buffer.
  size_t count = count_encodings();
  uint8_t intact[6];
  memset(intact, 0xAA, sizeof(intact));
  if (change_encoding(intact, 2, count + 500, g_utf8) != 0) {
    printf("test_change_encoding: invalid from_encoding should fail\n");
    return 1;
  }
  for (size_t i = 0; i < sizeof(intact); i++) {
    if (intact[i] != 0xAA) {
      printf("test_change_encoding: invalid from_encoding modified the buffer\n");
      return 1;
    }
  }
  memset(intact, 0xAA, sizeof(intact));
  if (change_encoding(intact, 2, g_utf8, count + 500) != 0) {
    printf("test_change_encoding: invalid to_encoding should fail\n");
    return 1;
  }
  for (size_t i = 0; i < sizeof(intact); i++) {
    if (intact[i] != 0xAA) {
      printf("test_change_encoding: invalid to_encoding modified the buffer\n");
      return 1;
    }
  }

  printf("test_change_encoding: ok\n");
  return 0;
}

int test_context_lifecycle() {
  if (deinit_louder(NULL) != LOUDER_ERROR_NOT_FOUND) {
    printf("test_context_lifecycle: deinit_louder(NULL) should return LOUDER_ERROR_NOT_FOUND\n");
    return 1;
  }

  // Two inits hand out distinct contexts.
  LouderContext *a = init_louder();
  LouderContext *b = init_louder();
  if (a == NULL || b == NULL) {
    printf("test_context_lifecycle: init_louder returned NULL\n");
    if (a) deinit_louder(a);
    if (b) deinit_louder(b);
    return 1;
  }
  if (a == b) {
    printf("test_context_lifecycle: two init_louder calls returned the same context\n");
    deinit_louder(a);
    deinit_louder(b);
    return 1;
  }

  if (deinit_louder(a) != LOUDER_SUCCESS || deinit_louder(b) != LOUDER_SUCCESS) {
    printf("test_context_lifecycle: deinit_louder should return LOUDER_SUCCESS\n");
    return 1;
  }

  printf("test_context_lifecycle: ok\n");
  return 0;
}

int test_names() {
  if (g_utf8 == (size_t)-1 || g_windows_1252 == (size_t)-1) {
    printf("test_names: required encodings unavailable\n");
    return 1;
  }

  const char *hello = "hello";
  const char *world = "world";

  // Null contexts are rejected.
  if (get_named_id(NULL, (const uint8_t *)hello, strlen(hello), g_utf8) != LOUDER_NAME_NONE) {
    printf("test_names: get_named_id(NULL, ...) should be LOUDER_NAME_NONE\n");
    return 1;
  }
  if (declare_name(NULL, (const uint8_t *)hello, strlen(hello), g_utf8) != LOUDER_NAME_NONE) {
    printf("test_names: declare_name(NULL, ...) should be LOUDER_NAME_NONE\n");
    return 1;
  }

  LouderContext *ctx = init_louder();
  if (ctx == NULL) {
    printf("test_names: init_louder() returned NULL\n");
    return 1;
  }
  int failed = 0;

  // Undeclared strings are unknown.
  if (get_named_id(ctx, (const uint8_t *)hello, strlen(hello), g_utf8) != LOUDER_NAME_NONE) {
    printf("test_names: undeclared string should be unknown\n");
    failed = 1;
  }

  // First declaration assigns a new id and get_named_id finds it.
  LouderName first = declare_name(ctx, (const uint8_t *)hello, strlen(hello), g_utf8);
  LouderName id = get_named_id(ctx, (const uint8_t *)hello, strlen(hello), g_utf8);
  if (first == LOUDER_NAME_NONE || first != id) {
    printf("test_names: first declare_name returned %d, id is %d\n", first, id);
    failed = 1;
  }
  // Redeclaring the same string fails.
  if (declare_name(ctx, (const uint8_t *)hello, strlen(hello), g_utf8) != LOUDER_NAME_NONE) {
    printf("test_names: redeclaring a name should return LOUDER_NAME_NONE\n");
    failed = 1;
  }

  // The name reads back in the declaring encoding.
  uint8_t out[64];
  memset(out, 0xAA, sizeof(out));
  int32_t n = read_name(ctx, id, out, sizeof(out) - 1, g_utf8);
  if (n != (int32_t)strlen(hello) || memcmp(out, hello, strlen(hello)) != 0 || out[strlen(hello)] != '\0') {
    printf("test_names: read_name round-trip failed\n");
    failed = 1;
  }

  // A different encoding reads nothing.
  memset(out, 0xAA, sizeof(out));
  if (read_name(ctx, id, out, sizeof(out) - 1, g_windows_1252) != 0) {
    printf("test_names: read_name with the wrong encoding should fail\n");
    failed = 1;
  }

  // Truncated reads are NUL-terminated.
  memset(out, 0xAA, sizeof(out));
  n = read_name(ctx, id, out, 3, g_utf8);
  if (n != 3 || memcmp(out, "hel", 3) != 0 || out[3] != '\0') {
    printf("test_names: read_name truncation failed\n");
    failed = 1;
  }

  // The same bytes under another encoding are a different name.
  declare_name(ctx, (const uint8_t *)hello, strlen(hello), g_windows_1252);
  LouderName latin_id = get_named_id(ctx, (const uint8_t *)hello, strlen(hello), g_windows_1252);
  if (latin_id == LOUDER_NAME_NONE || latin_id == id) {
    printf("test_names: encodings should not share names\n");
    failed = 1;
  }

  // Each distinct string gets its own id.
  LouderName world_id = declare_name(ctx, (const uint8_t *)world, strlen(world), g_utf8);
  if (world_id == LOUDER_NAME_NONE || world_id == id) {
    printf("test_names: second declare_name should assign a distinct id\n");
    failed = 1;
  }
  if (get_named_id(ctx, (const uint8_t *)world, strlen(world), g_utf8) != world_id) {
    printf("test_names: get_named_id should find world\n");
    failed = 1;
  }

  // Unknown names read nothing.
  if (read_name(ctx, LOUDER_NAME_NONE, out, sizeof(out) - 1, g_utf8) != 0) {
    printf("test_names: LOUDER_NAME_NONE should read nothing\n");
    failed = 1;
  }
  if (read_name(ctx, 123456789, out, sizeof(out) - 1, g_utf8) != 0) {
    printf("test_names: unregistered ids should read nothing\n");
    failed = 1;
  }
  if (read_name(NULL, id, out, sizeof(out) - 1, g_utf8) != 0) {
    printf("test_names: read_name(NULL, ...) should read nothing\n");
    failed = 1;
  }

  if (deinit_louder(ctx) != LOUDER_SUCCESS) {
    printf("test_names: deinit_louder failed\n");
    failed = 1;
  }

  printf("test_names: ok\n");
  return failed;
}

int (*TESTS[])() = {
  test_test_linkage,
  test_get_next_encoding,
  test_get_encoding_name,
  test_change_encoding,
  test_context_lifecycle,
  test_names,
  nullptr
};

int main() {
  g_utf8 = find_encoding("utf-8");
  g_windows_1252 = find_encoding("windows-1252");

  int failed = 0;
  for (int i = 0; TESTS[i] != nullptr; i++) {
    int result = TESTS[i]();
    if (result) {
      failed++;
    }
  }

  if (failed) {
    printf("%d tests failed\n", failed);
    return 1;
  }

  printf("All tests passed\n");
  return 0;
}