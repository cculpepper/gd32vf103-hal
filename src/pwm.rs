use crate::pac::TIMER4;
use crate::rcu::{Clocks, APB1, APB2};
use crate::gpio::{Alternate, PushPull};
use crate::gpio::gpioa::*;


use crate::time::Hertz;

pub struct Channels {
    ch0: Option<PA0<Alternate<PushPull>>>,
    ch1: Option<PA1<Alternate<PushPull>>>,
    ch2: Option<PA2<Alternate<PushPull>>>,
    ch3: Option<PA3<Alternate<PushPull>>>,
}


pub struct Pwm {
    pub timer: TIMER4,
    pub frequency: Hertz,
    pub channels: Channels,
    pub clock_freq: Hertz,
}

impl Pwm{

    pub fn pwm(
        freq: Hertz,
        clock: Clocks,
        timer: TIMER4,
        apb1: &mut APB1,
        apb2: &mut APB2, // Only need this to enable the afios...
        ch0: Option<PA0<Alternate<PushPull>>>,
        ch1: Option<PA1<Alternate<PushPull>>>,
        ch2: Option<PA2<Alternate<PushPull>>>,
        ch3: Option<PA3<Alternate<PushPull>>>
        ) -> Self {
        // Honestly, this is probably not a good place to put this, but w.e.
        apb1.en().modify(|_,w| w.timer4en().set_bit());
        // apb1.rst().modify(|_,w| w.timer4rst().set_bit());
        // apb1.rst().modify(|_,w| w.timer4rst().clear_bit());
        // Just going to try to do it all in one step here.
        //
        // 1. Config clock
        // 2. Set shadow enable mode
        // Set active high polarity
        // Enable output with CHxEN
        // Compare output timing config with the CAR and CHxCV reg
        // Start by setting CEN to 1
        // OxCPRE to togglo?
        // Set CHxCOMCTL to 0x110
        // Need to be put into edge-aligned PWM
        // Period is determined by CAR, duty cycle by CHxCV
        // Register by register
        //  CTL0:
        //  CKDIV : 0 clock is TIMER_CK
        //  ARSE : 1 Auto reload shadow enable, 
        //  CAM : 00 Edge aligned mode
        //  DIR : 0 Count up
        //  SPM : 0 Counter continues
        //  UPS : X don't need interrupts
        //  UPDIS : X Don't need interrupts
        //  CEN : 1, only when finished
        // CTL1:
        //  TI0S : X Don't care
        //  MMC: XXX Don't care
        //  DMAS: X Don't care
        // CHCTL0
        //  CH0COMCEN: 0 Don't clear the PRE signal when high is detected on ETIF
        //  CH0COMCTL: 011 Toggle on match? or 110 for PWM Mode 0
        //  CH0COMSEN: 1 enable output shadow
        // CHCTL1
        //  See CHCTL1
        // CHCTL2
        //  CHxP Set to 1 for active high
        //  CHxEN: 1 Enable the output
        // PSC: Prescaler
        // CAR: Counter auto reload register
        //  Auto reload value of the counter
        // CHxCV: Compare value
        // j
        //
        //
        Pwm{
            timer : timer,
            frequency: freq,
            // channels : Channels{ch2},
            channels : Channels{ch0, ch1, ch2, ch3},
            clock_freq: clock.ck_apb1(),
        }

    }
    pub fn init(&mut self){

        self.init_timer4(self.frequency.into(), self.clock_freq);

        if self.channels.ch0.is_some(){
            // We're assuming the GPIOs are already in alternate mode here.
            self.disable(0);
            self.enable(0);
            self.set_duty(0, 0);
            // self.timer.chctl2.modify(|_,w| w.ch0p().clear_bit());
            // self.timer.chctl2.modify(|_,w| w.ch0p().set_bit());
            unsafe{
                self.timer.chctl0_output().modify(|_,w| w.ch0ms().bits(0b00));
                self.timer.chctl0_output().modify(|_,w| w.ch0comctl().bits(0b110));
                self.timer.chctl0_output().modify(|_,w| w.ch0comsen().clear_bit());
            }
        }
        if self.channels.ch1.is_some(){
            self.disable(1);
            self.enable(1);
            self.set_duty(1, 0);
            // self.timer.chctl2.modify(|_,w| w.ch1p().clear_bit());
            // self.timer.chctl2.modify(|_,w| w.ch1p().set_bit());
            unsafe{
                self.timer.chctl0_output().modify(|_,w| w.ch1ms().bits(0b00));
                self.timer.chctl0_output().modify(|_,w| w.ch1comctl().bits(0b110));
                self.timer.chctl0_output().modify(|_,w| w.ch1comsen().clear_bit());
            }
        }
        if self.channels.ch2.is_some(){
            self.disable(2);
            self.enable(2);
            self.set_duty(2, 0);
            // self.timer.chctl2.modify(|_,w| w.ch2p().clear_bit());
            // self.timer.chctl2.modify(|_,w| w.ch2p().set_bit());
            unsafe{
                self.timer.chctl1_output().modify(|_,w| w.ch2ms().bits(0b00));
                self.timer.chctl1_output().modify(|_,w| w.ch2comctl().bits(0b110));
                self.timer.chctl1_output().modify(|_,w| w.ch2comsen().clear_bit());
            }
        }
        if self.channels.ch3.is_some(){
            self.disable(3);
            self.enable(3);
            self.set_duty(3, 0);
            // self.timer.chctl2.modify(|_,w| w.ch3p().clear_bit());
            // self.timer.chctl2.modify(|_,w| w.ch3p().set_bit());
            unsafe{
                self.timer.chctl1_output().modify(|_,w| w.ch3ms().bits(0b00));
                self.timer.chctl1_output().modify(|_,w| w.ch3comctl().bits(0b110));
                self.timer.chctl1_output().modify(|_,w| w.ch3comsen().clear_bit());
            }
        }
        self.timer.ctl0.write(|w| w.arse().set_bit()); // Auto reload the shadow register
        self.timer.ctl0.write(|w|
                              w
                              .cen().set_bit() // Enable the counter
                             );
    }

