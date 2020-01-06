/** In this file the configuration for the program is done.
 * Also some compiler pragmas and defs are done here
 */


// Select the Target Board for Emo Mode
#define DEMO_MODE


#ifndef DEMO_MODE
	#include <Controllino.h>
#endif

 #ifndef CONFIG_H
 #define CONFIG_H

/**
 * Configuration of the HW
 *
 * 0  - RXD               - USB Serial
 * 1  - TXD               - USB Serial
 * 2  - IN0               - One Wire Bus Data (DS1820)
 * 3  - IN1
 * 4  - D0 - Relais/DO    - Pump Buffer
 * 5  - D1 - Relais/DO    - Magnetvalve Buffer
 * 6  - D2 - Relais/DO    - Inhibit Burner
 * 7  - D3 - Relais/DO
 * 8  - D4 - Relais/DO
 * 9  - D5 - Relais/DO
 * 10 - RTC SS            - SPI RTC
 * 11 - MOSI              - SPI RTC
 * 12 - MISO              - SPI RTC
 * 13 - SCK               - SPI RTC
 * A0 - A0 - AI/DI        - Pump warm Water
 * A1 - A1 - AI/DI        - Pump Heating
 * A2 - A2 - AI/DI        - Temperature Contact Wood
 * A3 - A3 - AI/DI        - Request Burner
 * A4 - D6/SDA - DO       - I2C
 * A5 - D7/SCL - DO       - I2C
 * A6 - A4 - AI
 * A7 - A5 - AI
 *
 */

//Project Information
#define PRG_VERSION "1.0.0.0"
#define PRG_AUTHOR "M. Brinkmann"
#define PRG_TITLE "KeepItWarm"

 //Global configuration
 #define TEMP_MINIMUM 60.0
 #define PUMP_DURATION 60000
 #define PUMP_PAUSE 60000
 #define ENABLE_SERIAL_PRINT

 //Definition according to the IO mapping above
#ifndef DEMO_MODE
	#define IO_OneWire CONTROLLINO_IN0

	#define O_MVBUFFER CONTROLLINO_D1
	#define O_PUMPBUFFER CONTROLLINO_D0

 	#define O_INHIBITBURNER CONTROLLINO_D2

 	#define I_PUMPWARMWATERACTIVE CONTROLLINO_A0
 	#define I_PUMPHEATINGACTIVE CONTROLLINO_A1
 	#define I_BURNERREQUEST CONTROLLINO_A2
#else
	#define IO_OneWire 13

	#define O_MVBUFFER 5 
	#define O_PUMPBUFFER 4 

 	#define O_INHIBITBURNER 6 

 	#define I_PUMPWARMWATERACTIVE A0
 	#define I_PUMPHEATINGACTIVE A1 
 	#define I_BURNERREQUEST A2 
#endif

//Onewire Definitions
#define TEMPERATURE_PRECISION 12
#define TEMPERATURE_UPDATE 10000
#define TEMPCONVERSION_TIME 1000
#define ADD_WARMWATER {0x28, 0xff, 0x4b, 0x96, 0x74, 0x16, 0x04, 0x6f}
#define ADD_BUFFERTOP {0x28, 0xff, 0x4b, 0x96, 0x74, 0x16, 0x04, 0x6f}
#define ADD_BUFFERBOTTOM {0x28, 0xff, 0x2f, 0x96, 0x74, 0x16, 0x04, 0x61}
#define ADD_HEAT_FLOW {0x28, 0xff, 0x4b, 0x96, 0x74, 0x16, 0x04, 0x6f}
#define ADD_HEAT_RETURN {0x28, 0xff, 0x4b, 0x96, 0x74, 0x16, 0x04, 0x6f}

//I2C Definitions for Master Mode
#define I2C_ADDRESS 42


#endif /* end of include guard:  */
