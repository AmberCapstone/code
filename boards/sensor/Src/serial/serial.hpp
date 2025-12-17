#pragma once

#include "sensor.pb.h"

namespace serial {

// Behaviour
void Init(void);
void Receive(void);
void SendStatus(void);

// Modifiers
extern "C" {
void SerialReceiveBytes(uint8_t* bytes, uint32_t len);
}

}  // namespace serial
