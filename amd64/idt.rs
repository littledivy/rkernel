use crate::pic::Chained;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;

pub static PICS: spin::Mutex<Chained> = spin::Mutex::new(unsafe { Chained::new(32, 32 + 8) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
           unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        // Timer
        idt[32].set_handler_fn(timer_interrupt_handler);
        // PS/2 Keyboard
        idt[32 + 1].set_handler_fn(crate::keyboard::keyboard_interrupt_handler);
        // Sound card
        idt[32 + 5].set_handler_fn(crate::sb16::sound_interrupt_handler);
         // PS/2 Mouse
        idt[32 + 12].set_handler_fn(crate::mouse::mouse_interrupt_handler);
        idt
    };
}

pub fn init() {
    crate::gdt::init();
    IDT.load();
    unsafe { PICS.lock().init() };
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {}
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!();
}
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().end(32);
    }
}
