TBF_HEADER_SIZE = 0x60;

FLASH_START  = 0x20030000;
FLASH_LENGTH = 0x00060000;
RAM_START    = 0x10010000;
RAM_LENGTH   = 0x00010000;

/* Stack size, including values stacked by the Rust runtime */
STACK_SIZE   = 8 * 1024;

INCLUDE $EF_TOCK_BASEDIR/encapfn_c_rt/encapfn_layout.ld
