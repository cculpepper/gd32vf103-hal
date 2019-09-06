use core::marker::PhantomData;

pub trait GpioExt {
    
    type Parts;

    fn split(self) -> Self::Parts;
}

pub struct Locked;

pub struct Unlocked;

pub struct Input<MODE> {
    _typestate_mode: PhantomData<MODE>,
}
pub struct Analog;

pub struct Floating;

pub struct PullDown;

pub struct PullUp;

pub struct Output<MODE> {
    _typestate_mode: PhantomData<MODE>,
}

pub struct Alternate<MODE> {
    _typestate_mode: PhantomData<MODE>,
}

pub struct PushPull;

pub struct OpenDrain;

pub mod gpioa {
    use core::marker::PhantomData;
    use super::{Input, Floating, OpenDrain, Output, Unlocked};
    use crate::pac::{GPIOA, gpioa};

    pub struct Parts {
        pub ctl0: CTL0,
        //ctl1
        pub pa0: PA0<Unlocked, Input<Floating>>,
        //pa1, ..
    }

    pub struct CTL0 {
        _private: ()
    }

    impl CTL0 {
        pub(crate) fn ctl0(&mut self) -> &gpioa::CTL0 {
            unsafe { &(*GPIOA::ptr()).ctl0 }
        }
    }

    pub struct PA0<LOCKED, MODE> {
        _typestate_locked: PhantomData<LOCKED>,
        _typestate_mode: PhantomData<MODE>,
    }

    impl<MODE> PA0<Unlocked, MODE> {
        pub fn into_open_drain_output(self, ctl0: &mut CTL0) -> PA0<Unlocked, Output<OpenDrain>> {
            let offset = 0;
            let ctl_mode = 0b0101;
            //todo: ATOMIC OPERATIONS
            ctl0.ctl0().modify(|r, w| unsafe {
                w.bits((r.bits() & !(0b1111 << offset)) | (ctl_mode << offset))
            });
            PA0 { _typestate_locked: PhantomData, _typestate_mode: PhantomData }
        }
    }
}
