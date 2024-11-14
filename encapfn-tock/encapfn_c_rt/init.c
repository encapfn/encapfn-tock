#include <stddef.h>

/* efrt_header is defined by the service linker script (encapfn_layout.ld).
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
struct efrt_header {
  void *data_flash_start;
  size_t data_size;
  void *data_ram_start;
  size_t bss_size;
  void *bss_start;
};

/* _ef_init is executed (sandboxed) by the kernel to have it initialize the
 * loaded binary's memory. The kernel passes the following arguments:
 *
 *     a0  Pointer to the encapfn_rtheader.
 *
 * After initialization, we are expected to return to the kernel by invoking
 * `ret` (`jalr x0, x1, 0`), where `a0` is used to indicate the initialization
 * status. `a0` = 0 is interpreted as successful initialization, all other
 * values indicate an error. `a1` must indicate the new stack pointer top, to
 * be used by the Encapsulated Functions runtime for stacking values, or
 * NULL if the top of the assigned memory region shall be used.
 */
__attribute__ ((section(".ef_init"), used))
__attribute__ ((weak))
__attribute__ ((naked))
__attribute__ ((noreturn))
void _ef_init(struct efrt_header *hdr __attribute__((unused))) {
#if defined(__riscv)
  __asm__ volatile (
    // Make sure all of the provided parameters are word-aligned, but only if
    // there is actually any data to copy (otherwise the linker will place them
    // wherever)
    "  lw   t1, 1*4(a0)             \n" // remaining = encapfn_rtheader.data_size
    "  beqz t1, .Lzero_bss          \n" // short circuit if we don't have data to copy
    "  andi t3, t1, 3               \n"
    "  bnez t3, .Linit_error        \n"

    "  lw   t0, 0*4(a0)             \n" // src = encapfn_rtheader.data_flash
    "  andi t3, t0, 3               \n"
    "  bnez t3, .Linit_error        \n"

    "  lw   t2, 2*4(a0)             \n" // dest = encapfn_rtheader.data_ram
    "  andi t3, t2, 3               \n"
    "  bnez t3, .Linit_error        \n"

    // Copy data
    "  beqz t1, .Lzero_bss          \n" // Jump to zero_bss if remaining is zero

    ".Ldata_loop_body:              \n"
    "  lw   t3, 0(t0)               \n" // t3 = *src
    "  sw   t3, 0(t2)               \n" // *dest = t3
    "  addi t0, t0, 4               \n" // src += 4
    "  addi t1, t1, -4              \n" // remaining -= 4
    "  addi t2, t2, 4               \n" // dest += 4
    "  bnez t1, .Ldata_loop_body    \n" // Loop if there's still data remaining

    ".Lzero_bss:                    \n"
    "  lw   t0, 3*4(a0)             \n" // remaining = rt_encapfn_rtheader.bss_size
    "  lw   t1, 4*4(a0)             \n" // dest = rt_encapfn_rtheader.bss_start
    "  add  t2, t1, t0              \n" // end = dest + remaining

    // Zero BSS
    "  beq  t1, t2, .Linit_done     \n" // Jump to init_done if no data to copy

    ".Lbss_loop_body:               \n"
    "  sb   zero, 0(t1)             \n" // *dest = zero
    "  addi t1, a0, 1               \n" // dest += 1
    "  beq  t1, t2, .Lbss_loop_body \n" // Iterate again if dest != end

    ".Linit_done:                   \n"
    "  li   a0, 0                   \n" // Report no error
    "  la   sp, _stack_top          \n" // Tell the runtime the location of _stack_top
    "  ret                          \n"

    ".Linit_error:                  \n"
    "  li   a0, 1                   \n"
    "  ret                          \n"
  );
#elif defined(__thumb__)
  __asm__ volatile (
    // Make sure all of the provided parameters are word-aligned:
    "  ldr r1, [r0, #0]              \n" // src = encapfn_rtheader.data_flash_start
    "  tst r1, #3		     \n"
    "  bne .Linit_error		     \n"

    "  ldr r2, [r0, #4]	             \n" // remaining = encapfn_rtheader.data_size
    "  tst r2, #3		     \n"
    "  bne .Linit_error		     \n"

    "  ldr r3, [r0, #8]	             \n" // dest = encapfn_rtheader.data_ram_start
    "  tst r3, #3		     \n"
    "  bne .Linit_error		     \n"

    // Copy data
    ".Ldata_loop: 		     \n"
    "  cmp r2, #0		     \n"
    "  beq .Lzero_bss		     \n"

    "  ldr r4, [r1, #0]		     \n"
    "  str r4, [r3, #0]		     \n"
    "  add r1, r1, #4		     \n"
    "  add r2, r2, #-4		     \n"
    "  add r3, r3, #4		     \n"
    "  b   .Ldata_loop		     \n"

    // Zero BSS
    ".Lzero_bss:		     \n"
    "  ldr r1, [r0, #12]	     \n" // remaining = encapfn_rtheader.bss_size
    "  ldr r2, [r0, #16]	     \n" // dest = encapfn_rtheader.bss_start
    "  mov r3, #0		     \n"

    ".Lzero_bss_loop: 		     \n"
    "  cmp r1, #0		     \n"
    "  beq .Linit_done		     \n"
    "  str r3, [r2, #0]		     \n"
    "  add r1, r1, #-4		     \n"
    "  add r2, r2, #4		     \n"
    "  b   .Lzero_bss_loop	     \n"

    ".Linit_done:		     \n"
    "  mov r0, #0		     \n"
    "  bx lr			     \n"

    ".Linit_error:		     \n"
    "  mov r0, #1		     \n"
    "  bx lr                         \n"
  );
#else
#error Missing _ef_init initialization routine for current arch.
#endif
}
