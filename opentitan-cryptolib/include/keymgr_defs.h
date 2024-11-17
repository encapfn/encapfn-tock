// Copyright lowRISC contributors (OpenTitan project).
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#ifndef OPENTITAN_SW_DEVICE_LIB_CRYPTO_INCLUDE_KEYMGR_DEFS_H_
#define OPENTITAN_SW_DEVICE_LIB_CRYPTO_INCLUDE_KEYMGR_DEFS_H_

#ifdef __cplusplus
extern "C" {
#endif  // __cplusplus

enum {
  /**
   * Number of 32-bit words for the salt.
   */
  kKeymgrSaltNumWords = 8,
  /**
   * Number of 32-bit words for each output key share.
   */
  kKeymgrOutputShareNumWords = 8,
};

/**
 * Data used to differentiate a generated keymgr key.
 */
typedef struct keymgr_diversification {
  /**
   * Salt value to use for key generation.
   */
  uint32_t salt[kKeymgrSaltNumWords];
  /**
   * Version for key generation (anti-rollback protection).
   */
  uint32_t version;
} keymgr_diversification_t;

/**
 * Generated key from keymgr.
 *
 * The output key material is 256 bits, generated in two shares.
 */
typedef struct keymgr_output {
  uint32_t share0[kKeymgrOutputShareNumWords];
  uint32_t share1[kKeymgrOutputShareNumWords];
} keymgr_output_t;

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  // OPENTITAN_SW_DEVICE_LIB_CRYPTO_INCLUDE_KEYMGR_DEFS_H_
