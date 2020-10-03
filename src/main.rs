//!
//! ## Configuration of the HW
//!
//! - 0  - PD0 - RXD               - USB Serial
//! - 1  - PD1 - TXD               - USB Serial
//! - 2  - PD2 - IN0               - One Wire Bus Data (DS1820)
//! - 3  - PD3 - IN1
//! - 4  - PD4 - D0 - Relais/DO    - Pump Buffer
//! - 5  - PD5 - D1 - Relais/DO    - Magnetvalve Buffer
//! - 6  - PD6 - D2 - Relais/DO    - Inhibit Burner
//! - 7  - PD7 - D3 - Relais/DO
//! - 8  - PB0 - D4 - Relais/DO
//! - 9  - PB1 - D5 - Relais/DO
//! - 10 - PB2 - RTC SS            - SPI RTC
//! - 11 - PB3 - MOSI              - SPI RTC
//! - 12 - PB4 - MISO              - SPI RTC
//! - 13 - PB5 - SCK               - SPI RTC
//! - A0 - PC0 - A0 - AI/DI        - Pump warm Water
//! - A1 - PC1 - A1 - AI/DI        - Pump Heating
//! - A2 - PC2 - A2 - AI/DI        - Temperature Contact Wood
//! - A3 - PC3 - A3 - AI/DI        - Request Burner
//! - A4 - PC4 - D6/SDA - DO       - I2C
//! - A5 - PC5 - D7/SCL - DO       - I2C
//! - A6 - ADC6 - A4 - AI
//! - A7 - ADC7 - A5 - AI
//!
//! ## Temperature Sensors
//! - WARMWATER {0x28, 0xff, 0x4b, 0x96, 0x74, 0x16, 0x04, 0x6f}
//! - BUFFERTOP {0x28, 0xFF, 0x4B, 0x96, 0x74, 0x16, 0x04, 0x6F}
//! - BUFFERBOTTOM {0x28, 0xFF, 0x2F, 0x96, 0x74, 0x16, 0x04, 0x61}
//! - BOILER {0x28, 0xFF, 0x2C, 0x99, 0x74, 0x16, 0x04, 0xB5}

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(llvm_asm)]

// Pull in the panic handler from panic-halt
extern crate avr_std_stub;

use atmega328p_hal as hal;
use atmega328p_hal::atmega328p as chip;
use hal::prelude::*;

use ufmt::{uwrite, uwriteln};

type Clock = hal::clock::MHz16;

const DISPLAY_ADD_I2C: u8 = 0x27;

mod io;
mod onewire;
mod temperature;
mod timer;

type Display = hd44780_driver::HD44780<
    hd44780_driver::bus::I2CBus<
        hal::i2c::I2c<Clock, hal::port::mode::Input<hal::port::mode::PullUp>>,
    >,
>;

#[derive(PartialEq, Copy, Clone)]
enum State {
    Init,
    BufferOff,
    BufferOn,
    ActivatePump,
}

impl ufmt::uDisplay for State {
    fn fmt<W>(&self, f: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        ufmt::uwrite!(f, "{}", self.to_string())
    }
}

impl State {
    fn to_string(&self) -> &'static str {
        match self {
            State::Init => "Init",
            State::BufferOff => "Buffer  Off",
            State::BufferOn => "Buffer On",
            State::ActivatePump => "Activate Pump",
        }
    }
}

