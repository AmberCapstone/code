MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH =  224K /* BANK_1 */
    NVM   : ORIGIN = 0x08038000, LENGTH =   32K
    RAM   : ORIGIN = 0x20000000, LENGTH =   40K /* SRAM */
}

__flash_start = ORIGIN(FLASH);
__flash_end = ORIGIN(FLASH) + LENGTH(FLASH);
__nvm_start = ORIGIN(NVM);
__nvm_end = ORIGIN(NVM) + LENGTH(NVM);
