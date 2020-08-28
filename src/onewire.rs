use atmega328p_hal::prelude::*;
use core::cmp::Ordering;
use embedded_hal as hal;
use embedded_hal::digital::v2::*;
use hal::blocking::delay::DelayUs;
use num_format::{Buffer, Locale};

pub enum Command {
    Search,
}

impl From<Command> for u8 {
    fn from(cmd: Command) -> Self {
        use Command::*;
        match cmd {
            Search => 0xF0,
        }
    }
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
        self.io_pin.set_high();

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
        self.io_pin.set_high();
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

    // Read a byte from the line
    fn read_byte(&mut self, delay: &mut dyn DelayUs<u16>) -> Result<u8, Error<E>> {
        let mut byte = 0_u8;
        for _ in 0..8 {
            byte >>= 1;
            if self.reset(delay)? {
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
        delay: &mut dyn DelayUs<u16>,
        serial: &mut atmega328p_hal::usart::WriteUsart0<crate::Clock>,
    ) -> Result<Option<bool>, Error<E>> {
        serial.write_str("Start One Wire search ...\n");

        // Search State
        let mut last_discrepancy = 0;
        let mut rom_no = [0_u8; 8];

        loop {
            // Reset and check is device is present
            serial.write_str("Reseting the bus\n");
            if !self.reset(delay)? {
                serial.write_str("No device on bus\n");
                return Ok(None);
            };

            let mut last_zero: u8 = 0;

            // Send Search command
            serial.write_str("Sending Search Command\n");
            self.write_byte(Command::Search.into(), delay)?;

            for id_bit_number in 1_u8..=64 {
                // Read two bits
                let id_bit = self.read_bit(delay)?;
                let comp_id_bit = self.read_bit(delay)?;

                // Eval the two recived bits
                let serach_direction = if id_bit && comp_id_bit {
                    // No Device found
                    return Ok(None);
                } else if id_bit != comp_id_bit {
                    // no discrepancy found
                    id_bit
                } else {
                    // a discrepancy found
                    let direction = match id_bit_number.cmp(&last_discrepancy) {
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
                            is_bit_set(&rom_no, id_bit_number)
                        }
                    };
                    if !direction {
                        last_zero = id_bit_number;
                    }
                    direction
                };
                self.write_bit(serach_direction, delay)?;

                // set id in rom_no
                set_bit(&mut rom_no, id_bit_number, serach_direction);
            }

            last_discrepancy = last_zero;

            serial.write_str("Rom number found: [");
            let mut buffer = Buffer::default();
            for byte in rom_no.iter() {
                buffer.write_formatted(byte, &Locale::de);
                serial.write_str(buffer.as_str());
                serial.write_str(", ");
            }
            serial.write_str("]\n");
            if last_discrepancy == 0 {
                serial.write_str("Done!");
                break;
            }
        }

        Ok(None)
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

pub struct DS18B20 {}

impl DS18B20 {
    pub fn new() -> Self {
        Self {}
    }
}

pub enum Error<E: Sized> {
    /// Wire does is not pulled up by resistor. Maybe it is shortend
    WireNotHigh,
    /// An Error on the IO Port occured
    PortError(E),
}

impl<E: Sized> From<E> for Error<E> {
    fn from(e: E) -> Self {
        Self::PortError(e)
    }
}
