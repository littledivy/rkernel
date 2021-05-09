use spin::Mutex;
use x86_64::instructions::port::Port;

pub struct Pci {
    address_port: Port<u32>,
    data_port: Port<u32>,
}

static PCI: Mutex<Pci> = Mutex::new(unsafe { Pci::default() });

impl Pci {
    pub unsafe fn config_read(&mut self, bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
        self.address_port.write(
            0x80000000
                | (bus as u32) << 16
                | (slot as u32) << 11
                | (function as u32) << 8
                | (offset & 0b1111_1100) as u32,
        );

        self.data_port.read()
    }

    pub const unsafe fn default() -> Self {
        Self {
            address_port: Port::new(0xCF8),
            data_port: Port::new(0xCFC),
        }
    }
}

pub struct PciDevice {
    vendor_id: u32,
    device_id: u32,
    subclass: u32,
    class: u32,
    header_type: u32,
}

impl PciDevice {
    pub unsafe fn new(bus: u8, slot: u8, function: u8) -> Self {
        let mut pci = PCI.lock();
        Self {
            vendor_id: pci.config_read(bus, slot, function, 0x0),
            device_id: pci.config_read(bus, slot, function, 0x2),
            subclass: pci.config_read(bus, slot, function, 0xa),
            class: pci.config_read(bus, slot, function, 0xb),
            header_type: pci.config_read(bus, slot, function, 0xe),
        }
    }
}

fn device_class() {}
pub fn init() {
    for slot in 0..31 {
        unsafe {
            let device = PciDevice::new(0, slot, 0);
            if device.vendor_id != 0xFFFFFFFF {
                for function in 0..7 {
                    let device = PciDevice::new(0, slot, function);
                    if device.vendor_id != 0xFFFFFFFF {
                        let info = alloc::format!(
                            "PCI (0:{}:{}) => {:04x} {:04x}\n",
                            slot,
                            function,
                            device.vendor_id,
                            device.device_id
                        );
                        crate::log!(info.as_bytes());
                    }
                }
            }
        }
    }
}
