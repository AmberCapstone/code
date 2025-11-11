#pragma once

#include "ov7670.hpp"

using namespace amber;

namespace twi {

typedef enum : uint8_t {
    OK = 0,
    ERROR = 1
} status_e;

void Initialize(void);

// Blocking
status_e WriteRegister(ov7670::reg reg, uint8_t data);

// Blocking
// Implicit return
status_e ReadRegister(ov7670::reg reg, uint8_t* out);

}  // namespace twi
