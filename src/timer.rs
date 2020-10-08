//! Basic implementation of a timer to be able to calculate time in a state

use atmega328p_hal::atmega328p as chip;

/// Holds the time since the application is running
/// Is updated in an interrupt. A safe read to this is possibly in an interrupt free function
static mut TIME: u32 = 0;

pub struct Timer1 {}

impl Timer1 {
    pub fn new(tc1: chip::TC1) -> Self {
        // Set the Clock Source
        tc1.tccr1b.write(|w| w.cs1().prescale_64());

        // Config timer to count to specific value and than reset
        tc1.tccr1a.write(|w| w.wgm1().bits(0b00));
        tc1.tccr1b.modify(|_, w| w.wgm1().bits(0b01));

        // Set the Output Compare Register to 250_000= 16_000_000/64 => 1s => 250 = 1s
        tc1.ocr1a.write(|w| unsafe { w.bits(250) });

        // Enable Interrupt on Output Compare Match
        tc1.timsk1.write(|w| w.ocie1a().set_bit());

        Self {}
    }

    pub fn get_time(&self) -> u32 {
        let mut my_time = 0;
        unsafe { avr_device::interrupt::free(|_| my_time = TIME) };
        my_time
    }
}

#[avr_device::interrupt(atmega328p)]
unsafe fn TIMER1_COMPA() {
    TIME += 1;
}
