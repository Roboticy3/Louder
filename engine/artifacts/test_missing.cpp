#include "louder_rust.h"
#include <cstdio>
#include <cstring>
#include <vector>
int main() {
  LouderContext* ctx = init_louder();
  const char* doc = "first\n> nope\n\nsecond\ntags: other\n";
  LouderName conv = declare_name(ctx, (const uint8_t*)"conv", 4);
  LoadError err = load_conversation_text(ctx, conv, (uint8_t*)doc, strlen(doc));
  printf("err=%d\n", err);
  std::vector<LoadFailure> failures;
  // drain events
  for (size_t i=0;;++i) {
    LouderOption<LouderEvent> opt = get_event(ctx, i);
    if (opt.tag == LouderOption<LouderEvent>::Tag::None) break;
    if (opt.some._0.data.tag == EventData::Tag::LoadError) failures.push_back(opt.some._0.data.load_error._0);
  }
  flush_events(ctx);
  printf("failures=%zu\n", failures.size());
  for (auto& f : failures) printf("pos=%d err=%d\n", f.position, f.error);
  deinit_louder(ctx);
  return 0;
}