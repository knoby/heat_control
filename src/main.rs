#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

// Pull in the panic handler from panic-halt
extern crate avr_std_stub;

use atmega328p_hal as hal;
use atmega328p_hal::atmega328p as chip;
use hal::prelude::*;

type Clock = hal::clock::MHz16;

mod display;
mod io;
mod onewire;
mod serial_logger;
mod temperature;
mod timer;

const DISPLAY_ADD_I2C: u8 = 0x27;

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

    // Some Variables used in the loop
    let mut temp_reading: temperature::PlantTemperatures;
    let mut new: bool;
    let mut time = 0_i16;
    let mut time_state_start = time;
    let mut time_in_state: i16;
    let mut state = State::Init;
    let mut state_last = State::Init;
    let mut state_old = State::Init;

    // Main Loop
    loop {
        time = timer1.millis() as i16;

        inputs.get_inputs();

        temp_reading = sensors.read_temperatures().unwrap_or_default();

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

        // Error Handling
        if temp_reading.buffer_top.is_none() {
            state = State::Error;
        }

        // Handle Statemachine
        new = state != state_last;
        if new {
            state_old = state_last;
            time_state_start = time;
        }
        state_last = state;

        time_in_state = time.wrapping_sub(time_state_start);

        // State Machine
        match state {
            State::Error => {
                if new {
                    outputs.set_burner_inhibit(false);
                    outputs.set_magnet_valve_buffer(false);
                    outputs.set_pump_buffer(false);
                }
                if temp_reading.buffer_top.is_some() {
                    state = State::Init;
                }
            }

            State::Init => {
                if new {
                    outputs.set_burner_inhibit(false);
                    outputs.set_magnet_valve_buffer(false);
                    outputs.set_pump_buffer(false);
                }
                if time_in_state > 2 {
                    state = State::BufferOff;
                }
            }

            State::BufferOff => {
                if new {
                    outputs.set_burner_inhibit(false);
                    outputs.set_magnet_valve_buffer(false);
                    outputs.set_pump_buffer(false);
                }
                if (temp_reading.buffer_top.unwrap() > temperature::MIN_BUFFER_TEMPERATURE)
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
                }
                if temp_reading.buffer_top.unwrap()
                    < (temperature::MIN_BUFFER_TEMPERATURE - temperature::BUFFER_HYSTERESIS)
                {
                    state = State::BufferOff;
                } else if inputs.get_start_burner()
                    && ((state_old == State::BufferOff) || (time_in_state > 30))
                {
                    state = State::ActivatePump;
                }
            }

            State::ActivatePump => {
                if new {
                    outputs.set_burner_inhibit(true);
                    outputs.set_magnet_valve_buffer(true);
                    outputs.set_pump_buffer(true);
                }
                if time_in_state >= 30 {
                    state = State::BufferOn;
                }
            }
        }

        outputs.set_outputs();

        if new {
            serial.debug_str(state.to_string());
            serial.debug_str(state_old.to_string());
            serial.debug_i16(time, "Time");
            serial.debug_str("-------------------");

            display.set_state(state);
            display.set_temp_top(temp_reading.buffer_top);
            display.set_temp_bottom(temp_reading.buffer_buttom);
        }
    }
}
