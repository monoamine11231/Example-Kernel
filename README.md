# peepo64

![widePeepoHappy](peepo-emotes/widePeepoHappy.png "test image")

## Dependencies for MacOS
`qemu-system-x86_64`, `objdump` (llvm package or binutils), `mkfs.fat` (dosfstools), `nasm`, `binutils`

## Tips
Use `make` to compile rust code + assembler bootloader + create the image + qemu run. No need for rust `bootimage`.

## Memory used
The map of lower memory (&lt;1MiB) should be complemented with [Memory Map (x86)](https://wiki.osdev.org/Memory_Map_(x86)).
<br>
| **Physical Address** | **Size**                    | **Description**                                         |
|----------------------|-----------------------------|---------------------------------------------------------|
| 0x500                | 0x200                       | VBR sector (Second bootloader)                          |
| 0x7bff               | NaN                         | Stack top                                               |
| 0x7c00               | 0x200                       | MBR sector (First bootloader)                           |
| 0x7e00               | 0x04                        | E820 memory map entries number                          |
| 0x7e04               | NaN                         | E820 memory map of upper memory (>= 1MiB)               |
|                      |                             |                                                         |
| 0x70000              | 0x08 (1 entry) (4K align)   | PML4 (Temporary for switching into 64-bit mode)         |
| 0x71000              | 0x08 (1 entry) (4K align)   | PDPT (Temporary for switching into 64-bit mode)         |
| 0x72000              | 0x18 (3 entries) (4K align) | PDT (Temporary for switching into 64-bit mode)          |
| 0x73000              | 0x1000 (Whole table used)   | PT #1 (Temporary for switching into 64-bit mode)        |
| 0x74000              | 0x1000 (Whole table used)   | PT #2 (Temporary for switching into 64-bit mode)        |
| 0x75000              | 0x1000 (Whole table used)   | PT #3 (Temporary for switching into 64-bit mode)        |
|                      |                             |                                                         |
| 0x100000             | 0x600                       | First three sectors (MBR + VBR + extra) (Yes, repeated) |
| 0x100600             | NaN                         | Kernel                                                  |

