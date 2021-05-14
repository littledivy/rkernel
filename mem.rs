use multiboot2::BootInformation;
use  multiboot2::MemoryMapTag;
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

 use x86_64::structures::paging::PageTableFlags as Flags;
pub const PT_PADDR: PhysAddr = unsafe { PhysAddr::new_unsafe(0x1000_0000) };
pub const PT_VADDR: VirtAddr = unsafe { VirtAddr::new_unsafe(1) };

#[inline(always)]
#[cold]
pub unsafe fn fast_set64(dst: *mut u64, src: u64, len: usize) {
    asm!("cld; rep stosq",
         in("rdi") dst as usize,
         in("rax") src,
         in("rcx") len,
         lateout("rdi") _,
         lateout("rcx") _,
    );
}

pub fn info(boot_info: BootInformation) {
    let phys_mem_offset = PT_VADDR;
    let mut mapper = unsafe { init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map_tag().unwrap()) };


    if let Some(fb_info) = boot_info.vbe_info_tag() {
      // let fb = fb_info.mode_info.framebuffer_base_ptr as u64;
      // let fb = boot_info.framebuffer_tag().unwrap().address;
      let fb = 0xFD000000;
      unsafe {

            let start_page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(fb));
            let end_page = Page::containing_address(VirtAddr::new(fb + 800 * 600));
            for page in Page::range_inclusive(start_page, end_page) {
                let frame = PhysFrame::containing_address(PhysAddr::new(page.start_address().as_u64()));
                let flags = Flags::PRESENT | Flags::NO_EXECUTE | Flags::WRITABLE | Flags::HUGE_PAGE;
                let result = mapper.map_to(page, frame, flags, &mut frame_allocator).unwrap();
                result.flush();
            }

      }
      unsafe { 
            let page_ptr: *mut u32 = fb as *mut u32;
            //fast_set64(page_ptr, 1, 100);
            for i in 0..80*60 {
              *page_ptr.offset(i) = 0x00ff0000;
            }
      }
    }
    if let Some(mmap) = boot_info.memory_map_tag() {
        let mut memory_size = 0;
        for region in mmap.memory_areas() {
            let start_addr = region.start_address();
            let end_addr = region.end_address();
            memory_size += end_addr - start_addr;
            let mut buffer = ryu::Buffer::new();
            crate::log!(b"Memory region ");
            let start_addr = buffer.format(start_addr as f64);
            crate::raw_write!(start_addr.as_bytes());
            crate::raw_write!(b" - ");
            let end_addr = buffer.format(end_addr as f64);
            crate::raw_write!(end_addr.as_bytes());
            crate::raw_write!(b"\n");
        }
    }
}

// Frame allocator

pub struct BootInfoFrameAllocator<'a> {
    memory_map: &'a MemoryMapTag,
    next: usize,
}

impl<'a> BootInfoFrameAllocator<'a> {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'a MemoryMapTag) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + 'a {
        // get usable regions from memory map
        let regions = self.memory_map.all_memory_areas();
        let usable_regions = regions.filter(|r| r.typ() == multiboot2::MemoryAreaType::Available);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.start_address()..r.end_address());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator<'_> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

// Paging

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub fn create_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    phy_addr: PhysAddr,
) {
    let frame = PhysFrame::containing_address(phy_addr);
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: this is not safe, we do it only for testing
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}
