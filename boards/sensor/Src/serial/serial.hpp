#pragma once

#include "sensor.pb.h"

namespace serial {

// Behaviour
void Init(void);
void Receive(void);

void Update_10hz(void);
void Update_100hz(void);

// Modifiers
extern "C" {
void SerialReceiveBytes(uint8_t* bytes, uint32_t len);
}

}  // namespace serial
