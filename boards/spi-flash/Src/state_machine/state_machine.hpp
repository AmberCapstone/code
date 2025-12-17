#pragma once

#include "spi_flash.pb.h"

namespace state_machine {

// Behaviour
void Init(void);
void Update_1khz(void);

// Accessors
void PopulateStatus(spi_flash_status_t* msg);

// Modifiers
void HandleAction(spi_flash_action_t action);

}  // namespace state_machine