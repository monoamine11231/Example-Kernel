FIRST_STACK equ 0x7bff
VBR_BOOTLOADER equ 0x500            ; Position of the second bootloader in memory

E820_MAP_BASE equ 0x7e00            ; Position of the e820 mapping for upper memory

KERNEL_LOAD_BASE equ 0x100000       ; Base where the kernel is being loaded to
SOURCE_PTR equ 0x70000              ; Source ptr where one head is read and moved from