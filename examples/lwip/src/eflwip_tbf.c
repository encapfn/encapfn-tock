#include <lwip/init.h>
#include <lwip/netif.h>
#include <lwip/dhcp.h>
#include <lwip/timeouts.h>
#include <stddef.h>

uint32_t sys_now() {
    return 1000;
}

typedef void (*fnptr)(void);

fnptr const
__attribute__ ((section (".encapfn_hdr")))
encapfn_fntab[15] = {
    /* 0  */ (fnptr) lwip_init,
    /* 1  */ (fnptr) netif_add,
    /* 2  */ (fnptr) netif_get_by_index,
    /* 3  */ (fnptr) netif_input,
    /* 4  */ (fnptr) netif_set_default,
    /* 5  */ (fnptr) netif_set_up,
    /* 6  */ (fnptr) pbuf_alloc,
    /* 7  */ (fnptr) pbuf_take,
    /* 8  */ (fnptr) dhcp_start,
    /* 9  */ (fnptr) sys_check_timeouts,
    /* 10 */ (fnptr) make_ip_addr_t,
    /* 11 */ (fnptr) etharp_add_static_entry,
    /* 12 */ (fnptr) make_ip4_addr_t,
    /* 13 */ (fnptr) netif_set_link_up,
    /* 14 */ (fnptr) etharp_output,
};

__attribute__ ((section (".encapfn_hdr")))
const size_t encapfn_fntab_length = sizeof(encapfn_fntab);
