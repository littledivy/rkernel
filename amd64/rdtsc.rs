pub fn rdtsc() -> f64 {
    unsafe {
        core::arch::x86_64::_mm_lfence();
        core::arch::x86_64::_rdtsc() as f64
    }
}

pub fn delta_ns(start: f64) -> f64 {
    (rdtsc() - start - 10f64) * 2300000000000f64
}
