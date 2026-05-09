MEMORY
{
    FLASH                             : ORIGIN = 0x08000000, LENGTH =  48K
    BOOTLOADER_STATE                  : ORIGIN = 0x0800C000, LENGTH =   8K
    ACTIVE                            : ORIGIN = 0x0800E000, LENGTH = 200K
    DFU                               : ORIGIN = 0x08040000, LENGTH = 256K
    RAM   (rwx)                       : ORIGIN = 0x20000000, LENGTH = 128K
}

__bootloader_state_start = ORIGIN(BOOTLOADER_STATE) - ORIGIN(FLASH);
__bootloader_state_end = ORIGIN(BOOTLOADER_STATE) + LENGTH(BOOTLOADER_STATE) - ORIGIN(FLASH);

__bootloader_active_start = ORIGIN(ACTIVE) - ORIGIN(FLASH);
__bootloader_active_end = ORIGIN(ACTIVE) + LENGTH(ACTIVE) - ORIGIN(FLASH);

__bootloader_dfu_start = ORIGIN(DFU) - ORIGIN(DFU);
__bootloader_dfu_end = ORIGIN(DFU) + LENGTH(DFU) - ORIGIN(DFU);