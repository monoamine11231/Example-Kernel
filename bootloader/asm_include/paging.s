[bits 32]
set_paging:
    ; ===================================================================================
    ; Performs identity mapping on the first 6MiB of RAM ad enables paging
    ; ===================================================================================
   
    mov     eax, cr4                 ; Enable PAE for 64bit tables
    or      eax, 1 << 5
    mov     cr4, eax   

                                    ; Page Map Table (1) (Total 6MiB)
    mov DWORD [0x70000], 0x71003    ; Set address to the Page table and set R/W+P flags
    mov DWORD [0x70004], 0x0        ; Fill upper 32 bits with 0x0


                                    ; Page Directory Pointer Table (1) 
    mov DWORD [0x71000], 0x72003    ; Set address to the Page table and set R/W+P flags
    mov DWORD [0x71004], 0x0        ; Fill upper 32 bits with 0x0


                                    ; Page Directory Table(s) (3) (2MiB each)
    mov DWORD [0x72000], 0x73003    ; Set address to the Page table and set R/W+P flags
    mov DWORD [0x72004], 0x0        ; Fill upper 32 bits with 0x0

    mov DWORD [0x72008], 0x74003    ; Set address to the Page table and set R/W+P flags
    mov DWORD [0x7200C], 0x0        ; Fill upper 32 bits with 0x0

    mov DWORD [0x72010], 0x75003
    mov DWORD [0x72004], 0x0


    ; Perform identical mapping on the first 4MiB
    mov ecx, 0x0                    
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

        mov [0x73000+edx+0], eax    ; Identical mapping, flags same as above
        mov DWORD [0x73000+edx+4], 0x00

        inc ecx
        cmp ecx, 0x0600             ; Until all 3 pages are filled
        jne fill_page_tables 

    mov     eax,0x70000
    mov     cr3,eax                 ; Load page-map level-4 base

    mov     ecx,0C0000080h          ; EFER MSR
    rdmsr
    or      eax,1 << 8              ; Enable long mode
    wrmsr

    mov     eax,cr0
    or      eax,1 << 31
    mov     cr0,eax                 ; Enable paging

    ret