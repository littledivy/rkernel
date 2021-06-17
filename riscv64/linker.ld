OUTPUT_ARCH( riscv )
ENTRY( _start )

SECTIONS {
  . = 0x80000000;

  PROVIDE( __memory_start = . );
  PROVIDE( __kernel_start = . );

  .text : {
    *( .text.proem ) *( .text .text.* )
  }

  .rodata : {
    *( .rodata .rodata.* )
  }
  
  .data : {
    *( .data .data.* )
    /*
       This is important. I do not understand why, nor do I care.
       https://groups.google.com/a/groups.riscv.org/d/msg/sw-dev/60IdaZj27dY/s1eJMlrUAQAJ
    */
    PROVIDE( __global_pointer = . + 0x800 );
  }

  .bss : {
    PROVIDE( __bss_start = . );

    *( .bss .bss.* )

    . += 512K;
    PROVIDE( __stack_top = . );

    PROVIDE( __bss_end = . );
  }
  PROVIDE( __kernel_end = . );

  . = ALIGN( 4K );
  PROVIDE( __heap_start = . );
  . += 256M;
  PROVIDE ( __heap_end = . );
}
