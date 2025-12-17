#pragma once

#include "spi_flash.pb.h"

// CubeMX
#include "adc.h"

namespace sensors {

// Behaviour
void Init(void);
void Update_10hz(void);

// Accessors
int32_t GetTemperatureC(void);
int32_t GetVrefintMv(void);
int32_t GetVbatMv(void);
void PopulateStatus(spi_flash_status_t* msg);

}  // namespace sensors