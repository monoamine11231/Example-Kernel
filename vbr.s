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

; Stage 2 bootloader message on screen
mov rax, 'S@T@A@G@'
mov [0xb8000], rax
mov eax, 'E@ @'
mov [0xb8008], eax
mov ax, '2@'
mov [0xb800C], ax 


; Set nice stack
mov rsp, 0x9000  
mov rbp, rsp

mov rax, [0x100600]  
; Jump to kernel. Remember that we loaded the whole disk when loading the kernel, so we
; must jump to 4th sector where the kernel is located
jmp 0x100600 

%include "asm_include/gdt64.s"
%include "asm_include/paging.s"