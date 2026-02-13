#pragma once

#include "m25pe.hpp"
#include "sensor.pb.h"

namespace flash {

constexpr uint32_t PAGE_SIZE = amber::m25pe::PAGE_SIZE;
constexpr uint32_t NUM_PAGES =
    amber::m25pe::TOTAL_SIZE / amber::m25pe::PAGE_SIZE;

// Control
void Init(void);
void Start(void);
void StartReadout(void);
void Update_1khz(void);

void ReceivePage(sensor_flash_page_t* page);
void UpdateReadoutReqNumber(uint32_t req_number);

// Accessors
bool IsDone(void);
void PopulateStatus(sensor_flash_status_t* msg);

}  // namespace flash