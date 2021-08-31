use embedded_hal::prelude::_embedded_hal_blocking_delay_DelayUs;

use crate::hal;
use crate::onewire;

const WARM_WATER_SENSOR_ADD: [u8; 8] = [0x28, 0xFF, 0x2C, 0x99, 0x74, 0x16, 0x04, 0xB5];
const BUFFER_BUTTOM_SENSOR_ADD: [u8; 8] = [0x28, 0xFF, 0x2F, 0x96, 0x74, 0x16, 0x04, 0x61];
const BUFFER_TOP_SENSOR_ADD: [u8; 8] = [0x28, 0xFF, 0x4B, 0x96, 0x74, 0x16, 0x04, 0x6F];
const BOILER_SENSOR_ADD: [u8; 8] = [0x28, 0xFF, 0x7B, 0x58, 0x55, 0x16, 0x03, 0x7B];

pub const MIN_BUFFER_TEMPERATURE: i16 = 550; // m°C
pub const BUFFER_HYSTERESIS: i16 = 50; // mK

const _ALARM_TEMP_LOW: i8 = 5;
const _ALARM_TEMP_HIGH: i8 = 95;
const MEASURERESOLUTION: onewire::ds18b20::MeasureResolution =
    onewire::ds18b20::MeasureResolution::Bit09;

/// Temperatures in the plant
#[derive(Default, PartialEq)]
pub struct PlantTemperatures {
    pub warm_water: Option<i16>,
    pub buffer_top: Option<i16>,
    pub buffer_buttom: Option<i16>,
    pub boiler: Option<i16>,
}

pub struct Sensors {
    bus: onewire::OneWire<hal::port::Pin<hal::port::mode::OpenDrain>>,
    warm_water: Option<onewire::DS18B20>,
    buffer_top: Option<onewire::DS18B20>,
    buffer_buttom: Option<onewire::DS18B20>,
    boiler: Option<onewire::DS18B20>,
}

impl Sensors {
    /// Setup function
    pub fn setup(pin: hal::port::Pin<hal::port::mode::OpenDrain>) -> Self {
        // Init the bus
        let mut bus = onewire::OneWire::new(pin);
        let mut delay = hal::delay::Delay::<crate::Clock>::new();

        // Init sensors and set their configuration
        let warm_water = init_sensor(WARM_WATER_SENSOR_ADD, &mut delay, &mut bus);
        let buffer_top = init_sensor(BUFFER_TOP_SENSOR_ADD, &mut delay, &mut bus);
        let buffer_buttom = init_sensor(BUFFER_BUTTOM_SENSOR_ADD, &mut delay, &mut bus);
        let boiler = init_sensor(BOILER_SENSOR_ADD, &mut delay, &mut bus);

        Sensors {
            bus,
            warm_water,
            buffer_top,
            buffer_buttom,
            boiler,
        }
    }

    // Start Temperature Measurement on all sensors. User has to take care that the conversion time is waited before read

    // Read Temperature for all sensors
    pub fn read_temperatures(&mut self) -> Option<PlantTemperatures> {
        let mut temperatures = PlantTemperatures::default();

        let mut delay = hal::delay::Delay::<super::Clock>::new();

        if onewire::DS18B20::start_measurement(&mut self.bus, &mut delay).is_err() {
            return None;
        };

        delay.delay_us(MEASURERESOLUTION.conversion_time() * 1_000);

        if let Some(sensor) = self.warm_water.as_mut() {
            temperatures.warm_water = sensor.read_temperature(&mut self.bus, &mut delay).ok();
        }
        if let Some(sensor) = self.buffer_buttom.as_mut() {
            temperatures.buffer_buttom = sensor.read_temperature(&mut self.bus, &mut delay).ok();
        }
        if let Some(sensor) = self.buffer_top.as_mut() {
            temperatures.buffer_top = sensor.read_temperature(&mut self.bus, &mut delay).ok();
        }
        if let Some(sensor) = self.boiler.as_mut() {
            temperatures.boiler = sensor.read_temperature(&mut self.bus, &mut delay).ok();
        }

        Some(temperatures)
    }
}

// Create a sensor and set the options
fn init_sensor(
    add: [u8; 8],
    _delay: &mut hal::delay::Delay<crate::Clock>,
    _bus: &mut onewire::OneWire<hal::port::Pin<hal::port::mode::OpenDrain>>,
) -> Option<onewire::DS18B20> {
    onewire::DS18B20::new(add).ok()
}
