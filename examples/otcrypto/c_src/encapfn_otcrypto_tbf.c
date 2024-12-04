#include "encapfn_otcrypto_tbf.h"

typedef void (*fnptr)(void);

fnptr const
__attribute__ ((section (".encapfn_hdr")))
encapfn_fntab[14] = {
  /* 0 */ (fnptr) keyblob_num_words,
  /* 1 */ (fnptr) keyblob_from_key_and_mask,
  /* 2 */ (fnptr) integrity_blinded_checksum,
  /* 3 */ (fnptr) otcrypto_hmac_init,
  /* 4 */ (fnptr) otcrypto_hmac_update,
  /* 5 */ (fnptr) otcrypto_hmac_final,
  /* 6 */ (fnptr) entropy_complex_init,
};

__attribute__ ((section (".encapfn_hdr")))
const size_t encapfn_fntab_length = sizeof(encapfn_fntab);
