; SYNTAX:   NASM
; SIZE:     353 bytes (of 448) 



[org 0x55a]
[bits 32]
main32_cont:
    call enable_long_mode


[bits 32]
enable_long_mode:
    call set_paging
    
    lgdt [GDT64_Descriptor]         ; Load the 64 bit GDT
    jmp KERNEL64_CODE_SEG:main64    ; Jump to long mode 

[bits 64]
main64:
    call clear_screen
    
    mov rax, 'S@T@A@G@'             ; Stage 2 bootloader message on screen
    mov [0xb8000], rax
    mov eax, 'E@ @'
    mov [0xb8008], eax
    mov ax, '2@'
    mov [0xb800C], ax 

    mov rsp, 0x400000                 ; Set stack at 0x9000  
    mov rbp, rsp

    call clear_screen


    jmp KERNEL_LOAD_BASE+0x600 

clear_screen:
    ; ===================================================================================
    ; Clears the screen when in VGA text mode 
    ; ===================================================================================

    push rcx
    mov rcx, 0xb8000

    clear:
        ; Black bg white space fill
        mov al, ' '
        mov ah, 0x00
        mov [rcx], ax

        inc rcx
        inc rcx
        cmp rcx, 0xb8000+80*25*2
        jne clear
    
    pop rcx

    ret

%include "bootloader/asm_include/defines.s"
%include "bootloader/asm_include/gdt64.s"
%include "bootloader/asm_include/paging.s"