#include "flash.hpp"

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

static SPI m25_spi(&hspi1, M25_nCHIP_SELECT_GPIO_Port, M25_nCHIP_SELECT_Pin);

static uint32_t last_crc = 0;

void Init(void) {
    state = SENSOR_FLASH_STATE_IDLE;
    // __HAL_RCC_CRC_CLK_DISABLE();
    HAL_GPIO_WritePin(M25_nWRITE_PROTECT_GPIO_Port, M25_nWRITE_PROTECT_Pin,
                      GPIO_PIN_SET);
    HAL_GPIO_WritePin(M25_nRESET_GPIO_Port, M25_nRESET_Pin, GPIO_PIN_SET);
    HAL_GPIO_WritePin(M25_nCHIP_SELECT_GPIO_Port, M25_nCHIP_SELECT_Pin,
                      GPIO_PIN_SET);
}

void Start(void) {
    state = SENSOR_FLASH_STATE_WRITING;
    request_number = 0;
}

void ReceivePage(sensor_flash_page_t* page) {
    if (state != SENSOR_FLASH_STATE_WRITING) {
        // Should be accepting packets
        return;
    }

    if (!page->has_page_number || !page->has_data || !page->has_crc) {
        // Reject incomplete packets
        return;
    }

    if (page->page_number != request_number) {
        // Reject out-of-sequence packets;
        return;
    }

    HAL_CRC_Calculate(&hcrc, &page->page_number, 4);
    last_crc =
        HAL_CRC_Accumulate(&hcrc, (uint32_t*)page->data, 256) ^ 0xFFFFFFFF;

    if (last_crc == page->crc) {
        request_number++;
    }

    if (request_number >= flash::NUM_PAGES) {
        // state = SENSOR_FLASH_STATE_VERIFYING;
        state = SENSOR_FLASH_STATE_DONE;
    }
}

// Accessors
bool IsDone(void) {
    return state == SENSOR_FLASH_STATE_DONE;
}

void PopulateStatus(sensor_flash_status_t* msg) {
    msg->has_state = true;
    msg->state = state;
    msg->has_page_request = (state == SENSOR_FLASH_STATE_WRITING);
    msg->page_request = request_number;

    msg->has_last_crc = true;
    msg->last_crc = last_crc;
}

}  // namespace flash