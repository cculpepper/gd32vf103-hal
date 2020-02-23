//! Timers
use crate::time::Hertz;
use crate::pac::TIMER6;
use crate::rcu::{Clocks, APB1};
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::timer::CountDown;

// I'd prefer using Timer<TIMERx> for convenience
/// Timer object
pub struct Timer<TIMER> {
    timer: TIMER,
    clock_scaler: u16,
    clock_frequency: Hertz,
} 


pub(crate) mod sealed {
    pub trait Remap {
        type Periph;
        const REMAP: u8;
    }
    pub trait Ch1<REMAP> {}
    pub trait Ch2<REMAP> {}
    pub trait Ch3<REMAP> {}
    pub trait Ch4<REMAP> {}
}

impl Timer<TIMER6> {
    /// Initialize the timer. 
    /// 
    /// An enable and reset procedure is procceed to peripheral to clean its state.
    pub fn timer6(timer: TIMER6, clock: Clocks, apb1: &mut APB1) -> Self {
        riscv::interrupt::free(|_| {
            apb1.en().modify(|_, w| w.timer6en().set_bit());
            apb1.rst().write(|w| w.timer6rst().set_bit());
            apb1.rst().write(|w| w.timer6rst().clear_bit());
        });
        Timer {
            timer: timer,
            clock_scaler: 1000,
            clock_frequency: clock.ck_apb1(),
        }
    }
}

impl<TIMER> Timer<TIMER> {
    // in future designs we do not stop timer in this function
    // but prefer using Timer<TIMER>::start(self, ...) -> SomeTimer
    // when SomeTimer should be stopped, it has function returns timer back
    // as SomeTimer::stop(self) -> Timer<TIMER>.
    /// Release the timer, return its ownership.
    pub fn release(self) -> TIMER {
        self.timer
    }
}

impl<T: Into<u32>> DelayMs<T> for Timer<TIMER6> {
    fn delay_ms(&mut self, ms: T) {
        let count = (ms.into() * self.clock_frequency.0) / (self.clock_scaler as u32 * 1000);
        if count > u16::max_value() as u32 {
            panic!("can not delay that long");
        }
        self.start(count as u16);
        nb::block!(self.wait()).ok();
    }
}

impl CountDown for Timer<TIMER6> {
    type Time = u16;
    
    fn start<T>(&mut self, count: T)
    where
        T: Into<Self::Time>,
    {
        unsafe {
            let c = count.into();
            riscv::interrupt::free(|_| {
                self.timer.psc.write(|w| w.psc().bits(self.clock_scaler));
                self.timer.intf.write(|w| w.upif().clear_bit());
                self.timer.swevg.write(|w| w.upg().set_bit());
                self.timer.intf.write(|w| w.upif().clear_bit());
                self.timer.car.modify(|_, w| w.carl().bits(c));
                self.timer.ctl0.modify(|_, w| w.cen().set_bit());
            });
        }
    }

    //TODO this signature changes in a future version, so we don'ot need the void crate.
    fn wait(&mut self) -> nb::Result<(), void::Void> {
        let flag = self.timer.intf.read().upif().bit_is_set();
        if flag {
            return Ok(());
        } else {
            return Err(nb::Error::WouldBlock);
        }
    }
}

// impl Periodic for Timer<TIMER2> {}
