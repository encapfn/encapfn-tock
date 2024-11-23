#include "demo.h"

int demo_nop(int a, bool *boolarray) {
    ((unsigned char *) boolarray)[0] = 1;

    // Uncomment to have validation fail:
    // ((unsigned char *) boolarray)[0] = 1;

    return a + 42;
}

void demo_invoke_callback(void (*callback_fn)()) {
  callback_fn();
}
