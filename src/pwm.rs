use crate::pac::TIMER4;
use crate::rcu::{Clocks};
use crate::gpio::{Mode, Alternate, PushPull};
use crate::gpio::gpioa::*;


use crate::time::Hertz;
// use crate::bb;
use embedded_hal;

pub struct Channels {
    ch1: Option<PA0<Alternate<PushPull>>>,
    ch2: Option<PA1<Alternate<PushPull>>>,
    ch3: Option<PA2<Alternate<PushPull>>>,
    ch4: Option<PA3<Alternate<PushPull>>>,
}


pub struct Pwm {
    pub timer: TIMER4,
    pub frequency: Hertz,
    pub channels: Channels,
    pub clock_freq: Hertz,
}

impl Pwm{

    pub fn new(
        freq: Hertz,
        clock: Clocks,
        timer: TIMER4,
        ch1: Option<PA0<Alternate<PushPull>>>,
        ch2: Option<PA1<Alternate<PushPull>>>,
        ch3: Option<PA2<Alternate<PushPull>>>,
        ch4: Option<PA3<Alternate<PushPull>>>
        ) -> Self {
        Pwm{
            timer : timer,
            frequency: freq,
            channels : Channels{ch1, ch2, ch3, ch4},
            clock_freq: clock.ck_apb1(),
        }

    }
    pub fn init(&mut self){

        self.init_timer4(self.frequency.into(), self.clock_freq);

        if self.channels.ch1.is_some(){
            // We're assuming the GPIOs are already in alternate mode here.
            self.set_duty(1, 0);
            self.enable(1);
        }
        if self.channels.ch2.is_some(){
            self.set_duty(2, 0);
            self.enable(2);
        }
        if self.channels.ch3.is_some(){
            self.set_duty(3, 0);
            self.enable(3);
        }
        if self.channels.ch4.is_some(){
            self.set_duty(4, 0);
            self.enable(4);
        }
    }

    fn init_timer4(
        &mut self,
        freq: Hertz,
        clk: Hertz,
        ){
        let ticks = clk.0 / freq.0;
        let psc = (ticks / (1 << 16)) as u16;
        let car = (ticks / (psc + 1) as u32) as u16;
        unsafe{
            self.timer.psc.write(|w| w.psc().bits(psc) );
            self.timer.car.write(|w| w.carl().bits(car));

            self.timer.ctl0.write(|w|
                                  w
                                  .cam().bits(0b00) // Edge aligned
                                  .dir().clear_bit() // Count up
                                  .spm().clear_bit() // Don't stop the counter
                                  .cen().set_bit() // Enable the counter
                                 );
        }
        unsafe {
            self.timer.chctl0_output().modify(|_,w|{
                w
                    // This is just going to use PWM mode 1, bits 111 in the CH0COMCTL
                    // It should be good enough for now to get pwm working
                    .ch0comctl().bits(0b111)
            });
        }
        // Timer should now be ticking.

    }

    pub fn disable(&mut self, ch: u8) {
        match ch{
            1 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch0en().clear_bit()
                                        ),
            2 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch1en().clear_bit()
                                        ),
            3 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch2en().clear_bit()
                                        ),
            4 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch3en().clear_bit()
                                        ),
            _ => (),
        }
    }

    pub fn enable(&mut self, ch: u8) {
        match ch{
            1 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch0en().set_bit()
                                        ),
            2 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch0en().set_bit()
                                        ),
            3 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch0en().set_bit()
                                        ),
            4 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch0en().set_bit()
                                        ),
            _ => (),
        }
    }

    pub fn get_duty(&mut self, ch: u8) -> u16 {
        // unsafe { (*$TIMERX::ptr()).ccr1.read().ccr().bits() }
        match ch{
            1 =>
                return (self.timer.ch0cv.read().bits() >> 16) as u16,
            2 =>
                return (self.timer.ch1cv.read().bits() >> 16) as u16,
            3 =>
                return (self.timer.ch2cv.read().bits() >> 16) as u16,
            4 =>
                return (self.timer.ch3cv.read().bits() >> 16) as u16,
            _ => 0,
        }
    }

    pub fn get_max_duty(&mut self, ch: u8) -> u16 {
        self.timer.car.read().bits()
    }

    pub fn set_duty(&mut self, ch: u8, duty: u16) {
        match ch{
            1 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch0en().clear_bit()
                                        ),
            2 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch1en().clear_bit()
                                        ),
            3 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch2en().clear_bit()
                                        ),
            4 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch3en().clear_bit()
                                        ),
            _ => ()
        }
        // unsafe { (*$TIMERX::ptr()).ccr1.write(|w| w.ccr().bits(duty)) }
    }
}
