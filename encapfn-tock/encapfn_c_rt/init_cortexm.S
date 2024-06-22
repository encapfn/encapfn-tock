/* encapfn_rtheader is defined by the service linker script (encapfn_layout.ld).
 * It has the following layout:
 *
 *     Field                       | Offset
 *     ------------------------------------
 *     Start of .data in flash     |      0
 *     Size of .data               |      4
 *     Start of .data in RAM       |      8
 *     Size of .bss                |     12
 *     Start of .bss in RAM        |     16
 */

/* encapfn_init is executed by the kernel to have it initialize its memory.
 * The kernel passes the following arguments:
 *
 *     r0  Pointer to the encapfn_rtheader.
 *
 * After initialization, we are expected to return to the kernel by invoking
 * `ret` (`bx lr`), where `r0` is used to indicate the initialization
 * status. `r0` = 0 is interpreted as successful initialization, all other
 * values indicate an error.
 */

/* .section .encapfn_init_text */

.syntax unified
.cpu cortex-m4
.thumb
.globl encapfn_init
encapfn_init:
	/* Make sure all of the provided parameters are word-aligned: */
	ldr r1, [r0, #0]	/* src = encapfn_rtheader.data_flash_start */
	tst r1, #3
	bne .Linit_error

	ldr r2, [r0, #4]	/* remaining = encapfn_rtheader.data_size */
	tst r2, #3
	bne .Linit_error

	ldr r3, [r0, #8]	/* dest = encapfn_rtheader.data_ram_start */
	tst r3, #3
	bne .Linit_error

	/* Copy data */
.Ldata_loop:
	cmp r2, #0
	beq .Lzero_bss

	ldr r4, [r1, #0]
	str r4, [r3, #0]
	add r1, r1, #4
	add r2, r2, #-4
	add r3, r3, #4
	b   .Ldata_loop

	/* Zero BSS */
.Lzero_bss:
	ldr r1, [r0, #12]	/* remaining = encapfn_rtheader.bss_size */
	ldr r2, [r0, #16]	/* dest = encapfn_rtheader.bss_start */
	mov r3, #0

.Lzero_bss_loop:
	cmp r1, #0
	beq .Linit_done
	str r3, [r2, #0]
	add r1, r1, #-4
	add r2, r2, #4
	b   .Lzero_bss_loop

.Linit_done:
	mov r0, #0
	bx lr

.Linit_error:
	mov r0, #1
	bx lr
