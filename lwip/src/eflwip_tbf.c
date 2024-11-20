#include <lwip/init.h>
#include <lwip/netif.h>
#include <stddef.h>

typedef void (*fnptr)(void);

fnptr const
__attribute__ ((section (".encapfn_hdr")))
encapfn_fntab[2] = {
    /* 0 */ (fnptr) lwip_init,
    /* 1 */ (fnptr) netif_add,
    /* 2 */ (fnptr) netif_get_by_index,
};

__attribute__ ((section (".encapfn_hdr")))
const size_t encapfn_fntab_length = sizeof(encapfn_fntab);
