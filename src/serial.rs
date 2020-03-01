//! (TODO) Serial Communication (USART)

#![macro_use]

use crate::pac;
use core::fmt::Write;
use pac::{GPIOA, RCU, USART0};

//TODO - use the APB/RCU/GPIO primitives in this crate, rather than unsafe memory poking!

// yay, math functions arn't implemented in core!
fn round(n: f32) -> f32 {
    let int_part: i32 = n as i32; //truncate
    let fraction_part: f32 = n - int_part as f32;
    if fraction_part >= 0.5 {
        return (int_part + 1) as f32;
    } else {
        return int_part as f32;
    }
}

fn init_usart() {
    // enable the peripheral clock
    unsafe {
        (*USART0::ptr()).ctl0.modify(|r, w| {
            w.bits(r.bits()).uen().clear_bit() //disable while being configured TODO could wait for TC=1?
        });

        (*RCU::ptr()).apb2en.modify(|r, w| {
            w.bits(r.bits())
                .usart0en()
                .set_bit()
                .afen()
                .set_bit()
                .paen()
                .set_bit()
        });

        (*GPIOA::ptr()).ctl1.modify(|r, w| {
            w.bits(r.bits())
                .md9()
                .bits(0b11) //output, 50mhz
                .ctl9()
                .bits(0b10) //alternate push pull
                .md10()
                .bits(0b00) //input
                .ctl10()
                .bits(0b01) //floating
        });

        // for 9600 baud rate @ 8mhz clock, USARTDIV = CLK/(16*baud)
        // USARTDIV = 8000000/(16*9600) = 52.0833333
        // integer part = 52, fractional ~= 1/16 -> intdiv=53, fradiv=1
        // can calculate automatically given clk and baud, but note that
        // if fradiv=16, then intdiv++; fradiv=0;

        let _baud = 9600f32;
        let clk_freq = 8_000_000f32;
        let usart_div = clk_freq / (16f32 * 9600f32);
        let mut int_div = usart_div as i32; //note that trunc(), fract(), rount() are not implemented in core...
        let mut fra_div = round(16.0 * (usart_div - int_div as f32)) as i32;
        if fra_div == 16 {
            int_div += 1;
            fra_div = 0;
        }

        (*USART0::ptr()).baud.modify(|r, w| {
            w.bits(r.bits())
                .intdiv()
                .bits(int_div as u16)
                .fradiv()
                .bits(fra_div as u8)
        });

        (*USART0::ptr()).ctl2.modify(|r, w| {
            w.bits(r.bits())
                .ctsen()
                .clear_bit() //enable CTS hardware flow control
                .rtsen()
                .clear_bit() //enable RTS hardware flow control
        });

        (*USART0::ptr()).ctl1.modify(|r, w| {
            w.bits(r.bits())
                .stb()
                .bits(0b00) //set # of stop bits = 1
                .cken()
                .clear_bit()
        });

        (*USART0::ptr()).ctl0.modify(|r, w| {
            w.bits(r.bits())
                .wl()
                .clear_bit() //set word size to 8
                .ten()
                .set_bit() //enable tx
                .ren()
                .set_bit() //enable rx
                .pcen()
                .clear_bit() //no parity check function plz
                .pm()
                .clear_bit() //0=even parity 1=odd parity
                .uen()
                .set_bit() //enable the uart, yay!
        });
    }
}

/// todo: more proper name
#[doc(hidden)] // experimental, not for practical use
pub struct SerialWrapper;

impl core::fmt::Write for SerialWrapper {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &byte in s.as_bytes() {
            unsafe {
                (*USART0::ptr()).data.write(|w| w.data().bits(byte.into()));
                while (*USART0::ptr()).stat.read().tbe().bit_is_clear() {}
            }
        }
        Ok(())
    }
}

// hold things in a static place
static mut STDOUT: Option<SerialWrapper> = None;

#[allow(unused_variables)]
#[doc(hidden)] // experimental, not for practical use
pub fn init_stdout(uart: USART0) {
    init_usart();
    unsafe {
        STDOUT.replace(SerialWrapper {});
    }
}

/// Writes string to stdout
#[doc(hidden)] // experimental, not for practical use
pub fn write_str(s: &str) {
    unsafe {
        if let Some(stdout) = STDOUT.as_mut() {
            let _ = stdout.write_str(s);
        } else {
            panic!("couldn't get stdout!");
        }
    }
}

