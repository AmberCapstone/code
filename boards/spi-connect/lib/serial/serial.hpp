#pragma once

#include <stdint.h>

namespace serial {

constexpr uint16_t MAX_LENGTH = 512;

bool IsBusy(void);

void Initialize(void);

void Transmit(const uint8_t* buffer, uint16_t length);

}  // namespace serial