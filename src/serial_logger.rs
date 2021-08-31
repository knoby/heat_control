use atmega_hal as hal;
use hal::{
    pac::USART0,
    port::{
        mode::{Input, Output},
        Pin, PD0, PD1,
    },
};

type UsartWrite = hal::usart::UsartWriter<USART0, Pin<Input, PD0>, Pin<Output, PD1>, super::Clock>;

#[allow(dead_code)]
pub struct SerialLogger {
    debug: bool,
    info: bool,
    mqtt: bool,
    serial: UsartWrite,
}

#[allow(dead_code)]
impl SerialLogger {
    pub fn new(serial: UsartWrite, debug: bool, info: bool, mqtt: bool) -> Self {
        Self {
            debug,
            info,
            mqtt,
            serial,
        }
    }

    pub fn debug_bool(&mut self, var: bool, text: &str) {
        if self.debug {
            ufmt::uwriteln!(&mut self.serial, "{}: {}", text, var).ok();
        }
    }

    pub fn debug_i16(&mut self, var: i16, text: &str) {
        if self.debug {
            ufmt::uwriteln!(&mut self.serial, "{}: {}", text, var).ok();
        }
    }

    pub fn debug_str(&mut self, text: &str) {
        if self.debug {
            ufmt::uwriteln!(&mut self.serial, "{}", text).ok();
        }
    }

    pub fn debug_option_i16(&mut self, var: Option<i16>, text: &str) {
        if self.debug {
            if let Some(temp) = var {
                ufmt::uwriteln!(&mut self.serial, "{}: {}", text, temp).ok();
            } else {
                ufmt::uwriteln!(&mut self.serial, "{}: None", text).ok();
            }
        }
    }

    pub fn info_bool(&mut self, var: bool, text: &str) {
        if self.info {
            ufmt::uwriteln!(&mut self.serial, "{}: {}", text, var).ok();
        }
    }

    pub fn info_i16(&mut self, var: i16, text: &str) {
        if self.info {
            ufmt::uwriteln!(&mut self.serial, "{}: {}", text, var).ok();
        }
    }

    pub fn info_str(&mut self, text: &str) {
        if self.info {
            ufmt::uwriteln!(&mut self.serial, "{}", text).ok();
        }
    }

    pub fn mqtt_bool(&mut self, var: bool, text: &str) {
        if self.mqtt {
            if var {
                ufmt::uwriteln!(&mut self.serial, "--MQTT--{}:=On", text).ok();
            } else {
                ufmt::uwriteln!(&mut self.serial, "--MQTT--{}:=Off", text).ok();
            }
        }
    }

    pub fn mqtt_option_i16(&mut self, var: Option<i16>, text: &str) {
        if self.mqtt {
            if let Some(var) = var {
                ufmt::uwriteln!(&mut self.serial, "--MQTT--{}:={}", text, var).ok();
            }
        }
    }

    pub fn mqtt_str(&mut self, var: &str, topic: &str) {
        if self.mqtt {
            ufmt::uwriteln!(&mut self.serial, "--MQTT--{}:={}", topic, var).ok();
        }
    }
}
