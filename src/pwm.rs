use crate::pac::TIMER4;
use crate::pac::AFIO;
use crate::rcu::{Clocks, APB1};
use crate::timer::Timer;

use crate::gpio::{self, gpioa, Alternate, Mode, PushPull};
use crate::gpio::gpioa::*;
use crate::time::Hertz;
// use crate::bb;
use embedded_hal;

use core::mem;


// pub struct C1;

pub trait Pins<P>{
    const C1: bool = false;
    type Channels;
}

mod private {
    pub trait Sealed{}
}
pub trait C1<TIMER>: private::Sealed{}

impl private::Sealed for PA0<Alternate<PushPull>>{}
impl C1<TIMER4> for PA0<Alternate<PushPull>>{}
impl<TIMER, P1> Pins<C1> for C1 
    // where P1 : gpio::Mode<Alternate<PushPull>>,
          // TIMER : TIMER4,
    where P1: C1<TIMER4>,
    

{

    const C1:bool = true;
    type Channels = Pwm<TIMER, C1>;
}


pub struct Pwm<TIMER, CHANNEL> {
    _channel: CHANNEL,
    _timer: TIMER,
}



impl Timer<TIMER4> {
    pub fn pwm<P, PINS>(
        self,
        _pins: PINS,
        freq: Hertz,
        clock: Clocks,
        ) -> PINS::Channels
        where
            PINS: Pins<P>
        {
            todo!();
            // // mapr.modify_mapr(|_, w| w.timer4_remap().bit(REMAP::REMAP == 1));

            // let Self { timer, clock_scaler, clock_frequency } = self;
            // // Frequency going into the timer is CK_TIMER4
// // This should be timer4, pins, pwm preq and clock frequency
            let Self {timer} = self;
            timer4(timer, _pins, freq.into(), clock.ck_apb1())
        }
}

fn timer4<P, PINS>(
    timer: TIMER4,
    _pins: PINS,
    freq: Hertz,
    clk: Hertz,
    ) -> PINS::Channels where
        PINS: Pins<P>
{
    if PINS::C1 {
        unsafe {
            timer.chctl0_output().modify(|_,w|{
                w
                    // This is just going to use PWM mode 1, bits 111 in the CH0COMCTL
                    // It should be good enough for now to get pwm working
                    .ch0comctl().bits(0b111)
            });
        }
    }

    let ticks = clk.0 / freq.0;
    let psc = (ticks / (1 << 16)) as u16;
    let car = (ticks / (psc + 1) as u32) as u16;
    unsafe{
        timer.psc.write(|w| w.psc().bits(psc) );
        timer.car.write(|w| w.carl().bits(car));

        timer.ctl0.write(|w|
                         w
                         .cam().bits(0b00) // Edge aligned
                         .dir().clear_bit() // Count up
                         .spm().clear_bit() // Don't stop the counter
                         .cen().set_bit() // Enable the counter
                        );
        mem::MaybeUninit::uninit().assume_init()
    }

}

// Channel 1
impl embedded_hal::PwmPin for Pwm<TIMER4, C1> {
    type Duty = u16;

    fn disable(&mut self) {
        self.timer.chctl2.modify(|_,w|
            w
                .ch0en().clear_bit()
        );
        // bb::clear(&(*$TIMERX::ptr()).ccer, 0)
    }

    fn enable(&mut self) {
        self.timer.chctl2.modify(|_,w|
            w
                .ch0en().set_bit()
        );
        // unsafe { bb::set(&(*$TIMERX::ptr()).ccer, 0) }
    }

    fn get_duty(&self) -> u16 {
        // unsafe { (*$TIMERX::ptr()).ccr1.read().ccr().bits() }
        10
    }

    fn get_max_duty(&self) -> u16 {
        // unsafe { (*$TIMERX::ptr()).arr.read().arr().bits() }
        10
    }

    fn set_duty(&mut self, duty: u16) {
        // unsafe { (*$TIMERX::ptr()).ccr1.write(|w| w.ccr().bits(duty)) }
    }
}

