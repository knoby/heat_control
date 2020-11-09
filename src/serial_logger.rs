use atmega328p_hal as hal;
use hal::prelude::*;

#[allow(dead_code)]
pub struct SerialLogger {
    debug: bool,
    info: bool,
    mqtt: bool,
    serial: hal::usart::WriteUsart0<super::Clock>,
}

#[allow(dead_code)]
impl SerialLogger {
    pub fn new(
        serial: hal::usart::WriteUsart0<super::Clock>,
        debug: bool,
        info: bool,
        mqtt: bool,
    ) -> Self {
        Self {
            debug,
            info,
            mqtt,
            serial,
        }
    }

    fn new_line(&mut self) {
        self.serial.write_char('\n').ok();
    }

    fn write_i16(&mut self, var: i16) {
        let mut devider = 10;
        let mut var_32 = var;
        if var_32 < 0 {
            self.serial.write_str("-").ok();
            var_32 *= -1;
        }
        // find starting devider
        while var_32 >= devider {
            devider *= 10;
        }
        devider /= 10;
        loop {
            let decimal = var_32 / devider;
            self.write_num(decimal);
            var_32 -= decimal * devider;
            devider /= 10;
            if devider == 0 {
                break;
            }
        }
    }

    fn write_i16_fixedpoint(&mut self, var: i16) {
        let mut devider = 10;
        let mut var_32 = var;
        if var_32 < 0 {
            self.serial.write_str("-").ok();
            var_32 *= -1;
        }
        // find starting devider
        while var_32 >= devider {
            devider *= 10;
        }
        devider /= 10;
        loop {
            let decimal = var_32 / devider;
            self.write_num(decimal);
            var_32 -= decimal * devider;
            devider /= 10;
            if devider == 1 {
                self.serial.write_str(".").ok();
            }
            if devider == 0 {
                break;
            }
        }
    }

    fn write_num(&mut self, num: i16) {
        match num {
            0 => self.serial.write_str("0").ok(),
            1 => self.serial.write_str("1").ok(),
            2 => self.serial.write_str("2").ok(),
            3 => self.serial.write_str("3").ok(),
            4 => self.serial.write_str("4").ok(),
            5 => self.serial.write_str("5").ok(),
            6 => self.serial.write_str("6").ok(),
            7 => self.serial.write_str("7").ok(),
            8 => self.serial.write_str("8").ok(),
            9 => self.serial.write_str("9").ok(),
            _ => Some(()),
        };
    }

    fn str(&mut self, text: &str) {
        self.serial.write_str(text).ok();
        self.new_line();
    }

    fn i16(&mut self, var: i16, text: &str) {
        self.serial.write_str(text).ok();
        self.serial.write_str(": ").ok();
        self.write_i16(var);
        self.new_line();
    }

    fn bool(&mut self, var: bool, text: &str) {
        self.serial.write_str(text).ok();
        self.serial.write_str(": ").ok();
        if var {
            self.serial.write_str("TRUE").ok();
        } else {
            self.serial.write_str("FALSE").ok();
        }
        self.new_line();
    }

    pub fn debug_bool(&mut self, var: bool, text: &str) {
        if self.debug {
            self.bool(var, text);
        }
    }

    pub fn debug_i16(&mut self, var: i16, text: &str) {
        if self.debug {
            self.i16(var, text);
        }
    }

    pub fn debug_str(&mut self, text: &str) {
        if self.debug {
            self.str(text);
        }
    }

    pub fn debug_option_i16(&mut self, var: Option<i16>, text: &str) {
        if self.debug {
            if let Some(temp) = var {
                self.debug_i16(temp, text);
            } else {
                self.serial.write_str(text).ok();
                self.serial.write_str(": ").ok();
                self.serial.write_str("None").ok();
            }
        }
    }

    pub fn info_bool(&mut self, var: bool, text: &str) {
        if self.info {
            self.bool(var, text);
        }
    }

    pub fn info_i16(&mut self, var: i16, text: &str) {
        if self.info {
            self.i16(var, text);
        }
    }

    pub fn info_str(&mut self, text: &str) {
        if self.info {
            self.str(text);
        }
    }

    pub fn mqtt_bool(&mut self, var: bool, text: &str) {
        if self.mqtt {
            self.mqtt_prefix();
            self.serial.write_str(text).ok();
            self.serial.write_str(":=").ok();
            if var {
                self.serial.write_str("On").ok();
            } else {
                self.serial.write_str("Off").ok();
            }
            self.new_line();
        }
    }

    pub fn mqtt_option_i16(&mut self, var: Option<i16>, text: &str) {
        if self.mqtt {
            if let Some(var) = var {
                self.mqtt_prefix();
                self.serial.write_str(text).ok();
                self.serial.write_str(":=").ok();
                self.write_i16_fixedpoint(var);
                self.new_line();
            }
        }
    }

    pub fn mqtt_str(&mut self, var: &str, topic: &str) {
        if self.mqtt {
            self.mqtt_prefix();
            self.serial.write_str(topic).ok();
            self.serial.write_str(":=").ok();
            self.serial.write_str(var).ok();
            self.new_line();
        }
    }

    fn mqtt_prefix(&mut self) {
        self.serial.write_str("--MQTT--").ok();
    }
}
