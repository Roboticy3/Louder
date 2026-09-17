#include <stdint.h>
#include <stdio.h>

//basically the only reason I'm using cpp is because I want to see if the function passes through this "extern C" stuff I'm not too familiar with.
//otherwise, this should be a .c file.
extern "C" {
  int32_t __stdcall test_linkage() {
    return 123;
  }
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

int (*TESTS[])() = {
  test_test_linkage,
  nullptr
};

int main() {
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