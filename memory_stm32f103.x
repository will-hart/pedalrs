_page_size = 1K;
_eeprom_pages = 2; /* need to sync with config/mod.rs EEPROM_PAGES see also example https://github.com/idubrov/x2-feed/ */

/* Linker script for the STM32F103C8T6 */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K - (_eeprom_pages * _page_size)
  RAM : ORIGIN = 0x20000000, LENGTH = 20K
}