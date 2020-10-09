#![allow(dead_code)]

use core::convert::{TryFrom, TryInto};
use embedded_hal as hal;
use hal::blocking::delay::DelayUs;
use hal::digital::v2::*;

#[repr(u8)]
/// Commands as specified in the datasheet
pub enum Command {
    /// Start a conversion
    Convert = 0x44,
    /// Write Configuration to sensor
    WriteScratchpad = 0x4e,
    /// Read the whole scratchpad from the sensor
    ReadScratchpad = 0xBE,
    /// Copy scratchpad to eeprom
    CopyScratchpad = 0x48,
    /// Read scratchpad from eeprom
    RecallE2 = 0xB8,
    /// Check if there is a sensor on the bus that uses parasite power
    ReadPowerSupply = 0xB4,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
/// Measurement resolutions defined in the manual.
/// The enum value can be used as a u8 vale for the config register
pub enum MeasureResolution {
    /// 9 Bit Resolution 0.5K Percision
    Bit09 = 0b0001_1111,
    /// 10 Bit Resolution 0.25K Percision
    Bit10 = 0b0011_1111,
    /// 11 Bit Resolution 0.125K Percision
    Bit11 = 0b0101_1111,
    /// 12 Bit Resolution 0.0625K Percision
    Bit12 = 0b0111_1111,
}

impl TryFrom<u8> for MeasureResolution {
    type Error = ();

    fn try_from(num: u8) -> Result<Self, Self::Error> {
        let resolution = match num {
            0b0001_1111 => MeasureResolution::Bit09,
            0b0011_1111 => MeasureResolution::Bit10,

            0b0101_1111 => MeasureResolution::Bit11,
            0b0111_1111 => MeasureResolution::Bit12,
            _ => return Err(()),
        };

        Ok(resolution)
    }
}

impl MeasureResolution {
    /// Return the conversion time in ms
    pub fn conversion_time(&self) -> u16 {
        match self {
            MeasureResolution::Bit09 => 94,
            MeasureResolution::Bit10 => 188,
            MeasureResolution::Bit11 => 375,
            MeasureResolution::Bit12 => 750,
        }
    }
}

pub struct DS18B20 {
    resolution: Option<MeasureResolution>,
    device: super::Device,
}

/// Struct that holds

impl DS18B20 {
    pub const fn family_code() -> u8 {
        0x28
    }

    /// Create a ds18b20 from a one wire device. Returns the device if the family code does not match
    pub fn from_device(device: super::Device) -> Result<Self, super::Device> {
        // Check the famliy code
        if device.rom_no[0] == Self::family_code() {
            Ok(Self {
                device,
                resolution: None,
            })
        } else {
            Err(device)
        }
    }

    /// Create a new sensor from an rom_no. Returns Err if the family code does not match
    pub fn new(rom_no: [u8; 8]) -> Result<Self, ()> {
        // Check the family code
        if rom_no[0] == Self::family_code() {
            Ok(Self {
                device: super::Device { rom_no },
                resolution: None,
            })
        } else {
            Err(())
        }
    }

    /// Reads the temperature from a device
    pub fn read_temperature<E: Sized, IO: OutputPin<Error = E> + InputPin<Error = E>>(
        &mut self,
        bus: &mut super::OneWire<IO>,
        delay: &mut dyn DelayUs<u16>,
    ) -> Result<i16, super::Error<E>> {
        bus.reset_and_select_rom(&self.device.rom_no, delay)?;

        bus.write_byte(Command::ReadScratchpad as u8, delay)?;

        let mut scratchpad = [0_u8; 8];
        bus.read_bytes(&mut scratchpad, delay)?;

        // Get resolution
        self.resolution = scratchpad[4].try_into().ok();

        let devider = if let Some(resolution) = self.resolution {
            match resolution {
                MeasureResolution::Bit09 => 2,
                MeasureResolution::Bit10 => 4,
                MeasureResolution::Bit11 => 8,
                MeasureResolution::Bit12 => 16,
            }
        } else {
            return Err(super::Error::DataError);
        };

        let temperature_raw = i16::from_le_bytes([scratchpad[0], scratchpad[1]]);

        if temperature_raw as u16 == 0xff_ff_u16 {
            // No response from device
            return Err(super::Error::DataError);
        }

        let temperature = temperature_raw / devider;

        Ok(temperature)
    }

    /// Start conversion on all devices on the bus
    pub fn start_measurement<E: Sized, IO: OutputPin<Error = E> + InputPin<Error = E>>(
        bus: &mut super::OneWire<IO>,
        delay: &mut dyn DelayUs<u16>,
    ) -> Result<(), super::Error<E>> {
        bus.reset(delay)?;

        bus.write_byte(super::Command::SkipRom as u8, delay)?;

        bus.write_byte(Command::Convert as u8, delay)?;

        Ok(())
    }

    pub fn set_resolution<E: Sized, IO: OutputPin<Error = E> + InputPin<Error = E>>(
        &mut self,
        bus: &mut super::OneWire<IO>,
        delay: &mut dyn DelayUs<u16>,
        resolution: MeasureResolution,
    ) -> Result<MeasureResolution, super::Error<E>> {
        self.resolution = None;

        bus.reset_and_select_rom(&self.device.rom_no, delay)?;

        bus.write_byte(Command::WriteScratchpad as u8, delay)?;
        bus.write_bytes(&[0, 0, resolution as u8], delay)?;

        self.resolution = Some(resolution);

        Ok(resolution)
    }
}
