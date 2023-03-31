; NASM SYNTAX


; Start off in 16-bit mode
[bits 16]
; Calculate memory offsets from here since BIOS loads 512-byte sector there
[org 0x7c00]


SECOND_BOOTLOADER_LOCATION equ 0x500
main:
    xor ax, ax
    mov ds, ax

    ; BIOS sets the boot drive number in dl register so save it for later
    mov [BOOT_DRIVE], dl

    ; Setup stack at nearly half of available RAM in real mode
    mov bp, 0x4000  
    mov sp, bp

    ; Load the VBR sector (next 512 bytes that contains the second stage bootloader)
    ; ==================================================================================

    ; Read mode for BIOS interupt
    mov ah, 0x02
    ; Read only one sector
    mov al, 1
    ; Start from sector 2 (sectors are indexed from 1)
    mov cl, 0x02
    ; Cyllinder 0, head 0
    mov ch, 0x00
    mov dh, 0x00
    ; Where in the memory should the VBR be loaded (0x500 is the start of usable memory)
    mov bx, SECOND_BOOTLOADER_LOCATION

    ; BIOS interup read second stage bootloader sector
    int 0x13

    ; If error occured: freeze the program as an indication
    jc disk_error
    ; BIOS sets al to the number of sectors actually readed, so compare it to the number of
    ; sectors we wanted to read and freeze if not equal
    cmp al, 1
    jne disk_error

    cli                        ; disable interrupts
    lgdt [GDT32_Descriptor]    ; load GDT register with start address of Global Descriptor Table
    mov eax, cr0 
    or al, 1                   ; set PE (Protection Enable) bit in CR0 (Control Register 0)
    mov cr0, eax

    jmp KERNEL32_CODE_SEG:init_long_mode

    ; Freeze here if somehow returned
    jmp $

; Freeze if an error occured
disk_error:
    ; Video Mode VGA
    mov ah, 00h
    mov al, 02h
    int 0x10

    ; Set active page
    mov al, 05h
    mov al, 00h
    int 0x10

    ; Print char
    mov ah, 0eh
    mov al, 92
    mov bh, 00h
    mov bl, 05h
    int 0x10

    jmp $


[bits 32]
init_long_mode:
    mov eax, KERNEL32_DATA_SEG    ; Set the A-register to the data descriptor.
    mov ds, ax                    ; Set the data segment to the A-register.
    mov es, ax                    ; Set the extra segment to the A-register.
    mov fs, ax                    ; Set the F-segment to the A-register.
    mov gs, ax                    ; Set the G-segment to the A-register.
    mov ss, ax                    ; Set the stack segment to the A-register

    call make_paging

    lgdt [GDT64_Descriptor]

    ; far jump to long mode
    jmp KERNEL64_CODE_SEG:long_mode 
         
[bits 64]
long_mode:
    cli                           ; Clear the interrupt flag.
    mov ax, KERNEL64_DATA_SEG     ; Set the A-register to the data descriptor.
    mov ds, ax                    ; Set the data segment to the A-register.
    mov es, ax                    ; Set the extra segment to the A-register.
    mov fs, ax                    ; Set the F-segment to the A-register.
    mov gs, ax                    ; Set the G-segment to the A-register.
    mov ss, ax                    ; Set the stack segment to the A-register
        
    ; Time for second bo otloader loca tion
    jmp SECOND_BOOTLOADER_LOCATION

; Include the description of gdt
%include "asm_include/gdt.s"

; Paging tables and methods
%include "asm_include/paging.s"

; The variable to store the boot drive number
BOOT_DRIVE db 0

; Null padding
times 510 - ($-$$) db 0
dw 0xaa55
times 4196 db 0