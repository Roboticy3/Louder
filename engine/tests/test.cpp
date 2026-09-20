#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

// Types mirroring louder_rust/src/lib.rs.
typedef int32_t LouderName;
typedef int32_t LouderActor;
typedef int32_t LouderPosition;
typedef size_t LouderEncoding;
struct LouderContext;

#define LOUDER_NAME_NONE ((LouderName)-1)
#define LOUDER_ACTOR_NONE ((LouderActor)-1)
#define LOUDER_SUCCESS 0
#define LOUDER_ERROR_NOT_FOUND (-1)

// Event codes: discriminants of louder's EventData enum.
#define EVENT_CODE_NONE 0u
#define EVENT_CODE_REGULAR_MOTION 1u
#define EVENT_CODE_WARP 2u
#define EVENT_CODE_FORCE_MOTION 3u
#define EVENT_CODE_CHOICE_APPEARANCE 4u
#define EVENT_CODE_CHANNEL_CHANGE 5u
#define EVENT_CODE_ADDITIONAL_DATA_CHANGE 6u
#define EVENT_CODE_SPEAKER_CHANGE 7u
#define EVENT_CODE_TEXT_CHANGE 8u
#define EVENT_CODE_STATEMENT 9u
#define EVENT_CODE_CONDITION 10u
#define EVENT_CODE_CALLBACK 11u
#define EVENT_CODE_ACTOR_ADDED 12u
#define EVENT_CODE_ACTOR_REMOVED 13u
#define EVENT_CODE_SPEAKER_ADDED 14u
#define EVENT_CODE_SPEAKER_REMOVED 15u
#define EVENT_CODE_CONVERSATION_ADDED 16u
#define EVENT_CODE_CONVERSATION_REMOVED 17u

// Condition outcome variant types (LouderVariantType).
#define VARIANT_TYPE_INT 0u
#define VARIANT_TYPE_FLOAT 1u
#define VARIANT_TYPE_STRING 2u

// Keys passed to the event getters.
#define EVENT_POSITION_SOURCE 0
#define EVENT_POSITION_TARGET 1
#define EVENT_POSITION_CHOICE_APPEARANCE 2
#define EVENT_INT_SOURCE_CHOICE_INDEX 0
#define EVENT_INT_CHOICE_INDEX 1

// Keys passed to the ask getters.
#define ASK_NAME_CONVERSATION 0
#define ASK_NAME_CHANNEL 1
#define ASK_NAME_CALLBACK 2
#define ASK_INT_ACTOR 0
#define ASK_INT_SOURCE_CHOICE_INDEX 2
#define ASK_POSITION_SOURCE 0
#define ASK_POSITION_TARGET 1

