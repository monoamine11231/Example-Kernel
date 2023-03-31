[bits 64]
[org 0x55a]
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


; Get the number of reserved sectors (16 bit value)
mov rdx, 0x50e
mov ax, WORD [rdx] 

; Subtract 2 since FSInfo and BPB account for 2 sectors
sub ax, 2

; Read the kernel 
mov ebx, 0x00000004     ; CHS value to read (from 4th sector where the kernel is stored)
mov ch, al               ; The number of sectors to read
mov rdi, 0x100000       ; Place kernel at 1MB (the kernel is linked to that position)
call ata_chs_read   


; Stage 3 bootloader message on screen
mov rax, 'S@T@A@G@'
mov [0xb8000], rax 
mov eax, 'E@ @'
mov [0xb8008], eax
mov ax, '3@'
mov [0xb800C], ax
       
mov rax, [0x100000]  
jmp 0x100000   

%include "asm_include/ata.s"