use crate::pac::TIMER4;
use crate::rcu::{Clocks};

use crate::time::Hertz;
// use crate::bb;
use embedded_hal;


fn init_pwm_4(
    freq: Hertz,
    clock: Clocks,
    timer: TIMER4,
    ch1: bool,
    ch2: bool,
    ch3: bool,
    ch4: bool
    ){
    init_timer4(&timer, freq.into(), clock.ck_apb1());
    if ch1{
        // We're assuming the GPIOs are already in alternate mode here.
        set_duty(&timer, 1, 0);
        enable(&timer, 1);
    }
    if ch2{
        set_duty(&timer, 2, 0);
        enable(&timer, 2);
    }
    if ch3{
        set_duty(&timer, 3, 0);
        enable(&timer, 3);
    }
    if ch4{
        set_duty(&timer, 4, 0);
        enable(&timer, 4);
    }
}

fn init_timer4(
    timer: &TIMER4,
    freq: Hertz,
    clk: Hertz,
    ){
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
    }
    // Timer should now be ticking.

}

pub fn setup(timer: &TIMER4, ch: u8){
    unsafe {
        timer.chctl0_output().modify(|_,w|{
            w
                // This is just going to use PWM mode 1, bits 111 in the CH0COMCTL
                // It should be good enough for now to get pwm working
                .ch0comctl().bits(0b111)
        });
    }
}
pub fn disable(timer: &TIMER4, ch: u8) {
    match ch{
        1 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch0en().clear_bit()
                               ),
        2 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch1en().clear_bit()
                               ),
        3 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch2en().clear_bit()
                               ),
        4 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch3en().clear_bit()
                               ),
        _ => (),
    }
}

pub fn enable(timer: &TIMER4, ch: u8) {
    match ch{
        1 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch0en().set_bit()
                               ),
        2 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch0en().set_bit()
                               ),
        3 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch0en().set_bit()
                               ),
        4 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch0en().set_bit()
                               ),
        _ => (),
    }
}

pub fn get_duty(timer: &TIMER4, ch: u8) -> u16 {
    // unsafe { (*$TIMERX::ptr()).ccr1.read().ccr().bits() }
    match ch{
        1 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch0en().clear_bit()
                               ),
        2 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch1en().clear_bit()
                               ),
        3 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch2en().clear_bit()
                               ),
        4 =>
            timer.chctl2.modify(|_,w|
                                w
                                .ch3en().clear_bit()
                               ),
        _ => ()
    }
    10
}

pub fn get_max_duty(timer: &TIMER4, ch: u8) -> u16 {
    // unsafe { (*$TIMERX::ptr()).arr.read().arr().bits() }
    10
}

pub fn set_duty(timer: &TIMER4, ch: u8, duty: u16) {
    // unsafe { (*$TIMERX::ptr()).ccr1.write(|w| w.ccr().bits(duty)) }
}

