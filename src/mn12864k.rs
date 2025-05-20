use core::cell::RefCell;

use embedded_hal_0_2::prelude::_embedded_hal_timer_CountDown;
use embedded_hal::{digital::OutputPin, spi::SpiBus};
use rp235x_hal::{self as hal, gpio::{FunctionSioOutput, PinId, PullDown}};
use hal::{spi, gpio, clocks::ClockSource};
use critical_section::Mutex;
use fugit::{ExtU32, ExtU32Ceil, RateExtU32};

use crate::gfx::draw_target::DrawTarget;

pub struct Framebuffer {
    pub buffer: [u8; 6*8*22],
}

impl Framebuffer {
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; 6*8*22]
        }
    }
}

impl Framebuffer {
    const PIXEL_OFFSETS: [usize; 6] = [0, 2, 4, 5, 3, 1];

    fn coord_to_bit_byte(position: (usize, usize)) -> (usize, usize) {
        let column = position.0 / 6;
        let pixel = (position.1 * 6) + Self::PIXEL_OFFSETS[position.0 % 6];
        let byte = (pixel / 8) + column * 8 * 6;
        let bit = pixel % 8;
        (bit, byte)
    }

    fn position_in_buffer(position: (isize, isize)) -> bool {
        position.0 >= 0 && position.0 < 128 &&
        position.1 >= 0 && position.1 < 64
    }

    pub fn set(&mut self, position: (isize, isize), color: bool) {
        if Self::position_in_buffer(position) {
            self.set_raw((position.0 as usize, position.1 as usize), color);
        }
    }

    pub fn get(&self, position: (isize, isize)) -> bool {
        if Self::position_in_buffer(position) {
            self.get_raw((position.0 as usize, position.1 as usize))
        } else {
            false
        }
    }

    pub fn set_raw(&mut self, position: (usize, usize), color: bool) {
        let (bit, byte) = Self::coord_to_bit_byte(position);
        if color {
            self.buffer[byte] |= 1 << bit;
        } else {
            self.buffer[byte] &= !(1 << bit);
        }
    }

    pub fn get_raw(&self, position: (usize, usize)) -> bool {
        let (bit, byte) = Self::coord_to_bit_byte(position);
        (self.buffer[byte] & (1 << bit)) != 0
    }

    pub fn clear(&mut self, color: bool) {
        if color {
            self.buffer.fill(0xFF);
        } else {
            self.buffer.fill(0x00);
        }
    }
}

impl DrawTarget for Framebuffer {
    fn set_pixel(&mut self, position: (isize, isize), color: bool) {
        self.set(position, color);
    }

    fn get_pixel(&self, position: (isize, isize)) -> bool {
        self.get(position)
    }
}

struct SwapChainState {
    pending_read: Option<usize>,
    free: [Option<usize>; 3],
}

impl SwapChainState {
    pub const fn new() -> Self {
        Self {
            pending_read: None,
            free: [Some(0), Some(1), Some(2)]
        }
    }

    pub fn get_next_target(&mut self) -> Option<usize> {
        for i in 0..3 {
            match self.free[i].take() {
                Some(i) => return Some(i),
                None => {}
            }
        }
        None
    }

    pub fn present(&mut self, index: usize) {
        if let Some(old_pending) = self.pending_read {
            self.free(old_pending);
        }
        self.pending_read = Some(index);
    }

    pub fn free(&mut self, index: usize) {
        for i in 0..3 {
            if self.free[i].is_none() {
                self.free[i] = Some(index);
                break;
            }
        }
    }

    pub fn pop_presented(&mut self) -> Option<usize> {
        self.pending_read.take()
    }
}

pub struct SwapChain {
    framebuffers: [Framebuffer; 3],
    state: Mutex<RefCell<SwapChainState>>,
}

unsafe impl Sync for SwapChain {}
unsafe impl Send for SwapChain {}

pub struct SwapChainTarget<'a> {
    swapchain: &'a mut SwapChain,
    index: usize,
    presented: bool,
}

impl SwapChainTarget<'_> {
    pub fn framebuffer(&mut self) -> &mut Framebuffer {
        &mut self.swapchain.framebuffers[self.index]
    }

    pub fn present(&mut self) {
        if self.presented {
            return;
        }
        critical_section::with(|cs| {
            let mut state = self.swapchain.state.borrow_ref_mut(cs);
            state.present(self.index);
        });
        self.presented = true;
    }
}

