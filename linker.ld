ENTRY(_start);

MEMORY
{
    RAM : ORIGIN = 0x8000_0000, LENGTH = 16M
}

SECTIONS
{
    .text : {
        *(.text._start)
            *(.text*)
            } > RAM

        .rodata : { *(.rodata*)  } > RAM
        .bss : { *(.bss*)  } > RAM
        .data : { *(.data*)  } > RAM
    }

