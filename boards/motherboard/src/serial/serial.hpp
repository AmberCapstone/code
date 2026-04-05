#pragma once

#include <stdint.h>

namespace serial {

void Init(void);
void Receive(void);

void Update_100hz(void);

extern "C" {
void SerialReceiveBytes(uint8_t* bytes, uint32_t len);
}

}  // namespace serial