// Compact assertion used by the tests below.
#define CHECK(cond) do { \
  if (!(cond)) { \
    printf("%s:%d: check failed: %s\n", __FUNCTION__, __LINE__, #cond); \
    failed = 1; \
  } \
} while (0)

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
  size_t __stdcall get_encoding_name(LouderEncoding index, uint8_t *buffer, size_t max_size);
  size_t __stdcall change_encoding(uint8_t *buffer, size_t max_size, LouderEncoding from_encoding, LouderEncoding to_encoding);
  LouderName __stdcall get_named_id(const LouderContext *ctx, const uint8_t *buffer, size_t max_size, LouderEncoding encoding);
  LouderName __stdcall declare_name(LouderContext *ctx, const uint8_t *buffer, size_t max_size, LouderEncoding encoding);
  int32_t __stdcall read_name(const LouderContext *ctx, LouderName name, uint8_t *buffer, size_t max_size, LouderEncoding encoding);

  // Event query and test-push functions.
  int32_t __stdcall is_event(const LouderContext *ctx, size_t index);
  uint32_t __stdcall get_event_code(const LouderContext *ctx, size_t index);
  LouderName __stdcall get_event_additional_data_changed(const LouderContext *ctx, size_t index);
  LouderActor __stdcall get_event_actor(const LouderContext *ctx, size_t index);
  LouderPosition __stdcall get_event_position(const LouderContext *ctx, size_t index, int32_t key);
  int32_t __stdcall get_event_int(const LouderContext *ctx, size_t index, int32_t key);
  uint8_t __stdcall get_event_background(const LouderContext *ctx, size_t index);
  uint32_t __stdcall get_event_condition_variant_type(const LouderContext *ctx, size_t index, int32_t condition_index);
  int32_t __stdcall get_event_condition_int(const LouderContext *ctx, size_t index, int32_t condition_index);
  float __stdcall get_event_condition_float(const LouderContext *ctx, size_t index, int32_t condition_index);
  LouderName __stdcall get_event_condition_string(const LouderContext *ctx, size_t index, int32_t condition_index);
  void __stdcall flush_events(LouderContext *ctx);
  int32_t __stdcall test_push_event(
      LouderContext *ctx,
      LouderName conversation,
      LouderName channel,
      LouderActor actor,
      uint32_t code,
      LouderPosition source_position,
      LouderPosition target_position,
      int32_t choice_index,
      uint8_t background,
      LouderPosition choice_position,
      LouderName key,
      LouderName name,
      LouderName statement,
      LouderName condition,
      uint32_t condition_variant_type,
      int32_t condition_int,
      float condition_float,
      LouderName condition_string,
      LouderName callback);

  // Ask query and test-push functions.
  int32_t __stdcall is_ask(const LouderContext *ctx, size_t index);
  LouderName __stdcall get_ask_name(const LouderContext *ctx, size_t index, int32_t key);
  LouderActor __stdcall get_ask_actor(const LouderContext *ctx, size_t index);
  LouderPosition __stdcall get_ask_position(const LouderContext *ctx, size_t index, int32_t key);
  int32_t __stdcall get_ask_int(const LouderContext *ctx, size_t index, int32_t key);
  LouderName __stdcall get_ask_callback(const LouderContext *ctx, size_t index);
  int32_t __stdcall answer_ask(LouderContext *ctx, size_t index, int32_t verdict);
  int32_t __stdcall push_ask(
      LouderContext *ctx,
      LouderName conversation,
      LouderName channel,
      LouderActor actor,
      LouderPosition source_position,
      LouderPosition target_position,
      int32_t choice_index,
      uint8_t background,
      LouderName callback);

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

