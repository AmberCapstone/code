#pragma once

#include "digital.hpp"
#include <Arduino.h>

namespace pin {

struct ArduinoDigitalOutput : public DigitalOutput {

    explicit ArduinoDigitalOutput(uint8_t pin) : _pin(pin) {
        pinMode(pin, OUTPUT);
    }

    void setHigh() override { digitalWrite(_pin, HIGH); }
    void setLow()  override { digitalWrite(_pin, LOW); }

private:
    uint8_t _pin;
};

struct ArduinoDigitalInput : public DigitalInput {

    explicit ArduinoDigitalInput(uint8_t pin) : _pin(pin) {
        pinMode(pin, INPUT);
    }

    bool read() const override { return digitalRead(_pin); }

private:
    uint8_t _pin;
};

} // namespace pin
