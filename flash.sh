avrdude  -patmega328p -carduino -P/dev/ttyUSB0 -b57600 -D -Uflash:w:target/avr-atmega328p/release/heat_control.elf:e