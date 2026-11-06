#include "louder_rust.h"
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>
int main() {
  LouderContext* ctx = init_louder();
  const char* doc = R"(start here
tags: start

second
tags: target

third
tags: target

first
tags: first
> target

link hub
link: target
tags: linky
)";
  LouderName conv = declare_name(ctx, (const uint8_t*)"conv", 4);
  LoadError err = load_conversation_text(ctx, conv, (uint8_t*)doc, strlen(doc));
  printf("err=%d\n", err);
  // positions
  for (int p=0;p<6;p++) {
    LouderName t = get_hub_text(ctx, conv, p);
    char b[256]; size_t w = read_name(ctx, t, (uint8_t*)b, 255); b[w]=0;
    LouderName s = get_hub_speaker(ctx, conv, p);
    char sb[256]; size_t sw = read_name(ctx, s, (uint8_t*)sb, 255); sb[sw]=0;
    printf("pos %d text='%s' speaker='%s' link=%d\n", p, b, sb, get_hub_link(ctx, conv, p));
    // choices count for actor none? use 0? We'll just print links via probing maybe
  }
  deinit_louder(ctx);
  return 0;
}