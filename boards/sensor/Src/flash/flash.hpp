#pragma once

#include "m25pe.hpp"
#include "sensor.pb.h"

namespace flash {

constexpr uint32_t PAGE_SIZE = amber::m25pe::PAGE_SIZE;

// Control
void Init(void);
void Start(void);

// Accessors
sensor_flash_state_t GetState(void);
uint32_t GetRequestNumber(void);

}  // namespace flash