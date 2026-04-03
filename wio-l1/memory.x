/* Memory layout for nRF52840 with SoftDevice S140 v7 and UF2 Bootloader */

MEMORY
{
  /* 
   * FLASH starts at 0x27000 to leave room for the SoftDevice S140 (156KB).
   * It ends at 0xED000 to leave room for the UF2 Bootloader (76KB) at the top of the 1MB flash.
   * Total length = 0xED000 - 0x27000 = 0xC6000 (792KB)
   */
  FLASH : ORIGIN = 0x27000, LENGTH = 0xC6000

  /* 
   * RAM starts at 0x20006000 to leave room for SoftDevice RAM overhead.
   * Total RAM on nRF52840 is 256KB (starts at 0x20000000, ends at 0x20040000).
   * Total length = 0x20040000 - 0x20006000 = 0x3A000 (232KB)
   */
  RAM   : ORIGIN = 0x20006000, LENGTH = 0x3A000
}