impl Drop for SwapChainTarget<'_> {
    fn drop(&mut self) {
        if self.presented {
            return;
        }
        critical_section::with(|cs| {
            let mut state = self.swapchain.state.borrow_ref_mut(cs);
            state.free(self.index);
        });
    }
}

impl SwapChain {
    pub const fn new() -> Self {
        Self {
            framebuffers: [Framebuffer::new(), Framebuffer::new(), Framebuffer::new()],
            state: Mutex::new(RefCell::new(SwapChainState::new()))
        }
    }

    pub fn acquire_next_target<'a>(&'a mut self) -> Option<SwapChainTarget<'a>> {
        let image_index = critical_section::with(|cs| {
            let mut state = self.state.borrow_ref_mut(cs);
            state.get_next_target()
        });
        image_index.map(|index| {
            SwapChainTarget {
                swapchain: self,
                index,
                presented: false,
            }
        })
    }
}

#[repr(align(16))]
struct GridBuffer {
    buff: [u8; 6]
}

impl GridBuffer {
    pub const fn new() -> Self {
        Self {
            buff: [0u8; 6]
        }
    }
}

#[repr(align(16))]
struct PixelBuffer {
    buff: [u8; 48]
}

impl PixelBuffer {
    pub const fn new() -> Self {
        Self {
            buff: [0u8; 48]
        }
    }
}

pub struct Mn12864k<GSpi: spi::SpiDevice, GSpiPinout: spi::ValidSpiPinout<GSpi>, GLatPinId: PinId, GBlkPinId: PinId, PSpi: spi::SpiDevice, PSpiPinout: spi::ValidSpiPinout<PSpi>, PLatPinId: PinId, PBlkPinId: PinId, TimerDev: hal::timer::TimerDevice> {
    gspi: spi::Spi<spi::Enabled, GSpi, GSpiPinout>,
    glat: gpio::Pin<GLatPinId, FunctionSioOutput, PullDown>,
    gblk: gpio::Pin<GBlkPinId, FunctionSioOutput, PullDown>,
    pspi: spi::Spi<spi::Enabled, PSpi, PSpiPinout>,
    plat: gpio::Pin<PLatPinId, FunctionSioOutput, PullDown>,
    pblk: gpio::Pin<PBlkPinId, FunctionSioOutput, PullDown>,
    timer: hal::Timer<TimerDev>,
    swapchain: &'static mut SwapChain,
    grid_buffer: GridBuffer,
    pixel_buffer: PixelBuffer,
}

