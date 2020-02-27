/*!
  # Pulse width modulation

  The general purpose timers (`TIMER2`, `TIMER3`, and `TIMER4`) can be used to output
  pulse width modulated signals on some pins. The timers support up to 4
  simultaneous pwm outputs in separate `Channels`

  ## Usage for pre-defined channel combinations

  This crate only defines basic channel combinations for default AFIO remappings,
  where all the channels are enabled. Start by setting all the pins for the
  timer you want to use to alternate push pull pins:

  ```rust
  let gpioa = ..; // Set up and split GPIOA
// Select the pins you want to use
let pins = (
gpioa.pa0.into_alternate_push_pull(&mut gpioa.crl),
gpioa.pa1.into_alternate_push_pull(&mut gpioa.crl),
gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl),
gpioa.pa3.into_alternate_push_pull(&mut gpioa.crl),
);

// Set up the timer as a PWM output. If selected pins may correspond to different remap options,
// then you must specify the remap generic parameter. Otherwise, if there is no such ambiguity,
// the remap generic parameter can be omitted without complains from the compiler.
let (c1, c2, c3, c4) = Timer::timer2(p.TIMER2, &clocks, &mut rcc.apb1)
.pwm::<Tim2NoRemap, _, _, _>(pins, &mut afio.mapr, 1.khz())
.3;

// Start using the channels
c1.set_duty(c1.get_max_duty());
// ...
```

Then call the `pwm` function on the corresponding timer.

NOTE: In some cases you need to specify remap you need, especially for TIMER2
(see [Alternate function remapping](super::timer)):

```
let device: pac::Peripherals = ..;

// Put the timer in PWM mode using the specified pins
// with a frequency of 100 hz.
let (c0, c1, c2, c3) = Timer::timer2(device.TIMER2, &clocks, &mut rcc.apb1)
.pwm::<Tim2NoRemap, _, _, _>(pins, &mut afio.mapr, 100.hz());

// Set the duty cycle of channel 0 to 50%
c0.set_duty(c0.get_max_duty() / 2);
// PWM outputs are disabled by default
c0.enable()
```
*/


use crate::pac::TIMER4;
use crate::pac::AFIO;
use crate::rcu::{Clocks, APB1};
use crate::timer::Timer;

use crate::gpio::{self, Alternate, PushPull};
use crate::time::Hertz;
// use crate::bb;
use embedded_hal;

use core::mem;


pub trait Pins<REMAP, P> {
    const C1: bool = false;
    const C2: bool = false;
    const C3: bool = false;
    const C4: bool = false;
    type Channels;
}

use crate::timer::sealed::{Remap, Ch1, Ch2, Ch3, Ch4};

    impl <TIMER, REMAP, P1, P2, P3, P4> Pins<REMAP, (C1, C2, C3, C4)> for
     (P1, P2, P3, P4) where REMAP: Remap<Periph = TIMER>, 
     P1: Ch1<REMAP> + gpio::Mode<Alternate<PushPull>>, 
     P2: Ch2<REMAP> + gpio::Mode<Alternate<PushPull>>,
     P3: Ch3<REMAP> + gpio::Mode<Alternate<PushPull>>, 
     P4: Ch4<REMAP> + gpio::Mode<Alternate<PushPull>> {
        const C1: bool = true;
        const C2: bool = true;
        const C3: bool = true;
        const C4: bool = true;
        type
        Channels = (Pwm<TIMER, C1>, Pwm<TIMER, C2>, Pwm<TIMER, C3>, Pwm<TIMER, C4>);
    }

impl Timer<TIMER4> {
    pub fn pwm<REMAP, P, PINS, T>(
        self,
        pins: PINS,
        freq: T,
        clock: Clocks,
        ) -> PINS::Channels
        where
        REMAP: Remap<Periph = TIMER4>,
        PINS: Pins<REMAP, P>,
        T: Into<Hertz>,
        {
            // mapr.modify_mapr(|_, w| w.timer4_remap().bit(REMAP::REMAP == 1));

            let Self { timer, clock_scaler, clock_frequency } = self;
            // Frequency going into the timer is CK_TIMER4
// This should be timer4, pins, pwm preq and clock frequency
            timer4(timer, pins, freq.into(), clock.ck_apb1())
        }
}

pub struct Pwm<TIMER, CHANNEL> {
    channel: CHANNEL,
    timer: TIMER,
}

pub struct C1;
pub struct C2;
pub struct C3;
pub struct C4;

macro_rules! hal {
    ($($TIMERX:ident: ($timerX:ident),)+) => {
        $(
            fn $timerX<REMAP, P, PINS>(
                timer: $TIMERX,
                _pins: PINS,
                freq: Hertz,
                clk: Hertz,
                ) -> PINS::Channels
            where
            REMAP: Remap<Periph = $TIMERX>,
            PINS: Pins<REMAP, P>,
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
            impl embedded_hal::PwmPin for Pwm<$TIMERX, C1> {
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
        )+
    }
}

hal! {
    TIMER4: (timer4),
}
