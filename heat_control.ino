#include "config.hpp"
#include <OneWire.h>
#include <DallasTemperature.h>
#include <Wire.h>
#include <LiquidCrystal_I2C.h>


// Setup a oneWire instance to communicate with any OneWire devices  
OneWire oneWire(IO_OneWire); 
// Pass our oneWire reference to Dallas Temperature. 
DallasTemperature tempSensors(&oneWire);
// set the LCD address to 0x27 for a 16 chars and 2 line display
LiquidCrystal_I2C lcd(0x27,17,2);  

struct HeatState {
	bool xMVBuffer;
	bool xPumpBuffer;
	bool xInhibitBurner;
	bool xPumpHeatingActive;
	bool xPumpWarmWaterActive;
	bool xBurnerRequest;
	float f16TempBufferTop;
	float f16TempBufferButtom;
	float f16TempWarmWater;
	float f16TempFlow;
	float f16TempReturn;
};
// Global Struct holding the entire state
HeatState heat_state;

// Address of the temp sensors
DeviceAddress sensorAdd[] = {ADD_WARMWATER,
                        ADD_BUFFERTOP,
                        ADD_BUFFERBOTTOM,
                        ADD_HEAT_FLOW,
                        ADD_HEAT_RETURN};

// Setup Function
void setup() {
	init_digital_IO();
	init_tempsensors();
	init_display();

}

// Main Loop
void loop() {

	unsigned long last_update = millis();
	unsigned long last_send = millis();


	while(true){	

		// Program Logic
		if ((millis() - last_update) > 1000) {
			get_input();
			update_state();
			set_output();
			last_update = millis();
		}

		// Communication
		if ((millis() - last_send) > 60000) {
			send_serial();
			last_send = millis();
		}

	}
}




////////////////////////////////////////////////////////////////////////
// INIT Functions
////////////////////////////////////////////////////////////////////////

void init_digital_IO() {
	//===============================
	// INIT DIGITAL IO	
	//===============================

	//Setup the Outputs
	pinMode(O_MVBUFFER, OUTPUT);
	pinMode(O_PUMPBUFFER, OUTPUT);
	pinMode(O_INHIBITBURNER, OUTPUT);


	//Setupb the inputs
	pinMode(I_PUMPWARMWATERACTIVE, INPUT);
	pinMode(I_PUMPHEATINGACTIVE, INPUT);
	pinMode(I_BURNERREQUEST, INPUT);
}


void init_display() {
	//===============================
	// INIT I2C Display
	//===============================
    lcd.init();   // initialize the lcd
    lcd.backlight(); // turn backlight on
    // Welcom Message
    lcd.setCursor(0, 0);
    lcd.print("Heat Control ");
    lcd.setCursor(0,1);
    lcd.print("0.0.1");
}


void init_tempsensors() {
	//===============================
	// INIT One Wire Bus / Temperatur Sensors 
	//===============================

	// Start Lib
	tempSensors.begin();

	//Set Resoultion of the Sensros
	tempSensors.setResolution(sensorAdd[0], TEMPERATURE_PRECISION);
	tempSensors.setResolution(sensorAdd[1], TEMPERATURE_PRECISION);
	tempSensors.setResolution(sensorAdd[2], TEMPERATURE_PRECISION);
	tempSensors.setResolution(sensorAdd[3], TEMPERATURE_PRECISION);
	tempSensors.setResolution(sensorAdd[4], TEMPERATURE_PRECISION);
	

	//Set Wait for conversion on --> Halt the program
	tempSensors.setWaitForConversion(true);
}

////////////////////////////////////////////////////////////////////////
// Logic Functions
////////////////////////////////////////////////////////////////////////

enum StateMachine
{
	BufferOff,
	BufferActive,
	PumpWater,
};

StateMachine state = BufferOff;
unsigned long pump_start;
unsigned long pump_end;

void update_state() {
	switch (state) {
		case BufferOff:
			if ((heat_state.f16TempBufferTop >= TEMP_MINIMUM) 
					&& heat_state.xPumpHeatingActive 
					&& !heat_state.xPumpWarmWaterActive) {
				state = BufferActive;
			}			
			break;

		case BufferActive:
			if ((heat_state.xPumpWarmWaterActive)  // Warm Water Pump is active 
				|| (!heat_state.xPumpHeatingActive and heat_state.xBurnerRequest)  // Only Burner && no Pump
				|| (heat_state.f16TempBufferTop < (TEMP_MINIMUM-5.0))) { // Temperatur 5K below switch on temp
				state = BufferOff;
			}
			if (heat_state.xBurnerRequest && ((millis()-pump_end) > PUMP_PAUSE)) { // Temperature to low
				state = PumpWater;
				pump_start = millis();
			}
			break;

		case PumpWater:
			if ((millis() - pump_start) > PUMP_DURATION) {
				state = BufferActive;
				pump_end = millis();
			}
			if ((heat_state.xPumpWarmWaterActive)  // Warm Water Pump is active 
				|| (!heat_state.xPumpHeatingActive and heat_state.xBurnerRequest)  // Only Burner && no Pump
				|| (heat_state.f16TempBufferTop < (TEMP_MINIMUM-5.0))) { // Temperatur 5K below switch on temp
				state = BufferOff;
			}
			break;
	}	

	switch (state) {
	    case BufferOff:
	    	heat_state.xInhibitBurner = false;
	    	heat_state.xMVBuffer = false;
	    	heat_state.xPumpBuffer = false;
	    	break;
	    case BufferActive:	   	    
	    	heat_state.xInhibitBurner = true;
	    	heat_state.xMVBuffer = true;
	    	heat_state.xPumpBuffer = false;		
	    	break;
	    case PumpWater:	    
	    	heat_state.xInhibitBurner = true;
	    	heat_state.xMVBuffer = true;
	    	heat_state.xPumpBuffer = true;
	    	break;
	    default:
	    	heat_state.xInhibitBurner = false;
	    	heat_state.xMVBuffer = false;
	    	heat_state.xPumpBuffer = false;  
	}	
}


////////////////////////////////////////////////////////////////////////
// IO Functions 
////////////////////////////////////////////////////////////////////////

void get_input() {
	heat_state.xPumpHeatingActive = digitalRead(I_PUMPHEATINGACTIVE);
	heat_state.xPumpWarmWaterActive = digitalRead(I_PUMPWARMWATERACTIVE);
	heat_state.xBurnerRequest = digitalRead(I_BURNERREQUEST);


  	tempSensors.requestTemperatures();
 	heat_state.f16TempWarmWater = tempSensors.getTemp(sensorAdd[0]);
 	heat_state.f16TempBufferTop = tempSensors.getTemp(sensorAdd[1]);
 	heat_state.f16TempBufferButtom = tempSensors.getTemp(sensorAdd[2]);
 	heat_state.f16TempFlow = tempSensors.getTemp(sensorAdd[3]);
 	heat_state.f16TempReturn = tempSensors.getTemp(sensorAdd[4]);
}

void set_output() {
	digitalWrite(O_INHIBITBURNER, heat_state.xInhibitBurner);
	digitalWrite(O_PUMPBUFFER, heat_state.xPumpBuffer);
	digitalWrite(O_MVBUFFER, heat_state.xMVBuffer);
}

////////////////////////////////////////////////////////////////////////
// Send state to serial
////////////////////////////////////////////////////////////////////////