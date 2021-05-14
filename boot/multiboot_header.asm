section .multiboot_header
header_start:
    dd 0xe85250d6                ; magic number (multiboot 2)
    dd 0                         ; architecture 0 (protected mode i386)
    dd header_end - header_start ; header length
    ; checksum
    dd 0x100000000 - (0xe85250d6 + 0 + (header_end - header_start))

   
    dw 5
    dw 0
    dd 20
    dd 800 ; width
    dd 600 ; height
    dd 32 ; bbp
    
    ; required end tag
    dw 0    ; type
    dw 0    ; flags
    dd 0    ; size
header_end:
