use multiboot2::BootInformation;

pub fn info(boot_info: &BootInformation, screen: &mut crate::graphics::Screen) {
    if let Some(mmap) = boot_info.memory_map_tag() {
        let mut memory_size = 0;
        for region in mmap.memory_areas() {
            let start_addr = region.start_address();
            let end_addr = region.end_address();
            memory_size += end_addr - start_addr;

            let mut buffer = ryu::Buffer::new();
            let start_addr = buffer.format(start_addr as f64);
            crate::raw_write!(screen, b"MEM ");
            crate::raw_write!(screen, start_addr.as_bytes());
            crate::raw_write!(screen, b" - ");
            let end_addr = buffer.format(end_addr as f64);
            crate::raw_write!(screen, end_addr.as_bytes());
            crate::raw_write!(screen, b"\n");
        }
    }
}
