#include <stdbool.h>
#include <stddef.h>

int demo_nop(int a, bool *boolarray);
void demo_invoke_callback(size_t loop_iters, void (*callback_fn)(size_t a, size_t b, size_t c));
