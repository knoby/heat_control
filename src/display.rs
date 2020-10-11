use atmega328p_hal as hal;

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
        display.write_str("Heat Control", &mut delay).ok();
        Self { display, delay }
    }

    pub fn set_state(&mut self, state: &str) {
        self.display.set_cursor_pos(0, &mut self.delay).ok();
        self.display.write_str(state, &mut self.delay).ok();
        match state.len() {
            0 => self.display.write_bytes(&[0x20; 16], &mut self.delay).ok(),
            1 => self.display.write_bytes(&[0x20; 15], &mut self.delay).ok(),
            2 => self.display.write_bytes(&[0x20; 14], &mut self.delay).ok(),
            3 => self.display.write_bytes(&[0x20; 13], &mut self.delay).ok(),
            4 => self.display.write_bytes(&[0x20; 12], &mut self.delay).ok(),
            5 => self.display.write_bytes(&[0x20; 11], &mut self.delay).ok(),
            6 => self.display.write_bytes(&[0x20; 10], &mut self.delay).ok(),
            7 => self.display.write_bytes(&[0x20; 9], &mut self.delay).ok(),
            8 => self.display.write_bytes(&[0x20; 8], &mut self.delay).ok(),
            9 => self.display.write_bytes(&[0x20; 7], &mut self.delay).ok(),
            10 => self.display.write_bytes(&[0x20; 6], &mut self.delay).ok(),
            11 => self.display.write_bytes(&[0x20; 5], &mut self.delay).ok(),
            12 => self.display.write_bytes(&[0x20; 4], &mut self.delay).ok(),
            13 => self.display.write_bytes(&[0x20; 3], &mut self.delay).ok(),
            14 => self.display.write_bytes(&[0x20; 2], &mut self.delay).ok(),
            15 => self.display.write_bytes(&[0x20; 1], &mut self.delay).ok(),
            _ => Some(()),
        };
    }

    pub fn set_temp_top(&mut self, temp: Option<i16>) {
        self.display.set_cursor_pos(0x40, &mut self.delay).ok();
        self.display.write_str("O:", &mut self.delay).ok();
        self.display
            .write_bytes(&Self::temp_to_bytes(temp), &mut self.delay)
            .ok();
        self.display.write_bytes(&[0x20; 2], &mut self.delay).ok();
    }

    pub fn set_temp_bottom(&mut self, temp: Option<i16>) {
        self.display.set_cursor_pos(0x48, &mut self.delay).ok();
        self.display.write_str("U:", &mut self.delay).ok();
        self.display
            .write_bytes(&Self::temp_to_bytes(temp), &mut self.delay)
            .ok();
        self.display.write_bytes(&[0x20; 2], &mut self.delay).ok();
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
            for byte in out[1..4].iter_mut() {
                let decimal = var / devider;
                *byte = Self::num2char(decimal);
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
