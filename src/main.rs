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
//! - HEAT_FLOW {0x28, 0xFF, 0x2C, 0x99, 0x74, 0x16, 0x04, 0xB5}
//! - HEAT_RETURN {0x28, 0xff, 0x4b, 0x96, 0x74, 0x16, 0x04, 0x00}

#![no_std]
#![no_main]

// Pull in the panic handler from panic-halt
extern crate avr_std_stub;

use atmega328p_hal as hal;
use atmega328p_hal::atmega328p as chip;
use hal::prelude::*;

type Clock = hal::clock::MHz16;

const DISPLAY_ADD_I2C: u8 = 0x27;

mod io;
mod onewire;
mod temperature;

fn setup() -> (
    io::Inputs,
    io::Outputs,
    hal::usart::WriteUsart0<Clock>,
    hd44780_driver::HD44780<
        hd44780_driver::bus::I2CBus<
            hal::i2c::I2c<Clock, hal::port::mode::Input<hal::port::mode::PullUp>>,
        >,
    >,
    temperature::Sensors,
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

    serial.write_str("Heat Control/n").ok();

    // ------------------
    // I2C Display
    // ------------------
    let sda = portc.pc4.into_pull_up_input(&portc.ddr);
    let scl = portc.pc5.into_pull_up_input(&portc.ddr);

    // Create the i2c bus
    let i2c = hal::i2c::I2c::<Clock, _>::new(peripherals.TWI, sda, scl, 400_000);
    // Delay for display
    let mut delay = hal::delay::Delay::<Clock>::new();
    // Create the display
    let mut display = hd44780_driver::HD44780::new_i2c(i2c, DISPLAY_ADD_I2C, &mut delay).unwrap();
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

    // ------------------
    // Init the Sensors
    // ------------------

    // Split the pin for one wire
    let pd2 = portd.pd2.into_tri_state(&portd.ddr);

    // Setup the onewire bus
    let temperature_sensors = temperature::Sensors::setup(pd2.downgrade());

    // ------------------
    // digital IOs
    // ------------------

    let inputs = io::Inputs {
        heating_pump: portc.pc1.into_floating_input(&portc.ddr).downgrade(),
        start_burner: portc.pc3.into_floating_input(&portc.ddr).downgrade(),
        warm_water_pump: portc.pc0.into_floating_input(&portc.ddr).downgrade(),
    };

    let outputs = io::Outputs {
        burner_inhibit: portd.pd6.into_output(&portd.ddr).downgrade(),
        magnet_valve_buffer: portd.pd5.into_output(&portd.ddr).downgrade(),
        pump_buffer: portd.pd4.into_output(&portd.ddr).downgrade(),
    };

    (inputs, outputs, serial, display, temperature_sensors)
}

#[hal::entry]
fn main() -> ! {
    // This line forces the compiler to pull in the panic function. If the panic function can not be triggerd in main, but in a module, a linker error occures
    let _: () = Some(()).unwrap();

    // Init the hardware
    let (inputs, outputs, mut serial, display, mut sensors) = setup();

    loop {
        sensors.read_temperatures();
    }
}