fn setup() -> (
    io::Inputs,
    io::Outputs,
    hal::usart::WriteUsart0<Clock>,
    Display,
    temperature::Sensors,
    timer::Timer1,
) {
    // Get Peripherals for configuration
    let peripherals = chip::Peripherals::take().unwrap();

    // Get the ports
    let _portb = peripherals.PORTB.split();
    let portc = peripherals.PORTC.split();
    let portd = peripherals.PORTD.split();

    // ------------------
    // Serial Port
    // ------------------
    let rx = portd.pd0.into_floating_input(&portd.ddr);
    let tx = portd.pd1.into_output(&portd.ddr);
    let (_, mut serial) = hal::usart::Usart0::<Clock, hal::port::mode::Floating>::new(
        peripherals.USART0,
        rx,
        tx,
        9600,
    )
    .split();

    uwriteln!(serial, "Heat Control").ok();
    uwriteln!(serial, "Start Initialization ...").ok();

    // ------------------
    // I2C Display
    // ------------------
    uwriteln!(serial, "Initializing I2C LCD").ok();
    let sda = portc.pc4.into_pull_up_input(&portc.ddr);
    let scl = portc.pc5.into_pull_up_input(&portc.ddr);

    // Delay for display
    let mut delay = hal::delay::Delay::<Clock>::new();
    // Create the i2c bus
    let i2c = hal::i2c::I2c::<Clock, _>::new(peripherals.TWI, sda, scl, 400_000);
    // Create the display
    let mut display = hd44780_driver::HD44780::new_i2c(i2c, DISPLAY_ADD_I2C, &mut delay).unwrap();
    uwriteln!(serial, "I2C LCD Init Done").ok();

    display.reset(&mut delay).unwrap();
    display.clear(&mut delay).unwrap();
    display
        .set_display_mode(
            hd44780_driver::DisplayMode {
                cursor_blink: hd44780_driver::CursorBlink::Off,
                cursor_visibility: hd44780_driver::Cursor::Invisible,
                display: hd44780_driver::Display::On,
            },
            &mut delay,
        )
        .unwrap();

    display.write_str("Heat Control", &mut delay).ok();

    // ------------------
    // Init the Sensors
    // ------------------
    uwriteln!(serial, "Initialize OneWire DS18b20 Sensors").ok();
    // Split the pin for one wire
    let pd2 = portd.pd2.into_tri_state(&portd.ddr);

    // Setup the onewire bus
    let temperature_sensors = temperature::Sensors::setup(pd2.downgrade());

    // ------------------
    // digital IOs
    // ------------------
    uwriteln!(serial, "Initialize digital IOs").ok();
    let inputs = io::Inputs::new(
        portc.pc3.into_floating_input(&portc.ddr).downgrade(),
        portc.pc0.into_floating_input(&portc.ddr).downgrade(),
        portc.pc1.into_floating_input(&portc.ddr).downgrade(),
    );

    let outputs = io::Outputs::new(
        portd.pd6.into_output(&portd.ddr).downgrade(),
        portd.pd4.into_output(&portd.ddr).downgrade(),
        portd.pd5.into_output(&portd.ddr).downgrade(),
    );

    // ------------------
    // TIMER
    // ------------------
    let timer1 = timer::Timer1::new(peripherals.TC1);

    // Enable Interrupts
    unsafe {
        avr_device::interrupt::enable();
    }

    uwriteln!(serial, "Initialization Done!").ok();
    (
        inputs,
        outputs,
        serial,
        display,
        temperature_sensors,
        timer1,
    )
}

#[hal::entry]
fn main() -> ! {
    // Init the hardware
    let (mut inputs, mut outputs, mut serial, mut display, mut sensors, timer1) = setup();

    // Output the sensors on the bus to serial
    sensors.print_sensors(&mut serial);

    // Main Application Loop with state machine
    let mut state = State::Init;
    let mut old_state = State::Init;
    let mut last_state = State::Init;
    let mut time_state_start = 0_u32; // Seconds since start
    let mut time_since_send = 0_u32; // Seconds since last send of current state

    let mut delay = hal::delay::Delay::<Clock>::new();

    loop {
        // Get Input state
        inputs.get_inputs();

        // Handle Error Case
        let temperature = sensors.read_temperatures().unwrap_or_else(|| {
            state = State::Init;
            temperature::PlantTemperatures::default()
        });

        // Handle the statemachine
        let new = state != old_state;
        if new {
            time_state_start = timer1.get_time();
            last_state = state;
            uwriteln!(serial, "New State: {}", state).ok();
        };
        old_state = state;

        let time_in_state = timer1.get_time().wrapping_sub(time_state_start);

        match state {
            State::Init => {
                if new {
                    outputs.set_burner_inhibit(false);
                    outputs.set_magnet_valve_buffer(false);
                    outputs.set_pump_buffer(false);
                };
                if time_in_state > 10 && temperature.buffer_top.is_some() {
                    state = State::BufferOff;
                }
            }
            State::BufferOff => {
                if new {
                    outputs.set_burner_inhibit(false);
                    outputs.set_magnet_valve_buffer(false);
                    outputs.set_pump_buffer(false);
                };
                // Switch off
                if temperature.buffer_top.is_none() {
                    state = State::Init;
                } else if (temperature.buffer_top.unwrap()
                    > (temperature::MIN_BUFFER_TEMPERATURE + temperature::BUFFER_HYSTERESIS))
                    && !inputs.get_warm_water_pump()
                    && !inputs.get_start_burner()
                {
                    state = State::BufferOn;
                }
            }
            State::BufferOn => {
                if new {
                    outputs.set_burner_inhibit(true);
                    outputs.set_magnet_valve_buffer(true);
                    outputs.set_pump_buffer(false);
                };
                if temperature.buffer_top.is_none() {
                    state = State::Init;
                } else if temperature.buffer_top.unwrap() < temperature::MIN_BUFFER_TEMPERATURE {
                    state = State::BufferOff;
                } else if inputs.get_start_burner() {
                    // Check if burner is still on
                    if last_state != State::ActivatePump || time_in_state > 60 {
                        state = State::ActivatePump;
                    }
                }
            }
            State::ActivatePump => {
                if new {
                    outputs.set_burner_inhibit(true);
                    outputs.set_magnet_valve_buffer(true);
                    outputs.set_pump_buffer(true);
                };
                if temperature.buffer_top.is_none() {
                    state = State::Init;
                } else if time_in_state > 60 {
                    state = State::ActivatePump;
                }
            }
        }

        // Update Ouptuts
        outputs.set_outputs();

        // Send State to serial if 5 seconds passed
        if new || (time_in_state.wrapping_sub(time_since_send) > 5) {
            send_current_state(&inputs, &outputs, &temperature, &state, &mut serial);
            update_display(&mut display, 32.1, 34.4, state);
            time_since_send = time_in_state;
        }

        // Delay next update cycle
        delay.delay_ms(500_u16);
    }
}

