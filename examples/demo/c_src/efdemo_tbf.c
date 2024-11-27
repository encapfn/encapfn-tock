#include "demo.h"
#include <stddef.h>

typedef void (*fnptr)(void);

fnptr const
__attribute__ ((section (".encapfn_hdr")))
encapfn_fntab[2] = {
    /* 0 */ (fnptr) demo_nop,
    /* 1 */ (fnptr) demo_invoke_callback,
};

__attribute__ ((section (".encapfn_hdr")))
const size_t encapfn_fntab_length = sizeof(encapfn_fntab);
