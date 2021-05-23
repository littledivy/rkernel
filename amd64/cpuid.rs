use alloc::string::String;
use core::arch::x86_64::__cpuid_count;

pub fn init() {
    let mut brand = String::new();

    unsafe {
        for leaf in 0x80000002..=0x80000004 {
            let cpuid = __cpuid_count(leaf, 0);
            brand.push_str(&String::from_utf8_lossy(&cpuid.eax.to_le_bytes()));
            brand.push_str(&String::from_utf8_lossy(&cpuid.ebx.to_le_bytes()));
            brand.push_str(&String::from_utf8_lossy(&cpuid.ecx.to_le_bytes()));
            brand.push_str(&String::from_utf8_lossy(&cpuid.edx.to_le_bytes()));
        }
    }

    crate::log!(brand.as_bytes());
    crate::raw_write!(b"\n");
}
