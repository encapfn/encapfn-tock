TBF_HEADER_SIZE = 0x90;

FLASH_START  = 0x00080000;
FLASH_LENGTH = 0x00010000;
RAM_START    = 0x40080000;
RAM_LENGTH   = 0x00010000;

/* Stack size, including values stacked by the Rust runtime */
STACK_SIZE   = 32 * 1024;

INCLUDE ../../encapfn-tock/encapfn_c_rt/encapfn_layout.ld
