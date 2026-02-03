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

static int32_t request_number = 0;
static sensor_flash_state_t state = SENSOR_FLASH_STATE_UNKNOWN;
static std::optional<sensor_flash_state_t> pending_transition = std::nullopt;
static uint32_t state_elapsed_ms = 0;

// Readout / verify
static int32_t sequence_number = 0;
static int32_t readout_req_number = 0;
static uint32_t readout_crc = 0;

static uint32_t id = 0;

static SPI m25_spi(&hspi3, FLASH_CSn_GPIO_Port, FLASH_CSn_Pin);

static uint32_t last_crc = 0;

static bool writing = false;

static sensor_flash_page_t next_page = SENSOR_FLASH_PAGE_INIT_ZERO;
static sensor_flash_page_t readout_page = SENSOR_FLASH_PAGE_INIT_ZERO;

uint8_t page_buf[256] = {0};

static void Transition(sensor_flash_state_t new_state) {
    pending_transition = new_state;
}

void Init(void) {
    Transition(SENSOR_FLASH_STATE_IDLE);

    HAL_GPIO_WritePin(FLASH_RESETn_GPIO_Port, FLASH_RESETn_Pin, GPIO_PIN_SET);
    HAL_GPIO_WritePin(FLASH_CSn_GPIO_Port, FLASH_CSn_Pin, GPIO_PIN_SET);

    id = amber::m25pe::ReadIdentification(m25_spi);
}

void Start(void) {
    state = SENSOR_FLASH_STATE_UNKNOWN;
    Transition(SENSOR_FLASH_STATE_ERASING);
}

void Update_1khz(void) {
    bool on_enter = false;

    if (pending_transition.has_value()) {
        on_enter = true;
        state = *pending_transition;
        state_elapsed_ms = 0;
        pending_transition.reset();
    }

    switch (state) {
        case SENSOR_FLASH_STATE_UNKNOWN:
            // should never happen
            break;

        case SENSOR_FLASH_STATE_IDLE:
            if (on_enter) {
                request_number = -1;
            }
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

        case SENSOR_FLASH_STATE_PROGRAMMING:
            if (on_enter) {
                request_number = 0;
                writing = false;
            }

            if (!writing) {
                if (!next_page.has_page_number || !next_page.has_data ||
                    !next_page.has_crc) {
                    // Reject incomplete / absent packets
                    break;
                }

                if (next_page.page_number != request_number) {
                    // Reject out-of-sequence packets;
                    break;
                }

                // cast is safe since page_number should always be positive
                uint32_t pagenum_u32 =
                    static_cast<uint32_t>(next_page.page_number);
                HAL_CRC_Calculate(&hcrc, &pagenum_u32, 4);
                last_crc =
                    HAL_CRC_Accumulate(&hcrc, (uint32_t*)next_page.data, 256) ^
                    0xFFFFFFFF;

                if (last_crc != next_page.crc) {
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
                Transition(SENSOR_FLASH_STATE_VERIFYING);
            }
            break;

        case SENSOR_FLASH_STATE_VERIFYING:
            if (on_enter) {
                sequence_number = -1;
                readout_req_number = -1;
            }

            if (sequence_number < readout_req_number) {
                sequence_number++;
                amber::m25pe::ReadData(m25_spi, sequence_number * PAGE_SIZE,
                                       PAGE_SIZE, page_buf);
                readout_page.has_data = true;
                memcpy(readout_page.data, page_buf, PAGE_SIZE);

                readout_page.has_page_number = true;
                uint32_t pagenum_u32 = static_cast<uint32_t>(sequence_number);
                readout_page.page_number = pagenum_u32;

                HAL_CRC_Calculate(&hcrc, &pagenum_u32, 4);
                readout_crc =
                    HAL_CRC_Accumulate(&hcrc, (uint32_t*)page_buf, PAGE_SIZE) ^
                    0xffffffff;
                readout_page.has_crc = true;
                readout_page.crc = readout_crc;
            }

            if (readout_req_number == NUM_PAGES) {
                Transition(SENSOR_FLASH_STATE_DONE);
            }

            break;

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
    msg->has_id = true;
    msg->id = id;

    switch (state) {
        case SENSOR_FLASH_STATE_PROGRAMMING:
            msg->has_page_request = true;
            msg->page_request = request_number;

            msg->has_last_crc = true;
            msg->last_crc = last_crc;
            break;

        case SENSOR_FLASH_STATE_VERIFYING:
            msg->has_sequence_number = true,
            msg->sequence_number = sequence_number;

            msg->has_readout_crc = true;
            msg->readout_crc = readout_crc;

            msg->has_readout_page = true;
            msg->readout_page = readout_page;
            break;

        default:
            break;
    }
}

void UpdateReadoutReqNumber(int32_t req_number) {
    readout_req_number = req_number;
}

}  // namespace flash