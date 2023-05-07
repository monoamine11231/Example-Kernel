; SYNTAX:   NASM
; SIZE:     425 bytes (of 440) 



[org 0x7c00]
[bits 16]
main16:
    
    mov ah, 0x00                    
    mov al, 0x12
    int 0x10                        ;Telling Bios to setup VGA mode 12
    
    mov bp, FIRST_STACK             ; Setup stack at 0x7BFF
    mov sp, bp
    
    mov [BOOT_DRIVE], dl            ; Save the BIOS boot drive number from dl register


    call clear_segments

    call load_second_bootloader
 
    call get_e820_memory_map        ; Get a mapping for upper memory in e820 layout

    call enable_unreal_mode
    call load_kernel                ; Needs unreal mode
    call enable_protected_mode      ; Jumps to VBR bootloader
    
    ; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    ; Look into main32
    ; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!


[bits 32]
main32:
    mov eax, KERNEL32_DATA_SEG      ; Load the segment descriptor to segment registers
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ss, ax

    jmp KERNEL32_CODE_SEG:VBR_BOOTLOADER

    ; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    ; Look into bootloader/vbr.s
    ; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!



[bits 16]
load_second_bootloader:
    ; ===================================================================================
    ; Load the VBR sector (next 512 bytes that contains the second stage bootloader)
    ; ===================================================================================

    
    mov ah, 0x02                    ; Read mode for BIOS interupt
    mov al, 1                       ; Read only one sector
    mov cl, 0x02                    ; Start from sector 2 (sectors are indexed from 1)
    mov ch, 0x00                    ; Cyllinder 0
    mov dh, 0x00                    ; Head 0
    mov bx, VBR_BOOTLOADER          ; Address of where the VBR section has been loaded

    int 0x13                        ; BIOS interupt to read disk with the VBR

    
    jc error                        ; If disk read error has occured: freeze
    cmp al, 1
    jne error

    ret       

[bits 16]
get_e820_memory_map:
    ; ===================================================================================
    ; Loads a memory map in e820 descriptors at base address of 0x7e00
    ; ===================================================================================

    mov eax, 0xe820                 ; Interrupt code for BIOS
    xor ebx, ebx                    ; First entry, thus ebx must be 0
    mov ecx, 24                     ; We ask for one entry (24 bytes)
    mov di, E820_MAP_BASE + 4       ; +4 to prevent stuck on int
    mov edx, DWORD 0x0534D4150      ; Place the dword "SMAP" which is a signature

    mov [es:di+20], DWORD 0x01      ; Place 0x01 to the second uint64_t 4 ACPI compat
    xor bp, bp                      ; Keep entry count in bp
    int 0x15                        ; Make interrupt 

    jc error                        ; Not supported function
    mov edx, DWORD 0x0534D4150
    cmp eax, edx                    ; On success, eax must be set to "SMAP"
    jne error

    test ebx, ebx                   ; Check if it is only 1 entry at total (bad)
    je error

    jmp entry_check_e820

    continue_e820:
        mov eax, 0xe820
        mov ecx, 24                   
        mov [es:di+20], DWORD 0x01;
        int 0x15

        jc end_e820
        mov edx, 0x0534D4150        ; Some BIOSes trash this register
    
    entry_check_e820:
        jcxz entry_skip_e820        ; Skip zero length entry
        cmp cl, 20                  ; 24 byte ACPI 3.x responce?
        jbe entry_notext_e820
        test byte [es:di + 20], 0x1 ; If so: is the ignore data clear or not?
        je entry_skip_e820

    entry_notext_e820:
        mov ecx, [es:di + 8]        ; Get lower 32 bit uint of memory length
        or ecx, [es:di + 12]        ; or it with upper uint32_t to test for 0
        jz entry_skip_e820          ; If length of uint64_t is 0, skip the entry
    
        inc bp                      ; Increment the entry counter
        add di, 24                  ; Increment the entry destination offset

    entry_skip_e820:
        test ebx, ebx
        jne continue_e820

    end_e820:
        clc                         ; Clear the carry flag
        mov [E820_MAP_BASE], bp     ; Set the entry counter 32-bit int before the entries 
        mov [E820_MAP_BASE+2], WORD 0x00
        ret

