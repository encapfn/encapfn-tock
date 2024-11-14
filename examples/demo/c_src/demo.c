#include "demo.h"

int demo_nop(int a, _Bool *boolarray) {
    ((unsigned char *) boolarray)[0] = 1;

    // Uncomment to have validation fail:
    // ((unsigned char *) boolarray)[0] = 1;

    return a + 42;
}
