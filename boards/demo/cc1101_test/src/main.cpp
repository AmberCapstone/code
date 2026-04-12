/*
 * Wiring (CC1101 -> Arduino):
 * VCC  -> 3.3V
 * GND  -> GND
 * MOSI -> Pin 11
 * MISO -> Pin 12
 * SCK  -> Pin 13
 * CSN  -> Pin 10
 * GDO0 -> Not connected
 * GDO2 -> Not connected
 */

#include <Arduino.h>
#include <SPI.h>

#include "cc1101.hpp"
#include "arduino_digital.hpp"

void setup() {
  pin::ArduinoDigitalOutput cs_tx(10);
  pin::ArduinoDigitalOutput cs_rx(9);
  const pin::ArduinoDigitalInput miso(12);

  SPI.begin();
  SPI.beginTransaction(SPISettings(4000000, MSBFIRST, SPI_MODE0));

  cs_tx.setHigh();
  cs_rx.setHigh();

  amber::cc1101::Driver transmitter(SPI, miso, cs_tx);
  amber::cc1101::Driver receiver(SPI, miso, cs_rx);

  transmitter.reset();
  transmitter.configure(amber::cc1101::Driver::Frequency::MHZ_915);
  transmitter.begin(amber::cc1101::Driver::Direction::TX);

  receiver.reset();
  receiver.configure(amber::cc1101::Driver::Frequency::MHZ_915);
  receiver.begin(amber::cc1101::Driver::Direction::TX);
}

void loop() {
  delay(1000);
}
