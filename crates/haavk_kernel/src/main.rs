//! HAAVK Mandel Bare Metal Kernel
//! 曼德尔裸机内核（x86_64 no_std）
//!
//! 启动流程：
//!   BIOS/UEFI → bootloader → _start → kernel_main
//!   → 初始化 VGA 文本模式 → 打印 HAAVK 标志 → 死循环
//!
//! 构建（需在有 Rust nightly + qemu 的环境）：
//!   rustup target add x86_64-havk
//!   cargo xbuild --target x86_64-havk
//!   cargo run --release  # QEMU 启动

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

/// VGA 文本模式显存基地址（0xB8000，80x25 字符，每字符 2 字节）
const VGA_BUFFER: *mut u16 = 0xB8000 as *mut u16;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

/// VGA 颜色
#[allow(dead_code)]
mod color {
    pub const BLACK: u8 = 0;
    pub const BLUE: u8 = 1;
    pub const CYAN: u8 = 0x0B;
    pub const WHITE: u8 = 0x0F;
    pub const BRIGHT_CYAN: u8 = 0x0B;
}

struct VgaWriter {
    row: usize,
    col: usize,
    color: u8,
}

impl VgaWriter {
    fn new() -> Self {
        Self { row: 0, col: 0, color: color::WHITE }
    }

    fn clear(&mut self) {
        for i in 0..(VGA_WIDTH * VGA_HEIGHT) {
            unsafe { VGA_BUFFER.add(i).write_volatile(0x0F00); }
        }
        self.row = 0;
        self.col = 0;
    }

    fn write_byte(&mut self, b: u8) {
        match b {
            b'\n' => {
                self.col = 0;
                self.row += 1;
            }
            b'\r' => self.col = 0,
            _ => {
                let entry = (self.color as u16) << 8 | b as u16;
                unsafe {
                    VGA_BUFFER.add(self.row * VGA_WIDTH + self.col).write_volatile(entry);
                }
                self.col += 1;
                if self.col >= VGA_WIDTH {
                    self.col = 0;
                    self.row += 1;
                }
            }
        }
        if self.row >= VGA_HEIGHT {
            self.row = 0;
        }
    }

    fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut vga = VgaWriter::new();
    vga.clear();

    vga.color = color::BRIGHT_CYAN;
    vga.write_str("========================================\n");
    vga.write_str("   HAAVK OS · MANDLE BARE METAL KERNEL\n");
    vga.write_str("   sky belongs to haavk\n");
    vga.write_str("   new world is at your ears\n");
    vga.write_str("========================================\n");
    vga.write_str("\n");

    vga.color = color::WHITE;
    vga.write_str("Mandel Core v0.9 (bare-metal x86_64)\n");
    vga.write_str("Kernel entry: _start @ 0x100000\n");
    vga.write_str("VGA text mode initialized: 80x25\n");
    vga.write_str("CPU: x86_64 (long mode)\n");
    vga.write_str("\n");
    vga.write_str(">>> HAAVK kernel ready. halting...\n");

    halt();
}

fn halt() -> ! {
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut vga = VgaWriter::new();
    vga.color = 0x4C; // 红底浅红
    vga.write_str("\n*** KERNEL PANIC ***\n");
    halt();
}
