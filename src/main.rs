#![no_std]
#![no_main]
#![allow(unused)]
#![allow(static_mut_refs)]

use core::cell::RefCell;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;
use application::{AppSharedState, ButtonState, EncoderState, InputState};
use embedded_hal::digital::{InputPin, OutputPin};
use fugit::{Duration, ExtU32, RateExtU32};
use mn12864k::SwapChain;
use panic_halt as _;

use qcw_com::RemoteMessage;
use rp235x_hal::clocks::ClockSource;
use rp235x_hal::gpio::{FunctionSioInput, FunctionSioOutput, FunctionUart, PullBusKeep, PullNone, PullUp};
use rp235x_hal::pac::otp::critical;
use rp235x_hal::timer::Alarm;
use rp235x_hal::uart::UartPeripheral;
use rp235x_hal::Spi;
use rp235x_hal as hal;
use hal::pac as pac;
use pac::interrupt;
use hal::entry;
use embedded_hal::pwm::SetDutyCycle;

extern crate alloc;
use embedded_alloc::LlffHeap as Heap;

use alloc::format;

mod gfx;
mod mn12864k;
mod application;
mod app_views;
mod ui;

use qcw_com::*;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const XTAL_FREQ_HZ: u32 = 12_000_000;

#[link_section = ".start_block"]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

static CORE1_STACK: hal::multicore::Stack<4096> = hal::multicore::Stack::new();

struct Encoder {
    pub a_pin: hal::gpio::Pin<hal::gpio::bank0::Gpio28, hal::gpio::FunctionSioInput, PullNone>,
    pub b_pin: hal::gpio::Pin<hal::gpio::bank0::Gpio29, hal::gpio::FunctionSioInput, PullNone>,
    pub last_state: (bool, bool),
}

static GLOBAL_ENCODER: critical_section::Mutex<RefCell<Option<Encoder>>> = 
    critical_section::Mutex::new(RefCell::new(None));

static ENCODER_COUNT: AtomicI32 = AtomicI32::new(0);

struct Buttons {
    pub column_0_pin: hal::gpio::Pin<hal::gpio::bank0::Gpio8, FunctionSioOutput, PullNone>,
    pub column_1_pin: hal::gpio::Pin<hal::gpio::bank0::Gpio9, FunctionSioOutput, PullNone>,
    pub row_0_pin: hal::gpio::Pin<hal::gpio::bank0::Gpio4, FunctionSioInput, PullNone>,
    pub row_1_pin: hal::gpio::Pin<hal::gpio::bank0::Gpio7, FunctionSioInput, PullNone>,
    pub scan_alarm: hal::timer::Alarm0<hal::timer::CopyableTimer0>,
    pub column_0: bool,
}

static GLOBAL_BUTTONS: critical_section::Mutex<RefCell<Option<Buttons>>> = 
    critical_section::Mutex::new(RefCell::new(None));

static BUTTON_STATES: [AtomicBool; 4] = [
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
    AtomicBool::new(false),
];

const BUTTON_E: usize = 3;
const BUTTON_0: usize = 1;
const BUTTON_1: usize = 0;
const BUTTON_2: usize = 2;

