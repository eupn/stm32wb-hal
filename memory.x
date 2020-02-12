MEMORY
{
    FLASH (rx)                 : ORIGIN = 0x08000000, LENGTH = 512K
    RAM1 (xrw)                 : ORIGIN = 0x20000004, LENGTH = 191K
    RAM_SHARED (xrw)           : ORIGIN = 0x20030000, LENGTH = 10K
}

/* Place stack at the end of SRAM1 */
_stack_start = 0x20030000;

SECTIONS {
    MAPPING_TABLE (NOLOAD) : { *(MAPPING_TABLE) } >RAM_SHARED
    MB_MEM1 (NOLOAD)       : { *(MB_MEM1) } >RAM_SHARED
    MB_MEM2                : { *(MB_MEM2) } >RAM_SHARED
}
