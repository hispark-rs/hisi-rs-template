/*
 * Memory layout for HiSilicon {{chip}} (BS2X family, RV32IMFC, SparkLink/NearLink).
 *
 *   ROM:      0x000000 - 0x080000  (mask ROM; symbols only, not used by a Rust app)
 *   ITCM:     0x080000 - 0x100000  (512K instruction TCM)
 *   L2RAM:    0x100000 - ...        ({% if chip == "bs20" %}128K — BS20{% else %}160K — BS21/BS22{% endif %})
 *   FLASH:    0x10000000            (1M XIP NOR flash)
 *
 * Code in flash (XIP), data/bss/stack in L2RAM (matches the WS63 PROGRAM=flash /
 * SRAM=ram split). Values from the fbb_bs2x SDK (platform_core.h).
 */

MEMORY
{
    /* Mask ROM (secure-libc / printf / timing live here on silicon — region exists
       only for the PROVIDE symbols below). */
    BOOTROM  (rx) : ORIGIN = 0x00000000, LENGTH = 0x8000
    ROM      (rx) : ORIGIN = 0x00008000, LENGTH = 0x78000

    /* Instruction TCM (512K) */
    ITCM     (rwx): ORIGIN = 0x00080000, LENGTH = 0x70000

    /* Data TCM — carved from the top of the TCM window */
    DTCM     (rw) : ORIGIN = 0x000F0000, LENGTH = 0x10000

    /* XIP NOR flash (1M) + the program region within it */
    FLASH    (rx) : ORIGIN = 0x10000000, LENGTH = 0x100000
    PROGRAM  (rx) : ORIGIN = 0x10000000, LENGTH = 0x100000

    /* Main system RAM (L2RAM): {% if chip == "bs20" %}128K (BS20){% else %}160K (BS21/BS22){% endif %} */
    SRAM     (rwx): ORIGIN = 0x00100000, LENGTH = {% if chip == "bs20" %}0x20000{% else %}0x28000{% endif %}

    /* Preserved region (256 bytes at the top of L2RAM for boot state) */
    PRESERVE (rw) : ORIGIN = ORIGIN(SRAM) + LENGTH(SRAM) - 0x100, LENGTH = 0x100
}

/* Memory regions exported as symbols (the set hisi-riscv-rt's layout.ld expects). */
PROVIDE(__rom_start = ORIGIN(ROM));
PROVIDE(__rom_length = LENGTH(ROM));
PROVIDE(__itcm_start = ORIGIN(ITCM));
PROVIDE(__itcm_length = LENGTH(ITCM));
PROVIDE(__dtcm_start = ORIGIN(DTCM));
PROVIDE(__dtcm_length = LENGTH(DTCM));
PROVIDE(__sram_start = ORIGIN(SRAM));
PROVIDE(__sram_length = LENGTH(SRAM));
PROVIDE(__flash_start = ORIGIN(FLASH));
PROVIDE(__flash_length = LENGTH(FLASH));
PROVIDE(__program_start = ORIGIN(PROGRAM));
PROVIDE(__program_length = LENGTH(PROGRAM));

/* Stack sizes (overridable). */
__stack_size     = DEFINED(__stack_size)     ? __stack_size     : 0x2000;
__irq_stack_size = DEFINED(__irq_stack_size) ? __irq_stack_size : 0x800;
__exc_stack_size = DEFINED(__exc_stack_size) ? __exc_stack_size : 0x800;
__nmi_stack_size = DEFINED(__nmi_stack_size) ? __nmi_stack_size : 0x400;

/* riscv-rt required symbols. Stack top = top of L2RAM. */
PROVIDE(_stack_start = ORIGIN(SRAM) + LENGTH(SRAM));
PROVIDE(_max_hart_id = 0);
PROVIDE(_hart_stack_size = 0x2000);

PROVIDE(__sidata = 0);
PROVIDE(__sdata = 0);
PROVIDE(__edata = 0);
PROVIDE(__sbss = 0);
PROVIDE(__ebss = 0);

/* riscv-rt region aliases: text/rodata in flash, data/bss/stack/heap in RAM. */
REGION_ALIAS("REGION_TEXT", PROGRAM);
REGION_ALIAS("REGION_RODATA", PROGRAM);
REGION_ALIAS("REGION_DATA", SRAM);
REGION_ALIAS("REGION_BSS", SRAM);
REGION_ALIAS("REGION_STACK", SRAM);
REGION_ALIAS("REGION_HEAP", SRAM);