#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 8192;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(&raw mut HEAP_MEM as usize, HEAP_SIZE) }
    }

    let mut sio = hal::Sio::new(pac.SIO);

    let pins = hal::gpio::Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut pac.RESETS);

    let mut timer = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS, &clocks);

    let mut button_scan_alarm = timer.alarm_0().unwrap();

    let mut button_column_0_pin = pins.gpio8.into_push_pull_output().into_pull_type();
    let mut button_column_1_pin = pins.gpio9.into_push_pull_output().into_pull_type();
    let button_row_0_pin = pins.gpio4.into_floating_input();
    let button_row_1_pin = pins.gpio7.into_floating_input();

    critical_section::with(move |cs| {
        button_scan_alarm.cancel();
        button_scan_alarm.clear_interrupt();
        button_scan_alarm.enable_interrupt();
        button_scan_alarm.schedule(5u32.millis());

        button_column_0_pin.set_high();
        button_column_1_pin.set_low();

        *GLOBAL_BUTTONS.borrow_ref_mut(cs) = Some(
            Buttons {
                column_0_pin: button_column_0_pin,
                column_1_pin: button_column_1_pin,
                row_0_pin: button_row_0_pin,
                row_1_pin: button_row_1_pin,
                scan_alarm: button_scan_alarm,
                column_0: true
            }
        );
    });

    let mut encoder_a_pin = pins.gpio28.into_floating_input();
    let mut encoder_b_pin = pins.gpio29.into_floating_input();
    encoder_a_pin.set_schmitt_enabled(true);
    encoder_b_pin.set_schmitt_enabled(true);
    
    critical_section::with(move |cs| {
        encoder_a_pin.clear_interrupt(hal::gpio::Interrupt::EdgeLow);
        encoder_a_pin.set_interrupt_enabled(hal::gpio::Interrupt::EdgeLow, true);
        encoder_a_pin.clear_interrupt(hal::gpio::Interrupt::EdgeHigh);
        encoder_a_pin.set_interrupt_enabled(hal::gpio::Interrupt::EdgeHigh, true);
        encoder_b_pin.clear_interrupt(hal::gpio::Interrupt::EdgeLow);
        encoder_b_pin.set_interrupt_enabled(hal::gpio::Interrupt::EdgeLow, true);
        encoder_b_pin.clear_interrupt(hal::gpio::Interrupt::EdgeHigh);
        encoder_b_pin.set_interrupt_enabled(hal::gpio::Interrupt::EdgeHigh, true);

        let last_state = (encoder_a_pin.is_high().unwrap_or_default(), encoder_b_pin.is_high().unwrap_or_default());

        *GLOBAL_ENCODER.borrow_ref_mut(cs) = Some(Encoder {
            a_pin: encoder_a_pin,
            b_pin: encoder_b_pin,
            last_state,
        });
    });

    let txpin = pins.gpio0.into_floating_disabled().into_function();
    let rxpin = pins.gpio1.into_floating_disabled().into_function();

    let uart = hal::uart::UartPeripheral::new(pac.UART0, (txpin, rxpin), &mut pac.RESETS);
    let mut uart_config = hal::uart::UartConfig::new(10u32.kHz(), hal::uart::DataBits::Eight, None, hal::uart::StopBits::One);
    let mut uart = uart.enable(uart_config, clocks.peripheral_clock.get_freq()).unwrap();
    uart.set_fifos(true);

    unsafe {
        cortex_m::peripheral::NVIC::unmask(hal::pac::Interrupt::IO_IRQ_BANK0);
        cortex_m::peripheral::NVIC::unmask(hal::pac::Interrupt::TIMER0_IRQ_0);
    }

    let pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    let mut pwm3 = pwm_slices.pwm3;

    pwm3.channel_a.output_to(pins.gpio22);
    pwm3.channel_b.output_to(pins.gpio23);
    pwm3.set_ph_correct();
    pwm3.set_div_int(1);
    pwm3.set_div_frac(0);
    pwm3.set_top(950);
    _ = pwm3.channel_a.set_duty_cycle((950 / 2) - 200);
    _ = pwm3.channel_b.set_duty_cycle((950 / 2) + 200);
    pwm3.channel_b.set_inverted();
    pwm3.enable();
    pwm3.channel_a.set_enabled(true);
    pwm3.channel_b.set_enabled(true);

    let gsin = pins.gpio27.into_function();
    let gclk = pins.gpio26.into_function();
    let glat = pins.gpio20.into_push_pull_output_in_state(hal::gpio::PinState::Low);
    let gblk = pins.gpio21.into_push_pull_output_in_state(hal::gpio::PinState::Low);

    let gspi = Spi::<_, _, _, 8>::new(pac.SPI1, (gsin, gclk));

    let psin = pins.gpio3.into_function();
    let pclk = pins.gpio6.into_function();
    let plat = pins.gpio2.into_push_pull_output_in_state(hal::gpio::PinState::High);
    let pblk = pins.gpio5.into_push_pull_output_in_state(hal::gpio::PinState::Low);

    let pspi = Spi::<_, _, _, 8>::new(pac.SPI0, (psin, pclk));

    let mut swapchain = SwapChain::new();
    let swapchain_ptr = &mut swapchain as *mut _;

    let mut mn12864k = mn12864k::Mn12864k::new(&mut pac.RESETS, &clocks, gspi, glat, gblk, pspi, plat, pblk, timer, unsafe { &mut *swapchain_ptr });

    let mut mc = hal::multicore::Multicore::new(&mut pac.PSM, &mut pac.PPB, &mut sio.fifo);
    let cores = mc.cores();

    _ = cores[1].spawn(CORE1_STACK.take().unwrap(), move || {
        mn12864k.run();
    });

    let mut last_t = timer.get_counter();

    let mut previous_button_states: [bool; 4] = [false; 4];
    let mut previous_encoder_count: i32 = 0;

    let shared_state = AppSharedState {
    };

    let mut application = application::Application::new(shared_state);

    let mut rx_buffer = SerialBuffer::<512>::new();
    let mut tx_buffer = SerialBuffer::<512>::new();
    let mut incoming_messages = VecDeque::new();
    let mut outgoing_messages = VecDeque::new();

    loop {
        let now = timer.get_counter();
        let delta_t = now - last_t;
        last_t = now;

        let encoder_count = ENCODER_COUNT.load(Ordering::SeqCst) / 4;
        let encoder_delta_a = - encoder_count.wrapping_sub(previous_encoder_count);
        let encoder_delta_b = previous_encoder_count.wrapping_sub(encoder_count);
        let encoder_delta = if encoder_delta_a.abs() <= encoder_delta_b.abs() {
            encoder_delta_a
        } else {
            encoder_delta_b
        };
        previous_encoder_count = encoder_count;

        let buttons_state_pressed_released = [0, 1, 2, 3].map(|i| {
            let current_state = BUTTON_STATES[i].load(Ordering::SeqCst);
            let previous_state = previous_button_states[i];
            let pressed = current_state && !previous_state;
            let released = !current_state && previous_state;
            previous_button_states[i] = current_state;
            (current_state, pressed, released)
        });
        
        let input_state = application::InputState {
            encoder: EncoderState {
                count: encoder_count,
                delta: encoder_delta,
                button: ButtonState {
                    down: buttons_state_pressed_released[BUTTON_E].0,
                    pressed: buttons_state_pressed_released[BUTTON_E].1,
                    released: buttons_state_pressed_released[BUTTON_E].2,
                }
            },
            buttons: [
                ButtonState {
                    down: buttons_state_pressed_released[BUTTON_0].0,
                    pressed: buttons_state_pressed_released[BUTTON_0].1,
                    released: buttons_state_pressed_released[BUTTON_0].2,
                },
                ButtonState {
                    down: buttons_state_pressed_released[BUTTON_1].0,
                    pressed: buttons_state_pressed_released[BUTTON_1].1,
                    released: buttons_state_pressed_released[BUTTON_1].2,
                },
                ButtonState {
                    down: buttons_state_pressed_released[BUTTON_2].0,
                    pressed: buttons_state_pressed_released[BUTTON_2].1,
                    released: buttons_state_pressed_released[BUTTON_2].2,
                },
            ]
        };

        while uart.uart_is_readable() {
            let mut bytes = [0u8; 8];
            if let Ok(count) = uart.read_raw(&mut bytes[..]) {
                for b in 0..count {
                    rx_buffer.push(bytes[b]);
                }
                if count == 0 {
                    break;
                }
            } else {
                break;
            }
        }

        while let Some(message) = RemoteMessage::try_receive(&mut rx_buffer).unwrap() {
            incoming_messages.push_back(message);
        };

        let com_state = application::ComState {
            inbox: &mut incoming_messages,
            outbox: &mut outgoing_messages
        };

        application.update(delta_t.to_micros(), input_state, com_state);

        while let Some(message) = outgoing_messages.front() {
            if !message.try_send(&mut tx_buffer) {
                break;
            }
            outgoing_messages.pop_front();
        }

        while uart.uart_is_writable() {
            if let Some(byte) = tx_buffer.peek() {
                if let Ok(remaining) = uart.write_raw(&[byte]) {
                    if remaining.len() == 0 {
                        tx_buffer.pop();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if let Some(mut target) = swapchain.acquire_next_target() {
            let framebuffer = target.framebuffer();
            framebuffer.clear(false);
            application.render(framebuffer);
            target.present();
        }
    }
}

static ENCODER_PIN_STATE: (AtomicBool, AtomicBool) = (AtomicBool::new(false), AtomicBool::new(false));

#[interrupt]
fn IO_IRQ_BANK0() {
    critical_section::with(|cs| {
        if let Some(encoder_pins) = GLOBAL_ENCODER.borrow_ref_mut(cs).as_mut() {
            if encoder_pins.a_pin.interrupt_status(hal::gpio::Interrupt::EdgeHigh) |
               encoder_pins.a_pin.interrupt_status(hal::gpio::Interrupt::EdgeLow ) |
               encoder_pins.b_pin.interrupt_status(hal::gpio::Interrupt::EdgeHigh) |
               encoder_pins.b_pin.interrupt_status(hal::gpio::Interrupt::EdgeLow ) {

                for _ in 0..5 {
                    cortex_m::asm::nop();
                }

                let current_state = (encoder_pins.a_pin.is_high().unwrap_or_default(), encoder_pins.b_pin.is_high().unwrap_or_default());

                let inc = match (encoder_pins.last_state, current_state) {
                    ((false, false), (false, true )) =>  1,
                    ((false, false), (true , false)) => -1,
                    ((false, false), _             ) =>  0,
                    ((false, true ), (false, false)) => -1,
                    ((false, true ), (true , true )) =>  1,
                    ((false, true ), _             ) =>  0,
                    ((true , true ), (false, true )) => -1,
                    ((true , true ), (true , false)) =>  1,
                    ((true , true ), _             ) =>  0,
                    ((true , false), (true , true )) => -1,
                    ((true , false), (false, false)) =>  1,
                    ((true , false), _             ) =>  0,
                };
                encoder_pins.last_state = current_state;

                ENCODER_COUNT.fetch_add(inc, Ordering::SeqCst);

                encoder_pins.a_pin.clear_interrupt(hal::gpio::Interrupt::EdgeHigh);
                encoder_pins.a_pin.clear_interrupt(hal::gpio::Interrupt::EdgeLow);
                encoder_pins.b_pin.clear_interrupt(hal::gpio::Interrupt::EdgeHigh);
                encoder_pins.b_pin.clear_interrupt(hal::gpio::Interrupt::EdgeLow);

            }
        }
    });
}

#[interrupt]
fn TIMER0_IRQ_0() {
    critical_section::with(|cs| {
        if let Some(buttons) = GLOBAL_BUTTONS.borrow_ref_mut(cs).as_mut() {
            if buttons.column_0 {
                let button_0 = buttons.row_0_pin.is_high().unwrap_or_default();
                let button_1 = buttons.row_1_pin.is_high().unwrap_or_default();
                BUTTON_STATES[0].store(button_0, Ordering::SeqCst);
                BUTTON_STATES[1].store(button_1, Ordering::SeqCst);
                buttons.column_0_pin.set_low();
                buttons.column_1_pin.set_high();
            } else {
                let button_2 = buttons.row_0_pin.is_high().unwrap_or_default();
                let button_3 = buttons.row_1_pin.is_high().unwrap_or_default();
                BUTTON_STATES[2].store(button_2, Ordering::SeqCst);
                BUTTON_STATES[3].store(button_3, Ordering::SeqCst);
                buttons.column_0_pin.set_high();
                buttons.column_1_pin.set_low();
            }
            buttons.column_0 = !buttons.column_0;
            buttons.scan_alarm.cancel();
            buttons.scan_alarm.clear_interrupt();
            buttons.scan_alarm.schedule(5u32.millis());
        }
    });
}
