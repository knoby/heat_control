//! Definition of Structs for the input and outputs

use atmega328p_hal as hal;

pub struct Outputs {
    pub pump_buffer: hal::port::Pin<hal::port::mode::Output>,
    pub magnet_valve_buffer: hal::port::Pin<hal::port::mode::Output>,
    pub burner_inhibit: hal::port::Pin<hal::port::mode::Output>,
}

pub struct Inputs {
    pub start_burner: hal::port::Pin<hal::port::mode::Input<hal::port::mode::Floating>>,
    pub warm_water_pump: hal::port::Pin<hal::port::mode::Input<hal::port::mode::Floating>>,
    pub heating_pump: hal::port::Pin<hal::port::mode::Input<hal::port::mode::Floating>>,
}
