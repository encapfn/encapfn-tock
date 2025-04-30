// Copyright lowRISC contributors (OpenTitan project).
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#ifndef OPENTITAN_SW_DEVICE_LIB_CRYPTO_INCLUDE_ENTROPY_H_
#define OPENTITAN_SW_DEVICE_LIB_CRYPTO_INCLUDE_ENTROPY_H_

#ifdef __cplusplus
extern "C" {
#endif  // __cplusplus

/**
 * Configures the entropy complex in continuous mode.
 *
 * The complex is configured in continuous mode with FIPS mode enabled. This is
 * the default operational mode of the entropy_src, csrng, edn0 and edn1 blocks.
 *
 * @return Operation status in `status_t` format.
 */
OT_WARN_UNUSED_RESULT
status_t entropy_complex_init(void);

/**
 * Ensures that the entropy complex is ready for use.
 *
 * Ensures that the entropy complex is running and that `entropy_src` is in
 * FIPS mode, and verifies the thresholds for health tests in `entropy_src`.
 * This function should be called periodically while the entropy complex is in
 * use, because the threshold registers are not shadowed.
 *
 * This check does not ensure that the SW CSRNG is in FIPS mode, so it is safe
 * to call it while using the SW CSRNG in manual mode. However, it is important
 * to note that passing the check does not by itself guarantee FIPS-compatible
 * entropy from CSRNG.
 *
 * @return Operation status in `status_t` format.
 */
OT_WARN_UNUSED_RESULT
status_t entropy_complex_check(void);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  // OPENTITAN_SW_DEVICE_LIB_CRYPTO_INCLUDE_ENTROPY_H_
