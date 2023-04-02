[bits 32]
make_paging:
    ; Enable PAE for 64bit tables
    mov     eax, cr4
    or      eax, 1 << 5
    mov     cr4, eax   

    ; Page Map Table (1) (Total 4MiB)
    mov DWORD [0x70000], 0x71003   ; Set the address to the PDP table and set R/W+P flags, only the kernel can access it
    mov DWORD [0x70004], 0x0              ; Fill upper 32 bits with 0x0


    ; Page Directory Pointer Table (1) 
    mov DWORD [0x71000], 0x72003   ; Set the address to the PD table and set R/W+P flags, only the kernel can access it
    mov DWORD [0x71004], 0x0              ; Fill upper 32 bits with 0x0


    ; Page Directory Table(s) (2) (2MiB each)
    mov DWORD [0x72000], 0x73003   ; Set the address to the Page table and set R/W+P flags, only the kernel can access it
    mov DWORD [0x72004], 0x0              ; Fill upper 32 bits with 0x0

    mov DWORD [0x72008], 0x74003   ; Set the address to the Page table and set R/W+P flags, only the kernel can access it
    mov DWORD [0x7200C], 0x0              ; Fill upper 32 bits with 0x0


    mov ecx, 0x0
    ; Perform identical mapping on the first 4MiB
    fill_page_tables:
        mov eax, ecx
        mov edx, DWORD 0x08
        mul edx
        push eax
        
        mov eax, ecx
        mov edx, DWORD 0x1000
        mul edx
        or eax, DWORD 0x03
        pop edx

        mov [0x73000+edx+0], eax     ; Identical mapping, flags same as above
        mov DWORD [0x73000+edx+4], 0x00

        inc ecx
        cmp ecx, 0x0400
        jne fill_page_tables 

    mov     eax,0x70000
    mov     cr3,eax                 ; load page-map level-4 base

    mov     ecx,0C0000080h          ; EFER MSR
    rdmsr
    or      eax,1 << 8             ; enable long mode
    wrmsr

    mov     eax,cr0
    or      eax,1 << 31
    mov     cr0,eax                 ; enable paging

    ret