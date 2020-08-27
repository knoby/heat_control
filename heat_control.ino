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

enum States{
  Init,
  BufferOff,
  BufferOn,
  ActivatePump
};

struct HeatState {
  States eStep;
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
	unsigned long last_send = millis()-10000;

	while(true){	

		// Program Logic run with two second delay
		if ((millis() - last_update) > 2000) {
			get_input();
			update_state();
			set_output();
			last_update = millis();
		}

		// Communication sends data every minute
		if ((millis() - last_send) > 10000) {
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
    lcd.print("PRG_VERSION");
}


void init_tempsensors() {
	//===============================
	// INIT One Wire Bus / Temperatur Sensors 
	//===============================

	// Start Lib
	tempSensors.begin();

	//Set Resoultion of the Sensros
	//tempSensors.setResolution(sensorAdd[0], TEMPERATURE_PRECISION);
	tempSensors.setResolution(sensorAdd[1], TEMPERATURE_PRECISION);
	tempSensors.setResolution(sensorAdd[2], TEMPERATURE_PRECISION);
	tempSensors.setResolution(sensorAdd[3], TEMPERATURE_PRECISION);
	//tempSensors.setResolution(sensorAdd[4], TEMPERATURE_PRECISION);
	

	//Set Wait for conversion on --> Halt the program
	tempSensors.setWaitForConversion(true);
}

////////////////////////////////////////////////////////////////////////
// Logic Functions
////////////////////////////////////////////////////////////////////////

void update_state(){

  static States eStepOld = Init;
  static unsigned long time_enter = millis();

  // Handle Buffer not On --> Return To Start
  if(heat_state.f16TempBufferTop<(TEMP_MINIMUM-5.0)){
    heat_state.eStep = Init;
  }

  // Handling Step Change
  bool xNew = !(heat_state.eStep == eStepOld);
  
  // Safe the Time on entering State.
  if(xNew) {
    time_enter = millis();
  }
  
  // Safe State
  eStepOld = heat_state.eStep;
   

  // State Machine
  switch(heat_state.eStep) {

    // Init Step --> Go to BufferOff
    case Init:
      if(xNew) {
        heat_state.xInhibitBurner = false;
        heat_state.xPumpBuffer = false;
        heat_state.xMvBuffer = false;
      }
      heat_state.eStep = BufferOff;
      break;

      
    // Wait until Buffer Temp is high enough and the burner is not running
    case BufferOff:
    if(xNew) {
      heat_state.xInhibitBurner = false;
      heat_state.xPumpBuffer = false;
      heat_state.xMvBuffer = false;
    }
    if((heat_state.f16TempBufferTop>=TEMP_MINIMUM) && (!heat_state.xBurnerRequest)) {
      heat_state.eStep =  BufferOn;
    }
    break;


    // Wait for Request of Burner
    case BufferOn:
      if(xNew) {
        heat_state.xInhibitBurner = true;
        heat_state.xPumpBuffer = false;
        heat_state.xMvBuffer = true;      
      }
      if((heat_state.xBurnerRequest) && ((millis()-time_enter)>PUMP_PAUSE)) {
        heat_state.eStep = ActivatePump;
      }
      break;


    // Pulse the Pump on the Buffer
    case ActivatePump:
      if(xNew) {
        heat_state.xInhibitBurner = true;
        heat_state.xPumpBuffer = true;
        heat_state.xMvBuffer = true;   
      }
      if((millis()-time_enter)>=PUMP_DURATION) { // Switch off after Pulse Duration
        heat_state.eStep = BufferOn;
      } 
      if(!heat_state.xBurnerRequest) { // Switch off if no request for heat
        heat_state.eStep = BufferOn;
      }
      break;
  }



  // For Emergency Mode alway inhibit burner.
  heat_state.xInhibitBurner = true;
  

}


////////////////////////////////////////////////////////////////////////
// IO Functions 
////////////////////////////////////////////////////////////////////////

void get_input() {
	heat_state.xPumpHeatingActive = digitalRead(I_PUMPHEATINGACTIVE);
	heat_state.xPumpWarmWaterActive = digitalRead(I_PUMPWARMWATERACTIVE);
	heat_state.xBurnerRequest = digitalRead(I_BURNERREQUEST);


  	tempSensors.requestTemperatures();
 	//heat_state.f16TempWarmWater = tempSensors.getTempC(sensorAdd[0]);
 	heat_state.f16TempBufferTop = tempSensors.getTempC(sensorAdd[1]);
 	heat_state.f16TempBufferButtom = tempSensors.getTempC(sensorAdd[2]);
 	heat_state.f16TempFlow = tempSensors.getTempC(sensorAdd[3]);
 	//heat_state.f16TempReturn = tempSensors.getTempC(sensorAdd[4]);
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
  switch(heat_state.eStep) {
    case Init:
      Serial.println("State: Init");
      break;
    case BufferOff:
      Serial.println("State: BufferOff");
      break;
    case BufferOn:
      Serial.println("State: BufferOn");
      break;
    case ActivatePump:
      Serial.println("State: ActivatePump");
      break;
  }
  Serial.println("END");

}
