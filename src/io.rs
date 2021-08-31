//! Definition of Structs for the input and outputs
use crate::hal;

type OutputPin = hal::port::Pin<hal::port::mode::Output>;
type InputPin = hal::port::Pin<hal::port::mode::Input<hal::port::mode::Floating>>;

pub struct Outputs {
    pump_buffer_pin: OutputPin,
    pump_buffer: bool,
    magnet_valve_buffer_pin: OutputPin,
    magnet_valve_buffer: bool,
    burner_inhibit_pin: OutputPin,
    burner_inhibit: bool,
}

/// Impl PartEq only for the logical state
impl PartialEq for Outputs {
    fn eq(&self, other: &Self) -> bool {
        (self.pump_buffer == other.pump_buffer)
            && (self.magnet_valve_buffer == other.magnet_valve_buffer)
            && (self.burner_inhibit == other.burner_inhibit)
    }
}

#[allow(dead_code)]
impl Outputs {
    pub fn new(
        burner_inhibit_pin: OutputPin,
        pump_buffer_pin: OutputPin,
        magnet_valve_buffer_pin: OutputPin,
    ) -> Self {
        Self {
            burner_inhibit: false,
            burner_inhibit_pin,
            pump_buffer: false,
            pump_buffer_pin,
            magnet_valve_buffer: false,
            magnet_valve_buffer_pin,
        }
    }

    /// Set the physical outputs according to the setvalue
    pub fn set_outputs(&mut self) {
        if self.burner_inhibit {
            self.burner_inhibit_pin.set_high();
        } else {
            self.burner_inhibit_pin.set_low();
        }

        if self.magnet_valve_buffer {
            self.magnet_valve_buffer_pin.set_high();
        } else {
            self.magnet_valve_buffer_pin.set_low();
        }

        if self.pump_buffer {
            self.pump_buffer_pin.set_high();
        } else {
            self.pump_buffer_pin.set_low();
        }
    }

    pub fn set_magnet_valve_buffer(&mut self, value: bool) {
        self.magnet_valve_buffer = value
    }

    pub fn set_burner_inhibit(&mut self, value: bool) {
        self.burner_inhibit = value;
    }

    pub fn set_pump_buffer(&mut self, value: bool) {
        self.pump_buffer = value
    }

    pub fn get_magnet_valve_buffer(&self) -> bool {
        self.magnet_valve_buffer
    }

    pub fn get_burner_inhibit(&self) -> bool {
        self.burner_inhibit
    }

    pub fn get_pump_buffer(&self) -> bool {
        self.pump_buffer
    }
}

pub struct Inputs {
    pub start_burner: bool,
    start_burner_pin: InputPin,
    warm_water_pump: bool,
    warm_water_pump_pin: InputPin,
    heating_pump: bool,
    heating_pump_pin: InputPin,
}

#[allow(dead_code)]
impl Inputs {
    pub fn new(
        start_burner_pin: InputPin,
        warm_water_pump_pin: InputPin,
        heating_pump_pin: InputPin,
    ) -> Self {
        Self {
            start_burner: false,
            start_burner_pin,
            warm_water_pump: false,
            warm_water_pump_pin,
            heating_pump: false,
            heating_pump_pin,
        }
    }

    pub fn get_inputs(&mut self) {
        self.start_burner = self.start_burner_pin.is_high();
        self.warm_water_pump = self.warm_water_pump_pin.is_high();
        self.heating_pump = self.heating_pump_pin.is_high();
    }

    pub fn get_start_burner(&self) -> bool {
        self.start_burner
    }

    pub fn get_warm_water_pump(&self) -> bool {
        self.warm_water_pump
    }

    pub fn get_heating_pump(&self) -> bool {
        self.heating_pump
    }
}

impl PartialEq for Inputs {
    fn eq(&self, other: &Self) -> bool {
        (self.start_burner == other.start_burner)
            && (self.warm_water_pump == other.warm_water_pump)
            && (self.heating_pump == other.heating_pump)
    }
}
