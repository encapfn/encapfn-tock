#include "demo.h"

void demo_nop(void) {}

void demo_invoke_callback(void (*callback_fn)()) {
    callback_fn();
}
