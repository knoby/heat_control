#![allow(dead_code)]

use core::cmp::Ordering;
use embedded_hal as hal;
use hal::blocking::delay::DelayUs;
use hal::digital::v2::*;

pub mod ds18b20;

pub use ds18b20::DS18B20;

#[repr(u8)]
enum Command {
    Search = 0xF0,
    SelectRom = 0x55,
    SkipRom = 0xCC,
}

pub struct SearchState {
    last_discrepancy: u8,
    rom_no: [u8; 8],
    status: SearchStatus,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            last_discrepancy: 0,
            rom_no: [0; 8],
            status: SearchStatus::Next,
        }
    }
}

#[repr(u8)]
#[derive(PartialEq)]
enum SearchStatus {
    Next,
    End,
}

pub struct OneWire<IO: InputPin + OutputPin> {
    io_pin: IO,
}

impl<E: Sized, IO: InputPin<Error = E> + OutputPin<Error = E>> OneWire<IO> {
    pub fn new(io_pin: IO) -> Self {
        Self { io_pin }
    }

    /// Reset the bus
    pub fn reset(&mut self, delay: &mut dyn DelayUs<u16>) -> Result<bool, Error<E>> {
        // Line must be high
        self.io_pin.set_high()?;
        // Test if line is high
        self.is_line_high(delay)?;

        // Give the reset pulse
        self.io_pin.set_low()?;

        // Pulse must be a length of min 480u
        delay.delay_us(480);

        // Release the line
        self.io_pin.set_high()?;

        // Check for the line for a divice present pulse

        let mut line = false;
        //Wait for the device to react to the rising edge
        delay.delay_us(60);

        // Read the input
        for _ in 0..6 {
            line |= self.io_pin.is_low()?;
            delay.delay_us(10);
        }

        Ok(line)
    }

    // Writes a bit to the line
    fn write_bit(&mut self, bit: bool, delay: &mut dyn DelayUs<u16>) -> Result<(), Error<E>> {
        // Set line to low
        self.io_pin.set_low()?;
        // Wait time depending on the bit value
        delay.delay_us(if bit { 10 } else { 65 });
        // Write to high again
        self.io_pin.set_high()?;
        // Wait until the Slot is over
        delay.delay_us(if bit { 70 } else { 5 });
        Ok(())
    }

    fn read_bit(&mut self, delay: &mut dyn DelayUs<u16>) -> Result<bool, Error<E>> {
        // Set line to low to start
        self.io_pin.set_low()?;
        // Wati at least 1 us
        delay.delay_us(2);
        // Try to set the line to high
        self.io_pin.set_high()?;
        //Wait until the end of the read window
        delay.delay_us(5);
        let bit = self.io_pin.is_high();
        delay.delay_us(53);
        Ok(bit?)
    }

    // Write a byte to the line
    fn write_byte(&mut self, mut byte: u8, delay: &mut dyn DelayUs<u16>) -> Result<(), Error<E>> {
        for _ in 0..8 {
            self.write_bit((byte & 0x01) == 0x01, delay)?;
            byte >>= 1;
        }
        Ok(())
    }

    // Reset and select a rom
    fn reset_and_select_rom(
        &mut self,
        rom_no: &[u8; 8],
        delay: &mut dyn DelayUs<u16>,
    ) -> Result<(), Error<E>> {
        self.reset(delay)?;

        self.write_byte(Command::SelectRom as u8, delay)?;

        self.write_bytes(rom_no, delay)?;

        Ok(())
    }

    // Read a byte from the line
    fn read_byte(&mut self, delay: &mut dyn DelayUs<u16>) -> Result<u8, Error<E>> {
        let mut byte = 0_u8;
        for _ in 0..8 {
            byte >>= 1;
            if self.read_bit(delay)? {
                byte |= 0x80;
            }
        }
        Ok(byte)
    }

    // Send bytes to the line
    pub fn write_bytes(
        &mut self,
        bytes: &[u8],
        delay: &mut dyn DelayUs<u16>,
    ) -> Result<(), Error<E>> {
        for byte in bytes {
            self.write_byte(*byte, delay)?;
        }
        Ok(())
    }

    // Recive bytes
    pub fn read_bytes(
        &mut self,
        bytes: &mut [u8],
        delay: &mut dyn DelayUs<u16>,
    ) -> Result<(), Error<E>> {
        for byte in bytes {
            *byte = self.read_byte(delay)?;
        }
        Ok(())
    }