[bits 16]
enable_unreal_mode:
    ; ===================================================================================
    ; Enable unreal mode by protected mode -> caching the segment descriptor -> real mode
    ; ===================================================================================


    cli                             ; Disable interrupts
    lgdt [GDT32_Descriptor]         ; Load 32-bit GDT to GDT register
    
    mov eax, cr0 
    or al, 1                        ; Set PE (Protection Enable) bit in CR0
    mov cr0, eax

    mov bx, KERNEL32_DATA_SEG       ; 32-bit address space
    mov ds, bx 
    mov es, bx
    mov fs, bx 
    mov gs, bx
    mov ss, bx

    and al, 0xFE                    ; Set unreal mode by toggling off the PE bit
    mov cr0, eax
    
    call clear_segments
    sti                             ; Enable interrupts (for BIOS interrupts)

    ret

[bits 16]
load_kernel:
    ; ===================================================================================
    ; Load the kernel with BIOS interrupt and place it above 1MiB boundary (unreal mode)
    ; ===================================================================================

    
    mov bx, 0x50e                   ; Get # of reserved sectors (# kernel sectors + 2)
    mov ax, WORD [bx] 

    add ax, 1                       ; +1 for the MBR sector since we read the whole disk
    mov [SECTORS_TO_READ], ax

    mov cx, 0x0                     ; Read segments-counter
    
    read_kernel:
        push cx
        
        mov ax, WORD SOURCE_PTR>>4  ; Slice the address into base and segment
        mov es, ax
        mov bx, WORD SOURCE_PTR&0xFFFF

        mov ah, 02h                 ; Read interrupt number
        mov al, 63                  ; Read 63 sectors at a time (1 head)
        mov ch, 0                   ; Cylinder 0
        mov cl, 0x1                 ; Sector 1
        mov dh, [HEAD]              
        mov dl, [BOOT_DRIVE]        ; BIOS boot drive to read from
        
        int 0x13

        
        jc error                    ; If disk read error has occured: freeze
        cmp al, 63
        jne error


        xor ax, ax
        mov ds, ax
        mov es, ax

        mov esi, DWORD SOURCE_PTR   ; Source from where a track was loaded
        mov edi, [DEST_PTR]         ; Destination at 1MiB
        mov ecx, 16128              ; The number of ops to do, to copy fully to 1MiB area

        a32 rep movsw               ; a32 so that movsd uses esi & edi instead of si & di

        inc BYTE [HEAD]             ; Increment the number of heads read and the dest ptr
        add DWORD [DEST_PTR], 32256
        
        pop cx
        mov ax, [SECTORS_TO_READ]
        add cx, 63                  ; Register to cx register that we read 63 sectors
        cmp cx, ax             

        jl read_kernel              ; Read until the kernel + rest of head was read

    ret

[bits 16]
enable_protected_mode:
    cli
    lgdt [GDT32_Descriptor]

    mov eax, cr0 
    or al, 1                        ; set PE (Protection Enable) bit in CR0
    mov cr0, eax

    
    jmp KERNEL32_CODE_SEG:main32    ; Jump to 32-bit method to activate 32-bit funcs


[bits 16]
error:
    jmp $                           ; Freeze if an error occured  

[bits 16]
clear_segments:
    push ax

    xor ax, ax                      ; Zero out the registers
    mov ds, ax 
    mov es, ax
    mov fs, ax 
    mov gs, ax
    mov ss, ax

    pop ax
    ret


; Macros
%include "bootloader/asm_include/defines.s"
; Include the description of gdt (relative to root path)
%include "bootloader/asm_include/gdt32.s"


BOOT_DRIVE db 0x0                   ; The variable to store the BIOS boot drive #
SECTORS_TO_READ dw 0x0              ; Constant of the # of sectors that kernel takes + 1
HEAD db 0x0                         ; # of current head when kernel is being read
DEST_PTR dd KERNEL_LOAD_BASE        ; Dest ptr to where the read sectors are copied to


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