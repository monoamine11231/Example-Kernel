; GLOBAL DESCRIPTION TABLE (64 bit bit)

GDT64_Start:
    ; Null segment
    gdt64_null_entry:
        dq 0x00

    ; Kernel mode code segment
    gdt64_kernel_code_entry:
        ; Segment limit
        dw 0xffff
        ; Segment base 0-15 bits
        dw 0x0
        ; Segment base 16-23 bits
        db 0x0
        ; Access Byte
        db 0x9A
        ; Flags 4 bits + segment limit bits 16-19
        db 10101111b
        ; Segment base bits 24-31
        db 0x00

    ; Kernel mode data segment
    gdt64_kernel_data_entry:
        ; Follow https://wiki.osdev.org/GDT_Tutorial
        ; and https://wiki.osdev.org/Global_Descriptor_Table#Segment_Descriptor
        dw 0xffff
        dw 0x0
        db 0x0
        db 0x92
        db 11001111b
        db 0x0
    gdt64_task_segment: equ $ - GDT64_Start
        dd 0x00000068
        dd 0x00CF8900

    ; ; User mode code segment
    ; gdt64_user_code_entry:
    ;     dw 0xffff
    ;     dw 0x0
    ;     db 0x0
    ;     db 0xFA
    ;     db 10101111b
    ;     db 0x0

    ; ; User mode data segment
    ; gdt64_user_data_entry:
    ;     dw 0xffff
    ;     dw 0x0
    ;     db 0x0
    ;     db 0xF2
    ;     db 11001111b
    ;     db 0x0

; Label to calculate GDT size
GDT64_End:

; Metadata about GDT
GDT64_Descriptor:
    ; As in documentation, the size should be subtracted by 1
    dw GDT64_End - GDT64_Start - 1
    ; The adress to the beginning of the GDT followed empty dword
    ; that is used when changing to long mode and where a adress is
    ; a qword
    dd GDT64_Start;

KERNEL64_CODE_SEG equ gdt64_kernel_code_entry - GDT64_Start
KERNEL64_DATA_SEG equ gdt64_kernel_data_entry - GDT64_Start
; USER64_CODE_SEG equ gdt64_user_code_entry - GDT64_Start
; USER64_DATA_SEG equ gdt64_user_data_entry - GDT64_Start