int test_events() {
  if (g_utf8 == (size_t)-1) {
    printf("test_events: utf-8 encoding unavailable\n");
    return 1;
  }

  LouderContext *ctx = init_louder();
  if (ctx == NULL) {
    printf("test_events: init_louder() returned NULL\n");
    return 1;
  }
  int failed = 0;

  LouderName conv = declare_name(ctx, (const uint8_t *)"conv", 4, g_utf8);
  LouderName chan = declare_name(ctx, (const uint8_t *)"chan", 4, g_utf8);
  LouderName key = declare_name(ctx, (const uint8_t *)"key", 3, g_utf8);
  LouderName cond = declare_name(ctx, (const uint8_t *)"cond", 4, g_utf8);
  LouderName expr = declare_name(ctx, (const uint8_t *)"expr", 4, g_utf8);
  LouderName cb = declare_name(ctx, (const uint8_t *)"cb", 2, g_utf8);
  CHECK(conv != LOUDER_NAME_NONE && chan != LOUDER_NAME_NONE && key != LOUDER_NAME_NONE);
  CHECK(cond != LOUDER_NAME_NONE && expr != LOUDER_NAME_NONE && cb != LOUDER_NAME_NONE);

  // A fresh context has no events.
  CHECK(is_event(ctx, 0) == LOUDER_ERROR_NOT_FOUND);

  // 0: RegularMotion, source 10, target 20, choice 3, in the background.
  int32_t idx = test_push_event(ctx, conv, chan, 7, EVENT_CODE_REGULAR_MOTION,
      10, 20, 3, 1, -1, -1, -1, -1, -1, VARIANT_TYPE_INT, -1, 0.0f, -1, -1);
  CHECK(idx == 0);
  CHECK(is_event(ctx, 0) == LOUDER_SUCCESS);
  CHECK(get_event_code(ctx, 0) == EVENT_CODE_REGULAR_MOTION);
  CHECK(get_event_actor(ctx, 0) == 7);
  CHECK(get_event_position(ctx, 0, EVENT_POSITION_SOURCE) == 10);
  CHECK(get_event_position(ctx, 0, EVENT_POSITION_TARGET) == 20);
  CHECK(get_event_position(ctx, 0, EVENT_POSITION_CHOICE_APPEARANCE) == -1);
  CHECK(get_event_int(ctx, 0, EVENT_INT_SOURCE_CHOICE_INDEX) == 3);
  CHECK(get_event_int(ctx, 0, EVENT_INT_CHOICE_INDEX) == -1);
  CHECK(get_event_background(ctx, 0) == 1);
  CHECK(get_event_additional_data_changed(ctx, 0) == LOUDER_NAME_NONE);

  // 1: Warp carries the same motion fields.
  idx = test_push_event(ctx, conv, chan, 8, EVENT_CODE_WARP,
      30, 40, 4, 0, -1, -1, -1, -1, -1, VARIANT_TYPE_INT, -1, 0.0f, -1, -1);
  CHECK(idx == 1);
  CHECK(get_event_code(ctx, 1) == EVENT_CODE_WARP);
  CHECK(get_event_actor(ctx, 1) == 8);
  CHECK(get_event_position(ctx, 1, EVENT_POSITION_SOURCE) == 30);
  CHECK(get_event_position(ctx, 1, EVENT_POSITION_TARGET) == 40);
  CHECK(get_event_int(ctx, 1, EVENT_INT_SOURCE_CHOICE_INDEX) == 4);
  CHECK(get_event_background(ctx, 1) == 0);

  // 2: ForceMotion shares the motion layout.
  idx = test_push_event(ctx, conv, chan, 9, EVENT_CODE_FORCE_MOTION,
      50, 60, -1, 1, -1, -1, -1, -1, -1, VARIANT_TYPE_INT, -1, 0.0f, -1, -1);
  CHECK(idx == 2);
  CHECK(get_event_code(ctx, 2) == EVENT_CODE_FORCE_MOTION);
  CHECK(get_event_actor(ctx, 2) == 9);
  CHECK(get_event_position(ctx, 2, EVENT_POSITION_SOURCE) == 50);
  CHECK(get_event_position(ctx, 2, EVENT_POSITION_TARGET) == 60);
  CHECK(get_event_int(ctx, 2, EVENT_INT_SOURCE_CHOICE_INDEX) == -1);
  CHECK(get_event_background(ctx, 2) == 1);

  // 3: ChoiceAppearance, position 99, choice 5.
  idx = test_push_event(ctx, conv, chan, 10, EVENT_CODE_CHOICE_APPEARANCE,
      -1, -1, 5, 0, 99, -1, -1, -1, -1, VARIANT_TYPE_INT, -1, 0.0f, -1, -1);
  CHECK(idx == 3);
  CHECK(get_event_code(ctx, 3) == EVENT_CODE_CHOICE_APPEARANCE);
  CHECK(get_event_actor(ctx, 3) == 10);
  CHECK(get_event_position(ctx, 3, EVENT_POSITION_CHOICE_APPEARANCE) == 99);
  CHECK(get_event_position(ctx, 3, EVENT_POSITION_SOURCE) == -1);
  CHECK(get_event_position(ctx, 3, EVENT_POSITION_TARGET) == -1);
  CHECK(get_event_int(ctx, 3, EVENT_INT_CHOICE_INDEX) == 5);
  CHECK(get_event_int(ctx, 3, EVENT_INT_SOURCE_CHOICE_INDEX) == -1);
  CHECK(get_event_background(ctx, 3) == 0);

  // 4: AdditionalDataChange stores the changed key.
  idx = test_push_event(ctx, conv, chan, 11, EVENT_CODE_ADDITIONAL_DATA_CHANGE,
      -1, -1, -1, 0, -1, key, -1, -1, -1, VARIANT_TYPE_INT, -1, 0.0f, -1, -1);
  CHECK(idx == 4);
  CHECK(get_event_code(ctx, 4) == EVENT_CODE_ADDITIONAL_DATA_CHANGE);
  CHECK(get_event_actor(ctx, 4) == 11);
  CHECK(get_event_additional_data_changed(ctx, 4) == key);
  CHECK(get_event_additional_data_changed(ctx, 0) == LOUDER_NAME_NONE);

  // 5: Condition evaluating to Int(42).
  idx = test_push_event(ctx, conv, chan, 12, EVENT_CODE_CONDITION,
      -1, -1, -1, 0, -1, -1, -1, expr, cond, VARIANT_TYPE_INT, 42, 99.0f, cb, -1);
  CHECK(idx == 5);
  CHECK(get_event_code(ctx, 5) == EVENT_CODE_CONDITION);
  CHECK(get_event_actor(ctx, 5) == 12);
  CHECK(get_event_condition_variant_type(ctx, 5, 0) == VARIANT_TYPE_INT);
  CHECK(get_event_condition_int(ctx, 5, 0) == 42);
  CHECK(get_event_condition_float(ctx, 5, 0) == -1.0f);
  CHECK(get_event_condition_string(ctx, 5, 0) == LOUDER_NAME_NONE);
  // Only condition_index 0 is meaningful.
  CHECK(get_event_condition_variant_type(ctx, 5, 1) == VARIANT_TYPE_INT);
  CHECK(get_event_condition_int(ctx, 5, 1) == -1);

  // 6: Condition evaluating to Float(3.5).
  idx = test_push_event(ctx, conv, chan, 13, EVENT_CODE_CONDITION,
      -1, -1, -1, 0, -1, -1, -1, -1, cond, VARIANT_TYPE_FLOAT, -1, 3.5f, -1, -1);
  CHECK(idx == 6);
  CHECK(get_event_condition_variant_type(ctx, 6, 0) == VARIANT_TYPE_FLOAT);
  CHECK(get_event_condition_int(ctx, 6, 0) == -1);
  CHECK(get_event_condition_float(ctx, 6, 0) == 3.5f);
  CHECK(get_event_condition_string(ctx, 6, 0) == LOUDER_NAME_NONE);

  // 7: Condition evaluating to String(key).
  idx = test_push_event(ctx, conv, chan, 14, EVENT_CODE_CONDITION,
      -1, -1, -1, 0, -1, -1, -1, -1, cond, VARIANT_TYPE_STRING, -1, -1.0f, key, -1);
  CHECK(idx == 7);
  CHECK(get_event_condition_variant_type(ctx, 7, 0) == VARIANT_TYPE_STRING);
  CHECK(get_event_condition_int(ctx, 7, 0) == -1);
  CHECK(get_event_condition_float(ctx, 7, 0) == -1.0f);
  CHECK(get_event_condition_string(ctx, 7, 0) == key);

  // 8: Callback event just carries the code and actor.
  idx = test_push_event(ctx, conv, chan, 15, EVENT_CODE_CALLBACK,
      -1, -1, -1, 0, -1, -1, -1, -1, -1, VARIANT_TYPE_INT, -1, 0.0f, -1, cb);
  CHECK(idx == 8);
  CHECK(get_event_code(ctx, 8) == EVENT_CODE_CALLBACK);
  CHECK(get_event_actor(ctx, 8) == 15);

  // 9-17: the remaining codes exist and map to the right discriminants.
  static const uint32_t codes[] = {
    EVENT_CODE_CHANNEL_CHANGE,
    EVENT_CODE_TEXT_CHANGE,
    EVENT_CODE_STATEMENT,
    EVENT_CODE_ACTOR_ADDED,
    EVENT_CODE_ACTOR_REMOVED,
    EVENT_CODE_SPEAKER_ADDED,
    EVENT_CODE_SPEAKER_REMOVED,
    EVENT_CODE_CONVERSATION_ADDED,
    EVENT_CODE_CONVERSATION_REMOVED,
  };
  for (size_t i = 0; i < sizeof(codes) / sizeof(codes[0]); i++) {
    int32_t pushed = test_push_event(ctx, conv, chan, (LouderActor)(100 + i), codes[i],
        -1, -1, -1, 0, -1, -1, -1, -1, -1, VARIANT_TYPE_INT, -1, 0.0f, -1, -1);
    CHECK(pushed == (int32_t)(9 + i));
    CHECK(is_event(ctx, (size_t)(9 + i)) == LOUDER_SUCCESS);
    CHECK(get_event_code(ctx, (size_t)(9 + i)) == codes[i]);
  }

  // No event exists just past the queue.
  CHECK(is_event(ctx, 18) == LOUDER_ERROR_NOT_FOUND);

  // Invalid indexes report as missing with default values.
  CHECK(is_event(ctx, 9999) == LOUDER_ERROR_NOT_FOUND);
  CHECK(get_event_code(ctx, 9999) == EVENT_CODE_NONE);
  CHECK(get_event_actor(ctx, 9999) == LOUDER_ACTOR_NONE);
  CHECK(get_event_additional_data_changed(ctx, 9999) == LOUDER_NAME_NONE);
  CHECK(get_event_position(ctx, 9999, EVENT_POSITION_SOURCE) == -1);
  CHECK(get_event_position(ctx, 9999, EVENT_POSITION_TARGET) == -1);
  CHECK(get_event_position(ctx, 9999, EVENT_POSITION_CHOICE_APPEARANCE) == -1);
  CHECK(get_event_int(ctx, 9999, EVENT_INT_SOURCE_CHOICE_INDEX) == -1);
  CHECK(get_event_int(ctx, 9999, EVENT_INT_CHOICE_INDEX) == -1);
  CHECK(get_event_background(ctx, 9999) == 0);
  CHECK(get_event_condition_variant_type(ctx, 9999, 0) == VARIANT_TYPE_INT);
  CHECK(get_event_condition_int(ctx, 9999, 0) == -1);
  CHECK(get_event_condition_float(ctx, 9999, 0) == -1.0f);
  CHECK(get_event_condition_string(ctx, 9999, 0) == LOUDER_NAME_NONE);

  // Null contexts fail gracefully with defaults.
  CHECK(is_event(NULL, 0) == LOUDER_ERROR_NOT_FOUND);
  CHECK(get_event_code(NULL, 0) == EVENT_CODE_NONE);
  CHECK(get_event_actor(NULL, 0) == LOUDER_ACTOR_NONE);
  CHECK(get_event_additional_data_changed(NULL, 0) == LOUDER_NAME_NONE);
  CHECK(get_event_position(NULL, 0, EVENT_POSITION_SOURCE) == -1);
  CHECK(get_event_int(NULL, 0, EVENT_INT_CHOICE_INDEX) == -1);
  CHECK(get_event_background(NULL, 0) == 0);
  CHECK(get_event_condition_variant_type(NULL, 0, 0) == VARIANT_TYPE_INT);
  CHECK(get_event_condition_float(NULL, 0, 0) == -1.0f);
  CHECK(get_event_condition_string(NULL, 0, 0) == LOUDER_NAME_NONE);
  flush_events(NULL);
  CHECK(test_push_event(NULL, conv, chan, -1, EVENT_CODE_NONE,
      -1, -1, -1, 0, -1, -1, -1, -1, -1, VARIANT_TYPE_INT, -1, 0.0f, -1, -1) == LOUDER_ERROR_NOT_FOUND);

  // flush_events drains both queued events and asks.
  flush_events(ctx);
  CHECK(is_event(ctx, 0) == LOUDER_ERROR_NOT_FOUND);
  CHECK(is_event(ctx, 17) == LOUDER_ERROR_NOT_FOUND);
  idx = test_push_event(ctx, conv, chan, 7, EVENT_CODE_WARP,
      1, 2, -1, 0, -1, -1, -1, -1, -1, VARIANT_TYPE_INT, -1, 0.0f, -1, -1);
  CHECK(idx == 0);
  CHECK(is_event(ctx, 0) == LOUDER_SUCCESS);

  CHECK(deinit_louder(ctx) == LOUDER_SUCCESS);
  if (failed) {
    printf("test_events: failed\n");
  } else {
    printf("test_events: ok\n");
  }
  return failed;
}

