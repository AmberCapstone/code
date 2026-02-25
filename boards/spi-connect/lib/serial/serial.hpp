#pragma once

#include <stdint.h>

namespace serial {

constexpr uint16_t MAX_LENGTH = 256;

void Initialize(void);

bool IsBusy(void);
bool PacketReady(void);

void Transmit(const uint8_t* buffer, uint16_t length);

uint16_t ReadPacket(uint8_t* out); 

}  // namespace serial