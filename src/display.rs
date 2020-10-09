use atmega328p_hal as hal;
use hal::prelude::*;

type I2CDisplay = hd44780_driver::HD44780<
    hd44780_driver::bus::I2CBus<
        hal::i2c::I2c<super::Clock, hal::port::mode::Input<hal::port::mode::PullUp>>,
    >,
>;

pub struct Display {
    display: I2CDisplay,
    delay: hal::delay::Delay<super::Clock>,
}

impl Display {
    pub fn new(mut display: I2CDisplay) -> Self {
        let mut delay = hal::delay::Delay::<super::Clock>::new();
        display.clear(&mut delay).ok();
        Self { display, delay }
    }

    pub fn set_state(&mut self, state: super::State) {
        self.display.set_cursor_pos(0, &mut self.delay).ok();
        self.display
            .write_str("                ", &mut self.delay)
            .ok();
        self.display.set_cursor_pos(0, &mut self.delay).ok();
        self.display
            .write_str(state.to_string(), &mut self.delay)
            .ok();
    }

    pub fn set_temp_top(&mut self, temp: Option<i16>) {
        self.display.set_cursor_pos(0x40, &mut self.delay).ok();
        self.display.write_str("        ", &mut self.delay).ok();
        self.display.set_cursor_pos(0x40, &mut self.delay).ok();
        self.display.write_str("O:", &mut self.delay).ok();
        self.display
            .write_bytes(&Self::temp_to_bytes(temp), &mut self.delay)
            .ok();
    }

    pub fn set_temp_bottom(&mut self, temp: Option<i16>) {
        self.display.set_cursor_pos(0x48, &mut self.delay).ok();
        self.display.write_str("        ", &mut self.delay).ok();
        self.display.set_cursor_pos(0x48, &mut self.delay).ok();
        self.display.write_str("U:", &mut self.delay).ok();
        self.display
            .write_bytes(&Self::temp_to_bytes(temp), &mut self.delay)
            .ok();
    }

    fn temp_to_bytes(temp: Option<i16>) -> [u8; 4] {
        let mut out = [0; 4];
        if let Some(temp) = temp {
            // Check sign
            if temp < 0 {
                out[0] = b'-';
            } else {
                out[0] = b'+';
            }
            // Parse the
            let mut devider = 100;
            let mut var = temp;
            for index in 1..4 {
                let decimal = var / devider;
                out[index] = Self::num2char(decimal);
                var -= decimal * devider;
                devider /= 10;
            }
        } else {
            out = [b'N', b'o', b'n', b'e'];
        }
        out
    }

    fn num2char(input: i16) -> u8 {
        match input {
            0 => b'0',
            1 => b'1',
            2 => b'2',
            3 => b'3',
            4 => b'4',
            5 => b'5',
            6 => b'6',
            7 => b'7',
            8 => b'8',
            9 => b'9',
            _ => b' ',
        }
    }
}
