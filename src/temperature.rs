use crate::hal;
use crate::onewire;
use hal::prelude::*;

const BUFFER_TOP_SENSOR_ADD: [u8; 8] = [0x28, 0xFF, 0x4B, 0x96, 0x74, 0x16, 0x04, 0x6F];
const BUFFER_BUTTOM_SENSOR_ADD: [u8; 8] = [28, 0xFF, 0x2F, 0x96, 0x74, 0x16, 0x04, 0x61];
const WARM_WATER_SENSOR_ADD: [u8; 8] = [28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const HEAT_FLOW_SENSOR_ADD: [u8; 8] = [28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const HEAT_RETURN_SENSOR_ADD: [u8; 8] = [28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

const _ALARM_TEMP_LOW: i8 = 5;
const _ALARM_TEMP_HIGH: i8 = 95;
//const _PERCISION: onewire::ds18b20::MeasureResolution = onewire::ds18b20::MeasureResolution::TC4;

/// Temperatures in the plant
#[derive(Default)]
pub struct PlantTemperatures {
    pub warm_water: Option<u8>,
    pub buffer_top: Option<u8>,
    pub buffer_buttom: Option<u8>,
    pub heat_flow: Option<u8>,
    pub heat_return: Option<u8>,
}

pub struct Sensors {
    bus: onewire::OneWire<hal::port::Pin<hal::port::mode::TriState>>,
    warm_water: Option<onewire::DS18B20>,
    buffer_top: Option<onewire::DS18B20>,
    buffer_buttom: Option<onewire::DS18B20>,
    heat_flow: Option<onewire::DS18B20>,
    heat_return: Option<onewire::DS18B20>,
}

impl Sensors {
    /// Setup function
    pub fn setup(pin: hal::port::Pin<hal::port::mode::TriState>) -> Self {
        // Init the bus
        let mut bus = onewire::OneWire::new(pin);
        let mut delay = hal::delay::Delay::<crate::Clock>::new();

        // Init sensors and set their configuration
        let warm_water = init_sensor(&WARM_WATER_SENSOR_ADD, &mut delay, &mut bus);
        let buffer_top = init_sensor(&BUFFER_TOP_SENSOR_ADD, &mut delay, &mut bus);
        let buffer_buttom = init_sensor(&BUFFER_BUTTOM_SENSOR_ADD, &mut delay, &mut bus);
        let heat_flow = init_sensor(&HEAT_FLOW_SENSOR_ADD, &mut delay, &mut bus);
        let heat_return = init_sensor(&HEAT_RETURN_SENSOR_ADD, &mut delay, &mut bus);

        Sensors {
            bus,
            warm_water,
            buffer_top,
            buffer_buttom,
            heat_flow,
            heat_return,
        }
    }

    pub fn print_sensors(&mut self, serial: &mut hal::usart::WriteUsart0<crate::Clock>) {
        self.bus
            .search(&mut hal::delay::Delay::<crate::Clock>::new(), serial);
    }

    // Start Temperature Measurement on all sensors. User has to take care that the conversion time is waited before read

    // Read Temperature for all sensors
    pub fn read_temperatures(&mut self) -> Option<PlantTemperatures> {
        Some(PlantTemperatures::default())
    }
}

// Create a sensor and set the options
fn init_sensor(
    add: &[u8; 8],
    _delay: &mut hal::delay::Delay<crate::Clock>,
    _bus: &mut onewire::OneWire<hal::port::Pin<hal::port::mode::TriState>>,
) -> Option<onewire::DS18B20> {
    Some(onewire::DS18B20::new())
}
