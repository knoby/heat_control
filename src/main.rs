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

type Clock = hal::clock::MHz16;

//const DISPLAY_ADD_I2C: u8 = 0x27;

mod io;
mod onewire;
mod serial_logger;
mod temperature;
mod timer;

/*type Display = hd44780_driver::HD44780<
    hd44780_driver::bus::I2CBus<
        hal::i2c::I2c<Clock, hal::port::mode::Input<hal::port::mode::PullUp>>,
    >,
>; */

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
enum State {
    Init,
    BufferOff,
    BufferOn,
    ActivatePump,
    Error,
}

impl core::fmt::Display for State {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.to_string())
    }
}

impl State {
    fn to_string(&self) -> &'static str {
        match self {
            State::Init => "Init",
            State::BufferOff => "Buffer  Off",
            State::BufferOn => "Buffer On",
            State::ActivatePump => "Activate Pump",
            State::Error => "Error",
        }
    }
}

fn setup() -> (
    io::Inputs,
    io::Outputs,
    serial_logger::SerialLogger,
    temperature::Sensors,
    timer::Timer1,
) {
    // Get Peripherals for configuration
    let peripherals = chip::Peripherals::take().unwrap();

    // ------------------
    // Watchdog
    // ------------------

    // Get the ports
    let _portb = peripherals.PORTB.split();
    let portc = peripherals.PORTC.split();
    let portd = peripherals.PORTD.split();

    // ------------------
    // Serial Port
    // ------------------
    let rx = portd.pd0.into_floating_input(&portd.ddr);
    let tx = portd.pd1.into_output(&portd.ddr);
    let (_, serial) = hal::usart::Usart0::<Clock, hal::port::mode::Floating>::new(
        peripherals.USART0,
        rx,
        tx,
        9600,
    )
    .split();

    let mut serial = serial_logger::SerialLogger::new(serial, true, true, false);

    serial.info_str("Heat Control Init");

    // ------------------
    // digital IOs
    // ------------------
    serial.debug_str("Init IOs");
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
    serial.debug_str("Done");

    // ------------------
    // I2C Display
    // ------------------
    /*
        serial.debug_str("Init I2C Display");
        let sda = portc.pc4.into_pull_up_input(&portc.ddr);
        let scl = portc.pc5.into_pull_up_input(&portc.ddr);

        // Delay for display
        let mut delay = hal::delay::Delay::<Clock>::new();
        // Create the i2c bus
        let i2c = hal::i2c::I2c::<Clock, _>::new(peripherals.TWI, sda, scl, 400_000);
        // Create the display
        let mut display =
            hd44780_driver::HD44780::new_i2c(i2c, DISPLAY_ADD_I2C, &mut delay).unwrap();

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
        serial.debug_str("Done");
    */
    // ------------------
    // Init the Sensors
    // ------------------
    serial.debug_str("Init One Wire Sensors");
    // Split the pin for one wire
    let pd2 = portd.pd2.into_tri_state(&portd.ddr);

    // Setup the onewire bus
    let temperature_sensors = temperature::Sensors::setup(pd2.downgrade());
    serial.debug_str("Done");

    // ------------------
    // TIMER
    // ------------------
    serial.debug_str("Init Timer for millis");
    let timer1 = timer::Timer1::new(peripherals.TC1);
    serial.debug_str("Done");

    // ------------------
    // Enable Interrupts
    // ------------------
    serial.debug_str("Enable Interrupts");
    unsafe {
        avr_device::interrupt::enable();
    }
    serial.debug_str("Done");

    serial.debug_str("Init Done");

    (inputs, outputs, serial, temperature_sensors, timer1)
}

#[hal::entry]
fn main() -> ! {
    // Init the hardware
    let (mut inputs, mut outputs, mut serial, /*mut display,*/ mut sensors, timer1) = setup();

    // Some Variables used in the loop
    let mut temp_reading: temperature::PlantTemperatures =
        temperature::PlantTemperatures::default();
    let mut new: bool;
    let mut time_state_start = timer1.millis();
    let mut time_send = timer1.millis();
    let mut time_in_state = 0;
    let mut state = State::Init;
    let mut state_old = State::Init;
    let mut state_last = State::Init;

    // Main Loop
    loop {
        // Get Input state
        inputs.get_inputs();

        // Get Temperatures
        //temp_reading = sensors.read_temperatures().unwrap_or_default();

        // Simulation
        if timer1.millis() < 5_000 {
            temp_reading.buffer_top = None;
        } else if timer1.millis() < 25_000 {
            temp_reading.buffer_top = Some(22.4);
        } else if timer1.millis() < 35_000 {
            temp_reading.buffer_top = Some(33.8);
        } else {
            temp_reading.buffer_top = Some(18.5);
        }
        // Error Handling
        if temp_reading.buffer_top.is_none() {
            state = State::Error;
        }

        // Handle Statemachine
        new = state != state_last;
        state_last = state;
        if new {
            state_old = state;
            time_state_start = timer1.millis();
        }

        time_in_state = timer1.millis() - time_state_start;

        // State Machine
        match state {
            State::Error => {
                if new {}
                if temp_reading.buffer_top.is_some() {
                    state = State::Init;
                }
            }

            State::Init => {
                if new {}
                if time_in_state > 10_000 {
                    state = State::BufferOff;
                }
            }

            State::BufferOff => {
                if new {}
                if temp_reading.buffer_top.unwrap() > temperature::MIN_BUFFER_TEMPERATURE {
                    state = State::BufferOn;
                }
            }

            State::BufferOn => {
                if new {}
                if (temp_reading.buffer_top.unwrap() - temperature::BUFFER_HYSTERESIS)
                    > temperature::MIN_BUFFER_TEMPERATURE
                {
                    state = State::BufferOff;
                }
            }

            State::ActivatePump => if new {},
        }

        // Update Ouptuts
        outputs.set_outputs();

        if (timer1.millis() - time_send) > 1_000 {
            time_send = timer1.millis();
            serial.debug_str(state.to_string());
            if let Some(temp) = temp_reading.buffer_top {
                serial.debug_i32(temp as i32, "Puffer Oben");
            } else {
                serial.debug_str("None");
            }
            serial.debug_i32(time_in_state as i32, "Time in State");
            serial.debug_str("-------------------");
        }
    }
}
