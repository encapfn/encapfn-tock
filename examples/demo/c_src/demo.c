#include "demo.h"
#include "mac.h"

int demo_nop(int a, bool *boolarray) {
    otcrypto_kmac_mode_t kmac_mode = kOtcryptoKmacModeKmac128;
    otcrypto_blinded_key_t key;
    otcrypto_const_byte_buf_t input_message;
    otcrypto_word32_buf_t tag;

    otcrypto_hmac(&key, input_message, tag);

    ((unsigned char *) boolarray)[0] = 1;

    // Uncomment to have validation fail:
    // ((unsigned char *) boolarray)[0] = 1;

    return a + 42;
}
