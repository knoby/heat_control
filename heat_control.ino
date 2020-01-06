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
	bool xMvBuffer;       
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

	Serial.begin(9600);

}

// Main Loop
void loop() {

	unsigned long last_update = millis()-1000;
	unsigned long last_send = millis()-60000;

	while(true){	

		// Program Logic run with one second delay
		if ((millis() - last_update) > 1000) {
			get_input();
			update_state();
			set_output();
			last_update = millis();
		}

		// Communication sends data every minute
		if ((millis() - last_send) > 60000) {
			send_state();
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
    lcd.print("Heat Control");
    lcd.setCursor(0,1);
    lcd.print("0.1.0");
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

void update_state(){

	
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
	digitalWrite(O_MVBUFFER, heat_state.xMvBuffer);
}

////////////////////////////////////////////////////////////////////////
// Send state to serial
////////////////////////////////////////////////////////////////////////

void send_state() {

  Serial.println("===============================");	
  Serial.print("magnet_valve:");
  Serial.println(heat_state.xMvBuffer);
  Serial.print("burner_inhibit:");
  Serial.println(heat_state.xInhibitBurner);
  Serial.print("buffer_pump:");
  Serial.println(heat_state.xPumpBuffer);
  Serial.print("warm_water_pump:");
  Serial.println(heat_state.xPumpWarmWaterActive);
  Serial.print("heat_pump:");
  Serial.println(heat_state.xPumpHeatingActive);
  Serial.print("burner_request:");
  Serial.println(heat_state.xBurnerRequest);
  if (heat_state.f16TempBufferTop > -7040.0) {
    Serial.print("temp_buffer_top:");
    Serial.println(heat_state.f16TempBufferTop);
  }
  if (heat_state.f16TempBufferTop > -7040.0) {
    Serial.print("temp_buffer_buttom:");
    Serial.println(heat_state.f16TempBufferButtom);
  }
  if (heat_state.f16TempBufferTop > -7040.0) {
    Serial.print("temp_warm_water:");
    Serial.println(heat_state.f16TempWarmWater);
  }
  if (heat_state.f16TempBufferTop > -7040.0) {
    Serial.print("temp_heat_flow:");
    Serial.println(heat_state.f16TempFlow);
  }
  if (heat_state.f16TempBufferTop > -7040.0) {
    Serial.print("temp_heat_return:");
    Serial.println(heat_state.f16TempReturn);
  }
}


