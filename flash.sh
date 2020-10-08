#!/bin/bash

if [[ -e "/dev/ttyUSB0" ]]; then
    echo "Arduino"
    avrdude  -patmega328p -carduino -P/dev/ttyUSB0 -b57600 -D -Uflash:w:target/avr-atmega328p/release/heat_control.elf:e
    picocom -b 9600 --imap lfcrlf /dev/ttyUSB0
elif [[ -e "/dev/ttyACM0" ]]; then
    echo "Controllino"
    avrdude  -patmega328p -carduino -P/dev/ttyACM0 -b115200 -D -Uflash:w:target/avr-atmega328p/release/heat_control.elf:e
    picocom -b 9600 --imap lfcrlf /dev/ttyACM0
else
    echo "QEMU"
    qemu-system-avr -machine uno -bios target/avr-atmega328p/release/heat_control.elf -serial stdio -s -S
    #qemu-system-avr -machine uno -bios target/avr-atmega328p/release/heat_control.elf -serial stdio 
fi
