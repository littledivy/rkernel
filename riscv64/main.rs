#![no_std]
#![no_main]


#[no_mangle]
extern "C" fn kmain() {}

#[no_mangle]
extern "C" fn _rupt() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
 loop {}
}