impl <GSpi: spi::SpiDevice, GSpiPinout: spi::ValidSpiPinout<GSpi>, GLatPinId: PinId, GBlkPinId: PinId, PSpi: spi::SpiDevice, PSpiPinout: spi::ValidSpiPinout<PSpi>, PLatPinId: PinId, PBlkPinId: PinId, TimerDev: hal::timer::TimerDevice> 
    Mn12864k<GSpi, GSpiPinout, GLatPinId, GBlkPinId, PSpi, PSpiPinout, PLatPinId, PBlkPinId, TimerDev> {
    
    pub fn new(
        resets: &mut hal::pac::RESETS,
        clocks: &hal::clocks::ClocksManager,
        gspi: spi::Spi<spi::Disabled, GSpi, GSpiPinout, 8>,
        mut glat: gpio::Pin<GLatPinId, gpio::FunctionSioOutput, gpio::PullDown>,
        mut gblk: gpio::Pin<GBlkPinId, gpio::FunctionSioOutput, gpio::PullDown>,
        pspi: spi::Spi<spi::Disabled, PSpi, PSpiPinout, 8>,
        mut plat: gpio::Pin<PLatPinId, gpio::FunctionSioOutput, gpio::PullDown>,
        mut pblk: gpio::Pin<PBlkPinId, gpio::FunctionSioOutput, gpio::PullDown>,
        timer: hal::Timer<TimerDev>,
        swapchain: &'static mut SwapChain,
    ) -> Self {
        _ = glat.set_state(gpio::PinState::Low);
        _ = gblk.set_state(gpio::PinState::Low);
        _ = plat.set_state(gpio::PinState::High);
        _ = pblk.set_state(gpio::PinState::Low);
        Self {
            gspi: gspi.init(resets, clocks.peripheral_clock.get_freq(), 4u32.MHz(), embedded_hal::spi::MODE_1),
            glat,
            gblk,
            pspi: pspi.init(resets, clocks.peripheral_clock.get_freq(), 4u32.MHz(), embedded_hal::spi::MODE_1),
            plat,
            pblk,
            timer,
            swapchain,
            grid_buffer: GridBuffer::new(),
            pixel_buffer: PixelBuffer::new(),
        }
    }

    const PATTERN_A: u8 = 0b01010101;
    const PATTERN_B: u8 = 0b10101010;
    const PATTERN_C: [u8; 3] = [0b01_000101, 0b00101_0001, 0b000101_00];

    pub fn run(&mut self) -> ! {
        let mut framebuffer_index = None;

        let mut g = 1;

        let mut count_down = self.timer.count_down();
        let mut blk_count_down = self.timer.count_down();

        blk_count_down.start(5u32.micros_at_least());

        loop {
            framebuffer_index = critical_section::with(|cs| {
                let mut swapchain_state = self.swapchain.state.borrow_ref_mut(cs);
                match (swapchain_state.pop_presented(), framebuffer_index) {
                    (Some(new), Some(old)) => {
                        swapchain_state.free(old);
                        Some(new)
                    },
                    (Some(new), None) => Some(new),
                    (None, Some(old)) => Some(old),
                    _ => None
                }
            });
            if let Some(framebuffer_index) = &framebuffer_index {
                let framebuffer = &self.swapchain.framebuffers[*framebuffer_index];

                let column = (g - 1) / 2;
                match (g % 2, g == 43) {
                    // note 10
                    (1, false) => {
                        for i in 0..6*8 {
                            self.pixel_buffer.buff[i] = !((framebuffer.buffer[column * 6 * 8 + i] & Self::PATTERN_A).reverse_bits());
                        }
                    },
                    // note 11
                    (0, false) => {
                        for i in 0..6*8 {
                            self.pixel_buffer.buff[i] = !((framebuffer.buffer[column * 6 * 8 + i] & Self::PATTERN_B).reverse_bits());
                        }
                    },
                    // note 12
                    (_, true) => {
                        for i in 0..2*8 {
                            self.pixel_buffer.buff[i * 3 + 0] = !((framebuffer.buffer[column * 6 * 8 + i * 3 + 0] & Self::PATTERN_C[0]).reverse_bits());
                            self.pixel_buffer.buff[i * 3 + 1] = !((framebuffer.buffer[column * 6 * 8 + i * 3 + 1] & Self::PATTERN_C[1]).reverse_bits());
                            self.pixel_buffer.buff[i * 3 + 2] = !((framebuffer.buffer[column * 6 * 8 + i * 3 + 2] & Self::PATTERN_C[2]).reverse_bits());
                        }
                    },
                    _ => unreachable!()
                }

                _ = SpiBus::write(&mut self.pspi, &self.pixel_buffer.buff[..]);

                self.grid_buffer.buff = [0xFFu8; 48 / 8];
                self.grid_buffer.buff[(g - 1) / 8] &= !(1u8 << ((g - 1) % 8)).reverse_bits();
                self.grid_buffer.buff[ g      / 8] &= !(1u8 << ( g      % 8)).reverse_bits();
                
                _ = SpiBus::write(&mut self.gspi, &self.grid_buffer.buff[..]);

                _ = SpiBus::flush(&mut self.gspi);
                _ = SpiBus::flush(&mut self.pspi);

                _ = nb::block!(blk_count_down.wait());

                count_down.start(100u32.nanos());
                _ = nb::block!(count_down.wait());

                _ = self.gblk.set_state(gpio::PinState::Low);
                _ = self.pblk.set_state(gpio::PinState::Low);

                count_down.start(100u32.nanos());
                _ = nb::block!(count_down.wait());

                _ = self.plat.set_state(gpio::PinState::Low);
                _ = self.glat.set_state(gpio::PinState::Low);

                count_down.start(100u32.nanos());
                _ = nb::block!(count_down.wait());

                _ = self.plat.set_state(gpio::PinState::High);
                _ = self.glat.set_state(gpio::PinState::High);

                count_down.start(100u32.nanos());
                _ = nb::block!(count_down.wait());

                _ = self.gblk.set_state(gpio::PinState::High);
                _ = self.pblk.set_state(gpio::PinState::High);

                blk_count_down.start(5u32.micros_at_least());

                g %= 43;
                g += 1;
            }
        }
    }

}



