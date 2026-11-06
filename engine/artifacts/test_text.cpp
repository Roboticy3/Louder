#include "louder_rust.h"
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

int main() {
  LouderContext* ctx = init_louder();
  const char* doc = R"(A
Hello

B
World
tags: start

> start
)";
  LouderName conv = declare_name(ctx, (const uint8_t*)"conv", 4);
  std::vector<uint8_t> buf(doc, doc+strlen(doc));
  LoadError err = load_conversation_text(ctx, conv, buf.data(), buf.size());
  printf("err=%d\n", err);
  // positions
  for (int p=0;p<5;p++) {
    LouderName t = get_hub_text(ctx, conv, p);
    char b[256]; size_t w = read_name(ctx, t, (uint8_t*)b, 255); b[w]=0;
    LouderName s = get_hub_speaker(ctx, conv, p);
    char sb[256]; size_t sw = read_name(ctx, s, (uint8_t*)sb, 255); sb[sw]=0;
    printf("pos %d text='%s' speaker='%s'\n", p, b, sb);
  }
  deinit_louder(ctx);
  return 0;
}