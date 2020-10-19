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

const MIN_CYCLE_TIME: u32 = 1_000;
const WATCHDOG_TIME: hal::wdt::Timeout = hal::wdt::Timeout::Ms4000;
const DISPLAY_UPDATE_TIME: u32 = 10_000;
const MQTT_UPDATE_TIME: u32 = 15_000;
const SERIAL_UPDATE_TIME: u32 = 10_000;

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
    hal::wdt::Wdt,
) {
    // Get Peripherals for configuration
    let peripherals = chip::Peripherals::take().unwrap();

    // Get the ports
    let _portb = peripherals.PORTB.split();
    let portc = peripherals.PORTC.split();
    let portd = peripherals.PORTD.split();

    // ------------------
    // Watchdog
    // ------------------
    let mut watchdog = hal::wdt::Wdt::new(&peripherals.CPU.mcusr, peripherals.WDT);
    watchdog.start(WATCHDOG_TIME);

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

    let mut serial = serial_logger::SerialLogger::new(serial, false, false, true);

    serial.info_str("Heat Control Init");

    // ------------------
    // digital IOs
    // ------------------
    serial.debug_str("Init IOs");

    let inputs = io::Inputs::new(
        portc.pc2.into_floating_input(&portc.ddr).downgrade(),
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
        watchdog,
    )
}

