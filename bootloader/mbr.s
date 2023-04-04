; SYNTAX:   NASM
; SIZE:     254 bytes (of 440) 


; Start off in 16-bit mode
[bits 16]
; Calculate memory offsets from here since BIOS loads 512-byte sector there
[org 0x7c00]


SECOND_BOOTLOADER_LOCATION equ 0x500
main:
    xor ax, ax
    mov ds, ax
    mov ss, ax

    ; BIOS sets the boot drive number in dl register so save it for later
    mov [BOOT_DRIVE], dl

    ; Setup stack at nearly half of available RAM in real mode
    mov bp, 0x9000  
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

    mov bx, KERNEL32_DATA_SEG   ; 32 bit address space
    mov ds, bx 
    mov es, bx
    mov fs, bx 
    mov gs, bx
    mov ss, bx

    and al, 0xFE        ; unreal mode (switching back to real mode)
    mov cr0, eax
    
    xor ax, ax
    mov ds, ax 
    mov es, ax
    mov fs, ax 
    mov gs, ax
    mov ss, ax

    ; Get the number of reserved sectors (16 bit value)
    mov bx, 0x50e
    mov ax, WORD [bx] 

    ; Add one sector (MBR) one since we are reading the whole disk to prevent working
    ; working with offsets of sectors and heads
    add ax, 1
    mov [SECTORS_TO_READ], ax

    mov cx, 0x0
    sti             ; For BIOS interrupts

    ; Read everything by chunks of 1 track = 63 sectors
    read_kernel:
        push cx

        ; Destination where tmp read sectors are placed and then moved to higher address
        ; 0x1000:0x00 = 0x10000
        mov ax, WORD 0x1000
        mov es, ax
        mov bx, WORD 0x0000

        mov ah, 02h
        mov al, 63 ; Read 63 sectors at a time (1 head)
        mov ch, 0          
        mov cl, 0x1
        mov dh, [HEAD]
        mov dl, [BOOT_DRIVE]
        int 13h


        xor ax, ax
        mov ds, ax
        mov es, ax

        mov esi, DWORD 0x10000    ; Source from where a track was loaded
        mov edi, [DEST_PTR]       ; Destination at 1MiB
        mov ecx, 16128            ; The number of ops to do, to copy fully to 1MiB area

        a32 rep movsw             ; a32 to make it access the full edi and esi instead of di and si

        inc BYTE [HEAD]
        add DWORD [DEST_PTR], 32256
        
        pop cx
        mov ax, [SECTORS_TO_READ]
        add cx, 63                ; Register to cx register that we just read 63 sectors
        cmp cx, ax             

        jl read_kernel            ; Read everything by chunks of 1 track = 63 sectors


    ; Set protected mode again
    cli
    lgdt [GDT32_Descriptor]

    mov eax, cr0 
    or al, 1                   ; set PE (Protection Enable) bit in CR0 (Control Register 0)
    mov cr0, eax

    ; Set back the right segment and zero the segment registers
    mov eax, KERNEL32_DATA_SEG
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    ; Jump to second stage bootloader
    jmp KERNEL32_CODE_SEG:SECOND_BOOTLOADER_LOCATION

; Freeze if an error occured
disk_error:
    jmp $

; Include the description of gdt (relative to root path)
%include "bootloader/asm_include/gdt32.s"

; The variable to store the boot drive number
BOOT_DRIVE db 0x0
SECTORS_TO_READ dw 0x0
HEAD db 0x0
DEST_PTR dd 0x100000

; Fill the rest of bootsector code with 0x00
times 440 - ($-$$) db 0

; Unique Disk ID
dd 0x2c26cded
; Reserved
dw 0x0000

; First (and only) partition 
dd 0x00020000
dd 0x0de0000c
dd 0x00000001
dd 0x0001869f

; Second empty partition
dd 0x00000000
dd 0x00000000
dd 0x00000000
dd 0x00000000

; Third empty partition
dd 0x00000000
dd 0x00000000
dd 0x00000000
dd 0x00000000

; Forth empty partition
dd 0x00000000
dd 0x00000000
dd 0x00000000
dd 0x00000000

; Boot signature
dw 0xaa55