int test_asks() {
  if (g_utf8 == (size_t)-1) {
    printf("test_asks: utf-8 encoding unavailable\n");
    return 1;
  }

  LouderContext *ctx = init_louder();
  if (ctx == NULL) {
    printf("test_asks: init_louder() returned NULL\n");
    return 1;
  }
  int failed = 0;

  LouderName conv = declare_name(ctx, (const uint8_t *)"conv", 4, g_utf8);
  LouderName chan = declare_name(ctx, (const uint8_t *)"chan", 4, g_utf8);
  LouderName cb = declare_name(ctx, (const uint8_t *)"cb", 2, g_utf8);
  CHECK(conv != LOUDER_NAME_NONE && chan != LOUDER_NAME_NONE && cb != LOUDER_NAME_NONE);

  // A fresh context has no asks.
  CHECK(is_ask(ctx, 0) == LOUDER_ERROR_NOT_FOUND);

  // 0: a full ask, source 10, target 20, choice 3.
  int32_t idx = push_ask(ctx, conv, chan, 7, 10, 20, 3, 1, cb);
  CHECK(idx == 0);
  CHECK(is_ask(ctx, 0) == LOUDER_SUCCESS);
  CHECK(get_ask_name(ctx, 0, ASK_NAME_CONVERSATION) == conv);
  CHECK(get_ask_name(ctx, 0, ASK_NAME_CHANNEL) == chan);
  CHECK(get_ask_name(ctx, 0, ASK_NAME_CALLBACK) == cb);
  CHECK(get_ask_actor(ctx, 0) == 7);
  CHECK(get_ask_int(ctx, 0, ASK_INT_ACTOR) == 7);
  CHECK(get_ask_int(ctx, 0, ASK_INT_SOURCE_CHOICE_INDEX) == 3);
  CHECK(get_ask_position(ctx, 0, ASK_POSITION_SOURCE) == 10);
  CHECK(get_ask_position(ctx, 0, ASK_POSITION_TARGET) == 20);
  CHECK(get_ask_callback(ctx, 0) == cb);

  // 1: a second ask with a different actor and no chosen choice.
  idx = push_ask(ctx, conv, chan, 8, 30, 40, -1, 0, cb);
  CHECK(idx == 1);
  CHECK(get_ask_actor(ctx, 1) == 8);
  CHECK(get_ask_int(ctx, 1, ASK_INT_SOURCE_CHOICE_INDEX) == -1);
  CHECK(get_ask_position(ctx, 1, ASK_POSITION_SOURCE) == 30);
  CHECK(get_ask_position(ctx, 1, ASK_POSITION_TARGET) == 40);

  // Invalid indexes fail with defaults.
  CHECK(is_ask(ctx, 9999) == LOUDER_ERROR_NOT_FOUND);
  CHECK(get_ask_name(ctx, 9999, ASK_NAME_CONVERSATION) == LOUDER_NAME_NONE);
  CHECK(get_ask_name(ctx, 9999, ASK_NAME_CHANNEL) == LOUDER_NAME_NONE);
  CHECK(get_ask_name(ctx, 9999, ASK_NAME_CALLBACK) == LOUDER_NAME_NONE);
  CHECK(get_ask_actor(ctx, 9999) == LOUDER_ACTOR_NONE);
  CHECK(get_ask_int(ctx, 9999, ASK_INT_ACTOR) == -1);
  CHECK(get_ask_int(ctx, 9999, ASK_INT_SOURCE_CHOICE_INDEX) == -1);
  CHECK(get_ask_position(ctx, 9999, ASK_POSITION_SOURCE) == -1);
  CHECK(get_ask_position(ctx, 9999, ASK_POSITION_TARGET) == -1);
  CHECK(get_ask_callback(ctx, 9999) == LOUDER_NAME_NONE);

  // Null contexts fail gracefully with defaults.
  CHECK(is_ask(NULL, 0) == LOUDER_ERROR_NOT_FOUND);
  CHECK(get_ask_name(NULL, 0, ASK_NAME_CONVERSATION) == LOUDER_NAME_NONE);
  CHECK(get_ask_actor(NULL, 0) == LOUDER_ACTOR_NONE);
  CHECK(get_ask_int(NULL, 0, ASK_INT_ACTOR) == -1);
  CHECK(get_ask_position(NULL, 0, ASK_POSITION_TARGET) == -1);
  CHECK(get_ask_callback(NULL, 0) == LOUDER_NAME_NONE);
  CHECK(push_ask(NULL, conv, chan, -1, -1, -1, -1, 0, cb) == LOUDER_ERROR_NOT_FOUND);

  // answer_ask without a valid ask returns LOUDER_ERROR_NOT_FOUND.
  CHECK(answer_ask(NULL, 0, 1) == LOUDER_ERROR_NOT_FOUND);
  CHECK(answer_ask(ctx, 9999, 1) == LOUDER_ERROR_NOT_FOUND);

  // flush_events drains the ask cache too.
  flush_events(ctx);
  CHECK(is_ask(ctx, 0) == LOUDER_ERROR_NOT_FOUND);
  CHECK(is_ask(ctx, 1) == LOUDER_ERROR_NOT_FOUND);
  idx = push_ask(ctx, conv, chan, 9, 1, 2, 0, 0, cb);
  CHECK(idx == 0);
  CHECK(is_ask(ctx, 0) == LOUDER_SUCCESS);

  CHECK(deinit_louder(ctx) == LOUDER_SUCCESS);
  if (failed) {
    printf("test_asks: failed\n");
  } else {
    printf("test_asks: ok\n");
  }
  return failed;
}

int (*TESTS[])() = {
  test_test_linkage,
  test_get_next_encoding,
  test_get_encoding_name,
  test_change_encoding,
  test_context_lifecycle,
  test_names,
  test_events,
  test_asks,
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