use core::clone::Clone;
#[hal::entry]
fn main() -> ! {
    // Init the hardware
    let (mut serial, timer1, mut outputs, mut inputs, mut sensors, mut display, mut watchdog) =
        setup();

    let mut state = statemachine::HeatControl::init(timer1.millis());
    let mut time_display = 0;
    let mut time_mqtt: u32 = 0;
    let mut time_serial = 0;
    let mut old_state = state.to_u8();

    // Main Loop
    loop {
        let time = timer1.millis();

        inputs.get_inputs();

        #[cfg(not(feature = "simulation"))]
        let temp_reading = sensors.read_temperatures().unwrap_or_default();

        #[cfg(feature = "simulation")]
        let mut temp_reading = sensors.read_temperatures().unwrap_or_default();

        #[cfg(feature = "simulation")]
        {
            if time < 5_000 {
                temp_reading.buffer_top = None;
                inputs.start_burner = false;
            } else if time < 20_000 {
                temp_reading.buffer_top = Some(33);
                inputs.start_burner = false;
            } else if time < 40_000 {
                temp_reading.buffer_top = Some(34);
                inputs.start_burner = true;
            } else {
                temp_reading.buffer_top = Some(32);
                inputs.start_burner = true;
            }
        }

        // State Machine
        {
            use statemachine::*;
            use temperature::*;

            let new_state = match state {
                statemachine::HeatControl::Init(_) => {
                    outputs.set_burner_inhibit(false);
                    outputs.set_magnet_valve_buffer(false);
                    outputs.set_pump_buffer(false);

                    state.on_tick(Tick { time })
                }

                statemachine::HeatControl::BufferDisabled(_) => {
                    outputs.set_burner_inhibit(false);
                    outputs.set_magnet_valve_buffer(false);
                    outputs.set_pump_buffer(false);

                    match (
                        temp_reading.buffer_top,
                        inputs.get_start_burner(),
                        inputs.get_heating_pump(),
                    ) {
                        (Some(temp), false, true)
                            if temp >= (MIN_BUFFER_TEMPERATURE + BUFFER_HYSTERESIS) =>
                        {
                            state.on_enable(Enable {})
                        }
                        (_, _, _) => state,
                    }
                }

                statemachine::HeatControl::BufferEnabled(_) => {
                    outputs.set_burner_inhibit(true);
                    outputs.set_magnet_valve_buffer(true);
                    outputs.set_pump_buffer(false);

                    match (
                        temp_reading.buffer_top,
                        inputs.get_start_burner(),
                        inputs.get_heating_pump(),
                    ) {
                        (None, _, _) => state.on_disable(Disable {}),
                        (Some(temp), _, _) if temp < MIN_BUFFER_TEMPERATURE => {
                            state.on_disable(Disable {})
                        }
                        (_, _, false) => state.on_disable(Disable {}),
                        (Some(_), true, _) => state.on_activate_pump(ActivatePump { time }),
                        (_, _, _) => state,
                    }
                }

                statemachine::HeatControl::PumpActive(_) => {
                    outputs.set_burner_inhibit(true);
                    outputs.set_magnet_valve_buffer(true);
                    outputs.set_pump_buffer(true);

                    state.on_tick(statemachine::Tick { time })
                }

                statemachine::HeatControl::PumpPause(_) => {
                    outputs.set_burner_inhibit(true);
                    outputs.set_magnet_valve_buffer(true);
                    outputs.set_pump_buffer(false);

                    state.on_tick(statemachine::Tick { time })
                }

                _ => {
                    outputs.set_burner_inhibit(true);
                    outputs.set_magnet_valve_buffer(true);
                    outputs.set_pump_buffer(false);
                    state
                }
            };

            state = new_state;
        }

        let current_state = state.to_u8();

        // Set Outputs
        outputs.set_outputs();

        // Handle Display
        if (time.wrapping_sub(time_display) >= DISPLAY_UPDATE_TIME) || (current_state != old_state)
        {
            display.set_state(state.to_string());
            display.set_temp_top(temp_reading.buffer_top);
            display.set_temp_bottom(temp_reading.buffer_buttom);
            time_display = time;
        }

        // Handle MQTT Messages
        if (time.wrapping_sub(time_mqtt) >= MQTT_UPDATE_TIME) || (old_state != current_state) {
            serial.debug_str(state.to_string());

            serial.mqtt_option_i16(temp_reading.buffer_top, "Temperature/Puffer_Oben");
            serial.mqtt_option_i16(temp_reading.buffer_buttom, "Temperature/Puffer_Unten");
            serial.mqtt_option_i16(temp_reading.warm_water, "Temperature/Warmwasser");
            serial.mqtt_option_i16(temp_reading.boiler, "Temperature/Kessel");

            serial.mqtt_bool(inputs.get_start_burner(), "Inputs/BrennerStart");
            serial.mqtt_bool(inputs.get_warm_water_pump(), "Inputs/Pumpe_Warmwasser");
            serial.mqtt_bool(inputs.get_heating_pump(), "Inputs/Pumpe_Heizung");

            serial.mqtt_bool(outputs.get_burner_inhibit(), "Outputs/Brenner_Sperre");
            serial.mqtt_bool(
                outputs.get_magnet_valve_buffer(),
                "Outputs/Magnetventil_Puffer",
            );
            serial.mqtt_bool(outputs.get_pump_buffer(), "Outputs/Pumpe_Puffer");

            serial.mqtt_str(state.to_string(), "State");

            time_mqtt = time;
        }

        // Handle Serial Connection Debug
        if (time.wrapping_sub(time_serial) >= SERIAL_UPDATE_TIME) || (current_state != old_state) {
            serial.debug_str(state.to_string());

            serial.debug_option_i16(temp_reading.buffer_top, "Buffer Top");
            serial.debug_option_i16(temp_reading.buffer_buttom, "Buffer Bottom");
            serial.debug_option_i16(temp_reading.warm_water, "Warmwater");
            serial.debug_option_i16(temp_reading.boiler, "Boiler");

            serial.debug_bool(inputs.get_start_burner(), "Start Burner");
            serial.debug_bool(inputs.get_warm_water_pump(), "Warmwater Pump");
            serial.debug_bool(inputs.get_heating_pump(), "Heating Pump");

            serial.debug_bool(outputs.get_burner_inhibit(), "Burner Inhibit");
            serial.debug_bool(outputs.get_magnet_valve_buffer(), "Magnet Valve Buffer");
            serial.debug_bool(outputs.get_pump_buffer(), "Pump Buffer");

            time_serial = time;
        }

        old_state = state.to_u8();

        // Feed the watchdog
        watchdog.feed();

        // Get time for calculating to this point. Delay next loop to have a nearly const cycle time
        let calc_time = timer1.millis().wrapping_sub(time);
        if calc_time <= MIN_CYCLE_TIME {
            let delay_time = MIN_CYCLE_TIME - calc_time;
            hal::delay::Delay::<Clock>::new().delay_ms(delay_time as u16);
        }
    }
}
