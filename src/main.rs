#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// Pull in the panic handler from panic-halt
extern crate avr_std_stub;

#[macro_use]
extern crate machine;

use atmega328p_hal as hal;
use atmega328p_hal::atmega328p as chip;
use hal::prelude::*;

type Clock = hal::clock::MHz16;

mod display;
mod io;
mod onewire;
mod serial_logger;
mod statemachine;
mod temperature;
mod timer;

const DISPLAY_ADD_I2C: u8 = 0x27;

#[derive(PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum State {
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
    serial_logger::SerialLogger,
    timer::Timer1,
    io::Outputs,
    io::Inputs,
    temperature::Sensors,
    display::Display,
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

    serial.debug_str("Init I2C Display");
    let sda = portc.pc4.into_pull_up_input(&portc.ddr);
    let scl = portc.pc5.into_pull_up_input(&portc.ddr);

    // Delay for display
    let mut delay = hal::delay::Delay::<Clock>::new();
    // Create the i2c bus
    let i2c = hal::i2c::I2c::<Clock, _>::new(peripherals.TWI, sda, scl, 400_000);
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

    let display = display::Display::new(display);

    serial.debug_str("Done");

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

    (
        serial,
        timer1,
        outputs,
        inputs,
        temperature_sensors,
        display,
    )
}

#[hal::entry]
fn main() -> ! {
    // Init the hardware
    let (mut serial, timer1, mut outputs, mut inputs, mut sensors, mut display) = setup();

    let mut state = statemachine::HeatControl::init(0);

    // Main Loop
    loop {
        let time = timer1.millis();

        inputs.get_inputs();

        let temp_reading = sensors.read_temperatures().unwrap_or_default();

        #[cfg(feature = "simulation")]
        {
            if time < 5 {
                temp_reading.buffer_top = None;
                inputs.start_burner = false;
            } else if time < 10 {
                temp_reading.buffer_top = Some(33);
                inputs.start_burner = false;
            } else if time < 20 {
                temp_reading.buffer_top = Some(33);
                inputs.start_burner = true;
            } else {
                temp_reading.buffer_top = Some(32);
                inputs.start_burner = true;
            }
        }

        // State Machine
        let new_state = match state {
            statemachine::HeatControl::Init(_) => state.on_tick(statemachine::Tick { time }),
            statemachine::HeatControl::BufferDisabled(_) => {
                if let Some(temp) = temp_reading.buffer_top {
                    if temp > temperature::MIN_BUFFER_TEMPERATURE {
                        state.on_enable(statemachine::Enable {})
                    } else {
                        state
                    }
                } else {
                    state
                }
            }
            _ => state,
        };

        state = new_state;

        outputs.set_outputs();
    }
}
