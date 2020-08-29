use crate::hal;
use crate::onewire;
use hal::prelude::*;

const BUFFER_TOP_SENSOR_ADD: [u8; 8] = [0x28, 0xFF, 0x4B, 0x96, 0x74, 0x16, 0x04, 0x6F];
const BUFFER_BUTTOM_SENSOR_ADD: [u8; 8] = [0x28, 0xFF, 0x2F, 0x96, 0x74, 0x16, 0x04, 0x61];
const WARM_WATER_SENSOR_ADD: [u8; 8] = [0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const HEAT_FLOW_SENSOR_ADD: [u8; 8] = [40, 255, 65, 130, 88, 22, 4, 203];
const HEAT_RETURN_SENSOR_ADD: [u8; 8] = [40, 255, 123, 88, 85, 22, 3, 123];

const _ALARM_TEMP_LOW: i8 = 5;
const _ALARM_TEMP_HIGH: i8 = 95;
const MEASURERESOLUTION: onewire::ds18b20::MeasureResolution =
    onewire::ds18b20::MeasureResolution::Bit09;

/// Temperatures in the plant
#[derive(Default)]
pub struct PlantTemperatures {
    pub warm_water: Option<i16>,
    pub buffer_top: Option<i16>,
    pub buffer_buttom: Option<i16>,
    pub heat_flow: Option<i16>,
    pub heat_return: Option<i16>,
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
        let warm_water = init_sensor(WARM_WATER_SENSOR_ADD, &mut delay, &mut bus);
        let buffer_top = init_sensor(BUFFER_TOP_SENSOR_ADD, &mut delay, &mut bus);
        let buffer_buttom = init_sensor(BUFFER_BUTTOM_SENSOR_ADD, &mut delay, &mut bus);
        let heat_flow = init_sensor(HEAT_FLOW_SENSOR_ADD, &mut delay, &mut bus);
        let heat_return = init_sensor(HEAT_RETURN_SENSOR_ADD, &mut delay, &mut bus);

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
        // Create the search state
        let mut search_state = onewire::SearchState::new();

        serial
            .write_str("Searchsing OneWire Bus for devices\n")
            .ok();

        loop {
            match self.bus.search(
                &mut search_state,
                &mut hal::delay::Delay::<crate::Clock>::new(),
            ) {
                Ok(Some(rom_no)) => {
                    serial.write_str("Found Device: ").ok();
                    let mut buffer = num_format::Buffer::default();
                    for byte in rom_no.iter() {
                        buffer.write_formatted(byte, &num_format::Locale::de);
                        serial.write_str(buffer.as_str()).ok();
                        serial.write_char(' ').ok();
                    }
                    serial.write_char('\n').ok();
                }
                Ok(None) => {
                    serial.write_str("Nothing found").ok();
                    break;
                }
                Err(e) => {
                    if e == onewire::Error::SearchEnd {
                        serial.write_str("No more devices found\n").ok();
                    } else {
                        serial.write_str("An Error Occured").ok();
                    };
                    break;
                }
            }
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

        delay.delay_ms(MEASURERESOLUTION.conversion_time());

        if let Some(sensor) = self.warm_water.as_mut() {
            temperatures.warm_water = sensor.read_temperature(&mut self.bus, &mut delay).ok();
        }
        if let Some(sensor) = self.buffer_buttom.as_mut() {
            temperatures.buffer_buttom = sensor.read_temperature(&mut self.bus, &mut delay).ok();
        }
        if let Some(sensor) = self.buffer_top.as_mut() {
            temperatures.buffer_top = sensor.read_temperature(&mut self.bus, &mut delay).ok();
        }
        if let Some(sensor) = self.heat_flow.as_mut() {
            temperatures.heat_flow = sensor.read_temperature(&mut self.bus, &mut delay).ok();
        }
        if let Some(sensor) = self.heat_return.as_mut() {
            temperatures.heat_return = sensor.read_temperature(&mut self.bus, &mut delay).ok();
        }

        Some(temperatures)
    }
}

// Create a sensor and set the options
fn init_sensor(
    add: [u8; 8],
    _delay: &mut hal::delay::Delay<crate::Clock>,
    _bus: &mut onewire::OneWire<hal::port::Pin<hal::port::mode::TriState>>,
) -> Option<onewire::DS18B20> {
    onewire::DS18B20::new(add).ok()
}
