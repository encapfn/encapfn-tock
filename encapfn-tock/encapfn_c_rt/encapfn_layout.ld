
MEMORY {
  FLASH (X) : ORIGIN = FLASH_START + TBF_HEADER_SIZE, LENGTH = FLASH_LENGTH - TBF_HEADER_SIZE
  RAM   (W) : ORIGIN = RAM_START                    , LENGTH = RAM_LENGTH
}

/* GNU LD looks for `start` as an entry point by default, while LLVM's LLD looks
 * for `_start`. To be compatible with both, we manually specify an entry point.
 */
ENTRY(start)

SECTIONS {
    /* The FLASH memory section is placed at a TBF_HEADER_SIZE offset, to give
     * elf2tab room to prepend the TBF headers. Communicate this reservation to
     * elf2tab, such that it fills up the space after the TBF headers (if any)
     * as part of the protected region trailer:
     */
    tbf_protected_region_size = TBF_HEADER_SIZE;

    /* Sections located in FLASH.
     */

    _flash_origin = ORIGIN(FLASH);

    /* Encapsulated Functions header.
     */
    . = ALIGN(4);
    .text : {
        /* We define start here, in the first section to be included
         * in FLASH. This makes the linker and elf2tab happy, but we
         * don't actually use this symbol. */
        start = .;

        LONG(0x454E4350); /* MAGIC: ENCP */
        LONG(encapfn_rthdr - ORIGIN(FLASH));
        LONG(encapfn_init - ORIGIN(FLASH));
        LONG(encapfn_fntab - ORIGIN(FLASH));
	LONG(encapfn_fntab_length - ORIGIN(FLASH));

        KEEP(*(.encapfn_hdr));

        . = ALIGN(4);

        encapfn_rthdr = .;

        LONG(LOADADDR(.data));
        LONG(SIZEOF(.data));
        LONG(ADDR(.data));
        LONG(SIZEOF(.bss));
        LONG(ADDR(.bss));

        *(.text.*)
    } > FLASH

    /* Read-only data section. Contains strings and other global constants. */
    .rodata ALIGN(4) : {
        *(.rodata.*)
    } > FLASH

    /* Sections located in RAM at runtime.
     */
    _sram_origin = ORIGIN(RAM);

    /* Need to reserve room for the stack in the linker file. This will cause
     * stack to grow downward towards the end of the available memory section,
     * and thus fault instead of silently overwriting data. */
    .stack : {
        . = ALIGN(16);
	_stack_bottom = .;
	. = _stack_bottom + STACK_SIZE;
	. = ALIGN(16);
	_stack_top = .;
    }

    /* Read-write data section. This is deployed as part of FLASH but is copied
     * into RAM at runtime by the loader (i.e. Tock).
     */
    .data ALIGN(4) : {
        /* .sdata is the RISC-V small data section */
        *(.sdata .data)
    } > RAM AT > FLASH


    /* BSS section. These are zero-initialized static variables. */
    .bss ALIGN(4) (NOLOAD) : {
        /* .sbss is the RISC-V small data section */
        *(.sbss .bss.*)
    } > RAM AT > FLASH

    /* Sections we do not need. */
    /DISCARD/ :
    {
      *(.ARM.exidx .eh_frame)
    }
}
