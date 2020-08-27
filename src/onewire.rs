use embedded_hal as hal;
use embedded_hal::digital::v2::*;
use hal::blocking::delay::DelayUs;

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

        Ok(true)
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