    // Search for devices
    pub fn search(
        &mut self,
        search_state: &mut SearchState,
        delay: &mut dyn DelayUs<u16>,
    ) -> Result<Option<[u8; 8]>, Error<E>> {
        if search_state.status == SearchStatus::End {
            return Err(Error::SearchEnd);
        }

        // Reset and check is device is present
        if !self.reset(delay)? {
            search_state.status = SearchStatus::End;
            return Ok(None);
        };

        let mut last_zero: u8 = 0;

        // Send Search command
        self.write_byte(Command::Search as u8, delay)?;

        for id_bit_number in 1_u8..=64 {
            // Read two bits
            let id_bit = self.read_bit(delay)?;
            let comp_id_bit = self.read_bit(delay)?;

            // Eval the two recived bits
            let serach_direction = if id_bit && comp_id_bit {
                // No Device found
                search_state.status = SearchStatus::End;
                search_state.rom_no[0] = id_bit as u8;
                return Ok(Some(search_state.rom_no));
            } else if id_bit != comp_id_bit {
                // no discrepancy found
                id_bit
            } else {
                // a discrepancy found
                let direction = match id_bit_number.cmp(&search_state.last_discrepancy) {
                    Ordering::Equal => {
                        // We were at this position bevore. Now we choose the one path
                        true
                    }
                    Ordering::Greater => {
                        // a new discrepancy is found --> Choose the 0 path
                        false
                    }
                    Ordering::Less => {
                        // A discrepancy from a previous iteration is found just use the one from the rom code
                        is_bit_set(&search_state.rom_no, id_bit_number - 1)
                    }
                };
                if !direction {
                    last_zero = id_bit_number;
                }
                direction
            };
            self.write_bit(serach_direction, delay)?;

            // set id in rom_no
            set_bit(
                &mut search_state.rom_no,
                id_bit_number - 1,
                serach_direction,
            );
        }

        search_state.last_discrepancy = last_zero;

        // Check if it was the last device
        if search_state.last_discrepancy == 0 {
            search_state.status = SearchStatus::End;
        }

        // Check the crc
        Self::check_rom_crc(&search_state.rom_no)?;

        Ok(Some(search_state.rom_no))
    }

    fn is_line_high(&self, delay: &mut dyn DelayUs<u16>) -> Result<(), Error<E>> {
        for _ in 0..125 {
            if self.io_pin.is_high()? {
                return Ok(());
            } else {
                delay.delay_us(2);
            }
        }
        Err(Error::WireNotHigh)
    }

    fn check_rom_crc(add: &[u8; 8]) -> Result<(), Error<E>> {
        let crc_rec = add[7];
        let mut crc = crc_any::CRCu8::crc8maxim();
        crc.digest(&add[0..=6]);
        let crc_calc = crc.get_crc();
        if crc_calc == crc_rec {
            Ok(())
        } else {
            Err(Error::CrcError(crc_calc, crc_rec))
        }
    }
}

fn is_bit_set(array: &[u8], bit: u8) -> bool {
    if bit / 8 >= array.len() as u8 {
        return false;
    }
    let index = bit / 8;
    let offset = bit % 8;
    array[index as usize] & (0x01 << offset) != 0x00
}

fn set_bit(array: &mut [u8], bit: u8, value: bool) {
    if bit / 8 >= array.len() as u8 {
        return;
    }
    let index = bit / 8;
    let offset = bit % 8;
    if value {
        array[index as usize] |= 0x01 << offset
    } else {
        array[index as usize] &= !(0x01 << offset)
    }
}

pub struct Device {
    rom_no: [u8; 8],
}

impl Device {
    pub fn new(rom_no: [u8; 8]) -> Self {
        Self { rom_no }
    }
}

#[repr(u8)]
#[derive(PartialEq)]
pub enum Error<E: Sized> {
    /// Wire does is not pulled up by resistor. Maybe it is shortend
    WireNotHigh,
    /// An Error on the IO Port occured
    PortError(E),
    /// Search is at the end
    SearchEnd,
    /// CRC Value not ok
    CrcError(u8, u8),
    /// Some data does not make sense
    DataError,
}

impl<E: Sized> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Self::PortError(e)
    }
}
