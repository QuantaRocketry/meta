MEMORY
{
  /* SoftDevice S140 v7 takes the first 156KB */
  FLASH : ORIGIN = 0x27000, LENGTH = 0xC6000
  /* SoftDevice RAM overhead (adjust 0x6000 based on your SD config) */
  RAM   : ORIGIN = 0x20006000, LENGTH = 0x3A000
}

SECTIONS
{
  /* This stores the logging strings in the ELF file on your PC */
  .defmt 0 (INFO) :
  {
    KEEP(*(.defmt .defmt.*))
  }
}
/* 
This is crucial for RTIC/Cortex-M projects to merge the sections correctly
INSERT_AFTER .rodata; */