#pragma once

#include <stdint.h>

namespace serial {

constexpr uint16_t MAX_LENGTH = 512;

void Initialize(void);

void Transmit(uint8_t* buffer, uint16_t length);

}  // namespace serial