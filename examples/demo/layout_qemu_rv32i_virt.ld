TBF_HEADER_SIZE = 0x60;

FLASH_START  = 0x80100000;
FLASH_LENGTH = 0x00100000;
RAM_START    = 0x80300000;
RAM_LENGTH   = 0x00100000;

/* Stack size, including values stacked by the Rust runtime */
STACK_SIZE   = 32 * 1024;

INCLUDE ../../encapfn-tock/encapfn_c_rt/encapfn_layout.ld
