#include "demo.h"

int demo_nop(int a, bool *boolarray) {
    ((unsigned char *) boolarray)[0] = 1;

    // Uncomment to have validation fail:
    // ((unsigned char *) boolarray)[0] = 1;

    return a + 42;
}

void demo_invoke_callback(size_t loop_iters, void (*callback_fn)(size_t a, size_t b, size_t c)) {
    for (size_t i = 0; i < loop_iters; i++) {
        callback_fn(0xDEADBEEF, 0xCAFEBABE, i);
    }
}
