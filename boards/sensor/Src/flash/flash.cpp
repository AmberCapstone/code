#include "flash.hpp"

#include <optional>

#include "m25pe.hpp"
#include "stm_spi.hpp"

// Proto
#include "flash.pb.h"

// CubeMX
#include "crc.h"
#include "gpio.h"
#include "spi.h"

namespace flash {

static uint32_t request_number = 0;
static sensor_flash_state_t state = SENSOR_FLASH_STATE_UNKNOWN;
static std::optional<sensor_flash_state_t> pending_transition = std::nullopt;
static uint32_t state_elapsed_ms = 0;

// Readout
static uint32_t readout_req_number = 0;

static SPI m25_spi(&hspi3, FLASH_CSn_GPIO_Port, FLASH_CSn_Pin);

static sensor_flash_page_t next_page = SENSOR_FLASH_PAGE_INIT_ZERO;
static sensor_flash_page_t readout_page = SENSOR_FLASH_PAGE_INIT_ZERO;

static bool IsPagePopulated(sensor_flash_page_t* page) {
    // Doesn't verify CRC of data == CRC field
    return page->has_crc && page->has_data && page->has_page_number;
}

static uint32_t ComputePageCRC(sensor_flash_page_t* page) {
    HAL_CRC_Calculate(&hcrc, &page->page_number, 4);
    return HAL_CRC_Accumulate(&hcrc, (uint32_t*)page->data, 256) ^ 0xFFFFFFFF;
}

static void Transition(sensor_flash_state_t new_state) {
    pending_transition = new_state;
}

void Init(void) {
    Transition(SENSOR_FLASH_STATE_IDLE);

    HAL_GPIO_WritePin(FLASH_RESETn_GPIO_Port, FLASH_RESETn_Pin, GPIO_PIN_SET);
    HAL_GPIO_WritePin(FLASH_CSn_GPIO_Port, FLASH_CSn_Pin, GPIO_PIN_SET);
}

void Start(void) {
    state = SENSOR_FLASH_STATE_UNKNOWN;
    Transition(SENSOR_FLASH_STATE_ERASING);
}

void StartReadout(void) {
    state = SENSOR_FLASH_STATE_UNKNOWN;
    Transition(SENSOR_FLASH_STATE_READOUT);
}

void Update_1khz(void) {
    bool on_enter = false;

    if (pending_transition.has_value()) {
        state = *pending_transition;
        pending_transition.reset();
        on_enter = true;
        state_elapsed_ms = 0;
    }

    switch (state) {
        case SENSOR_FLASH_STATE_UNKNOWN:
            // should never happen
            break;

        case SENSOR_FLASH_STATE_IDLE:
            break;

        case SENSOR_FLASH_STATE_ERASING:
            if (on_enter) {
                amber::m25pe::EnableWriting(m25_spi);
                amber::m25pe::BulkErase(m25_spi);
            }

            if (!amber::m25pe::IsWriteInProgress(m25_spi)) {
                Transition(SENSOR_FLASH_STATE_PROGRAMMING);
            }
            break;

        case SENSOR_FLASH_STATE_PROGRAMMING: {
            static bool writing;

            if (on_enter) {
                request_number = 0;
                writing = false;
            }

            if (!writing) {
                if (!IsPagePopulated(&next_page)) {
                    // Reject incomplete / absent packets
                    break;
                }

                if (next_page.page_number != request_number) {
                    // Reject out-of-sequence packets;
                    break;
                }

                uint32_t crc = ComputePageCRC(&next_page);
                if (crc != next_page.crc) {
                    break;  // Reject corrupted packets
                }

                amber::m25pe::EnableWriting(m25_spi);
                amber::m25pe::PageProgram(m25_spi,
                                          next_page.page_number * PAGE_SIZE,
                                          next_page.data, PAGE_SIZE);
                writing = true;
            } else {
                // clear to avoid reprocessing
                next_page = SENSOR_FLASH_PAGE_INIT_ZERO;

                if (!amber::m25pe::IsWriteInProgress(m25_spi)) {
                    writing = false;
                    request_number++;
                }
            }

            if (request_number >= flash::NUM_PAGES) {
                Transition(SENSOR_FLASH_STATE_DONE);
            }
        } break;

        case SENSOR_FLASH_STATE_READOUT: {
            static uint32_t last_page_loaded;

            if (on_enter) {
                last_page_loaded = UINT32_MAX;

                readout_req_number = 0;
                readout_page.page_number = 0;
            }

            readout_page.has_page_number = true;
            if (readout_req_number > readout_page.page_number) {
                readout_page.page_number = readout_req_number;
            }

            if (last_page_loaded != readout_page.page_number) {
                readout_page.has_data = true;
                amber::m25pe::ReadData(m25_spi,
                                       readout_page.page_number * PAGE_SIZE,
                                       PAGE_SIZE, readout_page.data);

                readout_page.has_crc = true;
                readout_page.crc = ComputePageCRC(&readout_page);

                last_page_loaded = readout_page.page_number;
            }

            if (readout_req_number == NUM_PAGES) {
                Transition(SENSOR_FLASH_STATE_DONE);
            }
        } break;

        case SENSOR_FLASH_STATE_DONE:
            break;
    }

    state_elapsed_ms++;
}

void ReceivePage(sensor_flash_page_t* page) {
    next_page = *page;
}

// Accessors
bool IsDone(void) {
    return state == SENSOR_FLASH_STATE_DONE;
}

void PopulateStatus(sensor_flash_status_t* msg) {
    msg->has_state = true;
    msg->state = state;

    switch (state) {
        case SENSOR_FLASH_STATE_PROGRAMMING:
            msg->has_stm_page_request = true;
            msg->stm_page_request = request_number;
            break;

        case SENSOR_FLASH_STATE_READOUT:
            msg->has_readout_page = true;
            msg->readout_page = readout_page;
            break;

        default:
            break;
    }
}

void UpdateReadoutReqNumber(uint32_t req_number) {
    readout_req_number = req_number;
}

}  // namespace flash