fn update_display(display: &mut Display, puffer_oben: f32, puffer_unten: f32, state: State) {
    let mut delay = hal::delay::Delay::<Clock>::new();

    // Reset Disply
    display.clear(&mut delay).ok();

    // Print Current State
    display.write_str(state.to_string(), &mut delay).ok();

    // Create line with temperatures
    let mut line = heapless::Vec::<u8, heapless::consts::U16>::new();
    uwrite!(line, "O:{}°C U:{}°C", puffer_oben as u8, puffer_unten as u8).ok();

    // Send to display
    display.set_cursor_pos(40, &mut delay).ok();
    display.write_bytes(&line, &mut delay).ok();
}

/// Send current state to serial.
fn send_current_state(
    inputs: &io::Inputs,
    outputs: &io::Outputs,
    temperatures: &temperature::PlantTemperatures,
    state: &State,
    serial: &mut hal::usart::WriteUsart0<Clock>,
) {
    // Temperatures

    if let Some(temp_float) = temperatures.buffer_top {
        uwriteln!(
            serial,
            "--MQTT--Temperatur/Puffer_Oben:={}",
            (temp_float * 10.0) as i32
        )
        .ok();
    }
    if let Some(temp_float) = temperatures.buffer_buttom {
        uwriteln!(
            serial,
            "--MQTT--Temperatur/Puffer_Unten:={}",
            (temp_float * 10.0) as i32
        )
        .ok();
    }
    if let Some(temp_float) = temperatures.boiler {
        uwriteln!(
            serial,
            "--MQTT--Temperatur/Kessel:={}",
            (temp_float * 10.0) as i32
        )
        .ok();
    }
    if let Some(temp_float) = temperatures.warm_water {
        uwriteln!(
            serial,
            "--MQTT--Temperatur/Warmwasser:={}",
            (temp_float * 10.0) as i32
        )
        .ok();
    }

    // Inputs
    uwriteln!(
        serial,
        "--MQTT--Inputs/Brenner_Start:={}",
        bool2onoff(inputs.get_start_burner())
    )
    .ok();
    uwriteln!(
        serial,
        "--MQTT--Inputs/Pumpe_Warmwasser:={}",
        bool2onoff(inputs.get_warm_water_pump())
    )
    .ok();
    uwriteln!(
        serial,
        "--MQTT--Inputs/Pumpe_Heizung:={}",
        bool2onoff(inputs.get_heating_pump())
    )
    .ok();

    // Outputs
    uwriteln!(
        serial,
        "--MQTT--Outputs/Magnetventil_Puffer:={}",
        bool2onoff(outputs.get_magnet_valve_buffer())
    )
    .ok();
    uwriteln!(
        serial,
        "--MQTT--Outputs/Pumpe_Puffer:={}",
        bool2onoff(outputs.get_pump_buffer())
    )
    .ok();
    uwriteln!(
        serial,
        "--MQTT--Outputs/Brenner_Sperre:={}",
        bool2onoff(outputs.get_burner_inhibit())
    )
    .ok();

    // State
    uwriteln!(serial, "--MQTT--State:={}", state).ok();
}

fn bool2onoff(var: bool) -> &'static str {
    if var {
        "on"
    } else {
        "off"
    }
}