/// Writes formatted string to stdout
#[doc(hidden)] // experimental, not for practical use
pub fn write_fmt(args: core::fmt::Arguments) {
    unsafe {
        if let Some(stdout) = STDOUT.as_mut() {
            let _ = stdout.write_fmt(args);
        } else {
            panic!("couldn't get stdout!");
        }
    }
}

/// Macro for printing to the serial standard output
#[doc(hidden)] // experimental
#[macro_export]
macro_rules! sprint {
    ($s:expr) => {
        crate::serial::write_str($s)
    };
    ($($tt:tt)*) => {
        crate::serial::write_fmt(format_args!($($tt)*))
    };

}

// --- //

// use crate::pac::USART0;
use crate::afio::PCF0;
use crate::gpio::gpioa::{PA10, PA9};
use crate::gpio::{Alternate, Floating, Input, PushPull};
use crate::rcu::{Clocks, APB2};
use crate::time::Bps;

/// Serial config
pub struct Config {
    pub baudrate: Bps,
    pub parity: Parity,
    // pub stop_bits
}

/// Serial parity
pub enum Parity {
    ParityNone,
    ParityEven,
    ParityOdd,
}

impl Parity {
    // (word_length, parity_enable, parity_config)
    // word_length: 0 => 8 bits; 1 => 9 bits
    // parity_enable: 0 => disable; 1 => enable
    // parity_config: 0 => odd; 1 => even
    #[inline]
    fn config(&self) -> (bool, bool, bool) {
        match *self {
            Parity::ParityNone => (false, false, false),
            Parity::ParityEven => (true, true, false),
            Parity::ParityOdd => (true, true, true),
        }
    }
}

/// Serial abstraction
pub struct Serial<USART, PINS> {
    usart: USART,
    pins: PINS,
}

impl<PINS> Serial<USART0, PINS> {
    pub fn usart0(
        usart0: USART0,
        pins: PINS,
        pcf0: &mut PCF0,
        config: Config,
        clocks: Clocks,
        apb2: &mut APB2,
    ) -> Self
    where
        PINS: Pins<USART0>,
    {
        // calculate baudrate divisor fractor
        let (intdiv, fradiv) = {
            // use apb2 or apb1 may vary
            // round the value to get most accurate one (without float point)
            let baud_div = (clocks.ck_apb2().0 + config.baudrate.0 / 2) / config.baudrate.0;
            assert!(baud_div <= 0xFFFF, "impossible baudrate");
            ((baud_div & 0xFFF0) as u16, (baud_div & 0x0F) as u8)
        };
        // get parity config
        let (wl, pcen, pm) = config.parity.config();
        riscv::interrupt::free(|_| {
            // enable and reset usart peripheral
            apb2.en().modify(|_, w| w.usart0en().set_bit());
            apb2.rst().modify(|_, w| w.usart0rst().set_bit());
            apb2.rst().modify(|_, w| w.usart0rst().clear_bit());
            // set serial remap
            pcf0.pcf0()
                .modify(|_, w| w.usart0_remap().bit(PINS::REMAP == 1));
            // does not enable DMA in this section; DMA is enabled separately
            // set baudrate
            usart0
                .baud
                .write(|w| unsafe { w.intdiv().bits(intdiv).fradiv().bits(fradiv) });
            // todo: more config on stop bits

            usart0.ctl0.modify(|_, w| {
                // set parity check settings
                w.wl().bit(wl).pcen().bit(pcen).pm().bit(pm);
                // enable the peripheral
                // todo: split receive and transmit
                w.uen().set_bit().ren().set_bit().ten().set_bit()
            });
        });
        Serial {
            usart: usart0,
            pins,
        }
    }

    pub fn release(self, apb2: &mut APB2) -> (USART0, PINS) {
        // // disable the peripheral
        // self.usart.ctl0.modify(|_, w| w.uen().clear_bit()); // todo: is this okay?
        // disable the clock
        apb2.en().modify(|_, w| w.usart0en().clear_bit());

        // return the ownership
        (self.usart, self.pins)
    }
}

pub trait Pins<USART> {
    #[doc(hidden)] // internal use only
    const REMAP: u8;
}

impl Pins<USART0> for (PA9<Alternate<PushPull>>, PA10<Input<Floating>>) {
    const REMAP: u8 = 0;
}

//todo
