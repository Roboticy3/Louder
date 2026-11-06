#include "louder_rust.h"
#include <cstdio>
#include <cstring>
#include <vector>
int main() {
  LouderContext* ctx = init_louder();
  const char* doc = "A\nHello\ntags: START\n";
  LouderName conv = declare_name(ctx, (const uint8_t*)"conv", 4);
  LoadError err = load_conversation_text(ctx, conv, (uint8_t*)doc, strlen(doc), false);
  printf("err=%d\n", err);
  LouderName tag = get_named_id(ctx, (const uint8_t*)"START", 5);
  printf("START tag exists (lowercase false): %d\n", tag != LOUDER_NAME_NONE);
  LouderName tag2 = get_named_id(ctx, (const uint8_t*)"start", 5);
  printf("start tag exists: %d\n", tag2 != LOUDER_NAME_NONE);
  deinit_louder(ctx);
  return 0;
}