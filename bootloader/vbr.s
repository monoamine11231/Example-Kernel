[org 0x55a]

[bits 32]
call make_paging

; Load the 64 bit GDT
lgdt [GDT64_Descriptor]

; Jump to long mode 
jmp KERNEL64_CODE_SEG:long_mode

[bits 64]
long_mode:
; Clear screen
call clear_screen

    mov rsp, 0x400000                 ; Set stack at 0x9000  
    mov rbp, rsp

    call clear_screen


; Set nice stack
mov rsp, 0x9000  
mov rbp, rsp

call clear_screen

; Jump to kernel. Remember that we loaded the whole disk when loading the kernel, so we
; must jump to 4th sector where the kernel is located
jmp 0x100600 

clear_screen:
    push rcx
    mov rcx, 0xb8000

    clear__:
    ; Black bg white space fill
    mov al, ' '
    mov ah, 0x00
    mov [rcx], ax

    inc rcx
    inc rcx
    cmp rcx, 0xb8000+80*25*2
    jne clear__
    
    pop rcx
    ret

%include "bootloader/asm_include/gdt64.s"
%include "bootloader/asm_include/paging.s"