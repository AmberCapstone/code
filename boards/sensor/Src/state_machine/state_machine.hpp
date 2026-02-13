#pragma once

#include "sensor.pb.h"

namespace state_machine {

// Behaviour
void Init(void);
void Update_1khz(void);

// Accessors
sensor_state_t GetState(void);
void PopulateStatus(sensor_status_t* msg);

// Modifiers
void HandleAction(sensor_action_t action);

}  // namespace state_machine