    fn init_timer4(
        &mut self,
        freq: Hertz,
        clk: Hertz,
        ){
        let ticks = clk.0 / freq.0;
        let psc = 10;//(ticks / (1 << 16)) as u16;
        let car = 15999;//(ticks / (psc + 1) as u32) as u16;
        unsafe{
            self.timer.psc.write(|w| w.psc().bits(10));

            self.timer.ctl0.write(|w| w
                                  .cam().bits(0b00) // Count up
                                  .dir().clear_bit() // 
                                  .ckdiv().bits(0b00)
                                 );
            self.timer.car.write(|w| w.carl().bits(15999));

            self.timer.swevg.write(|w| w.upg().set_bit());


        }
        // Timer should now be ticking.

    }

    pub fn disable(&mut self, ch: u8) {
        match ch{
            0 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch0en().clear_bit()
                                        ),
            1 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch1en().clear_bit()
                                        ),
            2 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch2en().clear_bit()
                                        ),
            3 =>
                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch3en().clear_bit()
                                        ),
            _ => (),
        }
    }

    pub fn enable(&mut self, ch: u8) {
        match ch{
            0 => {
                self.timer.chctl2.modify(|_,w|
                                         w .ch0en().clear_bit());

                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch0en().set_bit()
                                         .ch0p().set_bit()
                                        );
                unsafe{
                    self.timer.chctl0_output().modify(|_,w|
                        w.ch0ms().bits(0b00));
                }
            },
            1 => {
                self.timer.chctl2.modify(|_,w|
                                         w .ch1en().clear_bit());

                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch1en().set_bit()
                                         .ch1p().set_bit()
                                        );
                unsafe{
                    self.timer.chctl0_output().modify(|_,w|
                        w.ch1ms().bits(0b00));
                }
            },
            2 => {
                self.timer.chctl2.modify(|_,w|
                                         w .ch2en().clear_bit());

                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch2en().set_bit()
                                         .ch2p().set_bit()
                                        );
                unsafe{
                    self.timer.chctl1_output().modify(|_,w|
                        w.ch2ms().bits(0b00));
                }
            },
            3 => {
                self.timer.chctl2.modify(|_,w|
                                         w .ch3en().clear_bit());

                self.timer.chctl2.modify(|_,w|
                                         w
                                         .ch3en().set_bit()
                                         .ch3p().set_bit()
                                        );
                unsafe{
                    self.timer.chctl1_output().modify(|_,w|
                        w.ch3ms().bits(0b00));
                }
            },
            _ => (),
        }
    }

    pub fn get_duty(&mut self, ch: u8) -> u16 {
        // unsafe { (*$TIMERX::ptr()).ccr1.read().ccr().bits() }
        match ch{
            0 =>
                // For some reason, these are implemented as 32 bit, not 16 bit.
                return (self.timer.ch0cv.read().ch0val().bits()),
            1 =>
                return (self.timer.ch1cv.read().ch1val().bits()),
            2 =>
                return (self.timer.ch2cv.read().ch2val().bits()),
            3 =>
                return (self.timer.ch3cv.read().ch3val().bits()),
            _ => 0,
        }
    }

    pub fn get_max_duty(&mut self) -> u16 {
        self.timer.car.read().bits()
    }

    pub fn set_duty(&mut self, ch: u8, duty: u16) {
        match ch{
            0 =>
                // For some reason, these are implemented as 32 bit, not 16 bit.
                // No, I'm an idiot, and just missed the value access register. 
                // I assumed I could access the entire thing, but there's an 
                // access function
                unsafe{
                    self.timer.ch0cv.write(|w| w.ch0val().bits(duty));
                }
            1 =>
                unsafe{
                    self.timer.ch1cv.write(|w| w.ch1val().bits(duty));
                }
            2 =>
                unsafe{
                    self.timer.ch2cv.write(|w| w.ch2val().bits(duty));
                }
            3 =>
                unsafe{
                    self.timer.ch3cv.write(|w| w.ch3val().bits(duty));
                }
            _ => {},
        }
    }
}
