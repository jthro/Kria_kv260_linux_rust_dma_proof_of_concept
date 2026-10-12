use std::os::unix::fs::OpenOptionsExt;
use std::{fs::OpenOptions, time};

use memmap::MmapOptions;

const AXI_GPIO_BASE: u64 = 0xA0030000;
const MAP_SIZE: usize = 4096;

const GPIO_DATA_REG_LEN: usize = 4;
const GPIO_TRI_REG_LEN: usize = 4;

struct Register<'a> {
    address: &'a mut [u8],
}

impl Register<'_> {
    fn write(&mut self, data: u32) {
        self.address.copy_from_slice(&u32::to_le_bytes(data));
    }

    fn read(&mut self) -> u32 {
        u32::from_le_bytes(self.address.try_into().unwrap())
    }
}

fn main() {
    println!("Hello, world!");

    let fd = OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_SYNC)
        .open("/dev/mem")
        .expect("Failed to open /dev/mem, are you root?");

    let mut mmap = unsafe {
        MmapOptions::new()
            .offset(AXI_GPIO_BASE)
            .len(MAP_SIZE)
            .map_mut(&fd)
            .expect("Failed to mmap")
    };

    let (addr1, mmap) = mmap.split_first_chunk_mut::<GPIO_DATA_REG_LEN>().unwrap();
    let (addr2, _) = mmap.split_first_chunk_mut::<GPIO_TRI_REG_LEN>().unwrap();

    let mut gpio_data = Register { address: addr1 };
    let mut gpio_tri = Register { address: addr2 };

    gpio_tri.write(0x00000000);

    for _ in 0..5 {
        println!("Toggling Pin HIGH ...");
        gpio_data.write(0xffffffff);
        println!("Pin value: {:x}", gpio_data.read());
        std::thread::sleep(time::Duration::from_secs(1));

        println!("Toggling Pin LOW...");
        gpio_data.write(0x00000000);
        println!("Pin value: {:x}", gpio_data.read());
        std::thread::sleep(time::Duration::from_secs(1));
    }
}
