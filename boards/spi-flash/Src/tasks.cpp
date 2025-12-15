#include "tasks.hpp"

#include <stdbool.h>

#include "cobs.hpp"
#include "common_macros.hpp"
#include "m25pe.hpp"
#include "spi_master.hpp"

// Proto
#include "pb_decode.h"
#include "pb_encode.h"
#include "spi_flash.pb.h"

// FreeRTOS
#include "FreeRTOS.h"
#include "task.h"

// Drivers
#include "usbd_cdc_if.h"
#include "usbd_core.h"

// CubeMX
#include "adc.h"
#include "gpio.h"
#include "main.h"
#include "spi.h"
#include "tim.h"
#include "usb.h"

extern USBD_HandleTypeDef hUsbDeviceFS;

class SPI : public amber::SpiMaster {
public:
    SPI(SPI_HandleTypeDef* hspi, GPIO_TypeDef* cs_n_port, uint16_t cs_n_pin)
        : hspi_(hspi), cs_n_port_(cs_n_port), cs_n_pin_(cs_n_pin) {}

    void Transmit(uint8_t* tx_data, uint32_t length) override {
        HAL_SPI_Transmit(hspi_, tx_data, length, 100);
    }

    void Receive(uint8_t* rx_data, uint32_t length) override {
        HAL_SPI_Receive(hspi_, rx_data, length, 100);
    }

    void TransmitReceive(uint8_t* tx_data, uint8_t* rx_data,
                         uint32_t length) override {
        HAL_SPI_TransmitReceive(hspi_, tx_data, rx_data, length, 100);
    }

    void SetChipSelect(bool high) override {
        HAL_GPIO_WritePin(cs_n_port_, cs_n_pin_,
                          static_cast<GPIO_PinState>(high));
    }

private:
    SPI_HandleTypeDef* hspi_;
    GPIO_TypeDef* cs_n_port_;
    uint16_t cs_n_pin_;
};

SPI m25_spi(&hspi1, M25_nCHIP_SELECT_GPIO_Port, M25_nCHIP_SELECT_Pin);

enum {
    PRIORITY_1000HZ = 3,
    PRIORITY_10HZ = 1,
};

static const size_t STACK_SIZE_WORDS = 512;

StaticTask_t t1000hz_ctrl;
StackType_t t1000hz_stack[STACK_SIZE_WORDS];

StaticTask_t t10hz_ctrl;
StackType_t t10hz_stack[STACK_SIZE_WORDS];

uint32_t raw_adc[3] = {0};

uint16_t rx_buf_count = 0;
uint8_t rx_buffer[1024];
uint8_t decoded_buffer[1024];
amber::cobs::Decoder rx_decoder(decoded_buffer);
uint8_t rx_counter = 0;

spi_flash_command_t cmd = SPI_FLASH_COMMAND_INIT_ZERO;

void task_1000hz(void* argument) {
    (void)argument;

    TickType_t wake_time = xTaskGetTickCount();

    while (true) {
        if (rx_buf_count > 0) {
            if (rx_decoder.Decode(rx_buffer, rx_buf_count)) {
                rx_counter++;
                pb_istream_s istream = pb_istream_from_buffer(
                    rx_decoder.buffer, rx_decoder.length);
                spi_flash_command_t temp;
                if (pb_decode(&istream, &spi_flash_command_t_msg, &temp)) {
                    cmd = temp;
                }
                rx_decoder.Reset();
            }
            rx_buf_count = 0;
        }

        xTaskDelayUntil(&wake_time, pdMS_TO_TICKS(1));
    }
}

void task_10hz(void* argument) {
    (void)argument;

    // Wait for USB initialization to complete
    while (hUsbDeviceFS.pClassData == NULL) {
        // prevent compiler from optimizing out the loop condition
        asm("nop");
    }

    TickType_t wake_time = xTaskGetTickCount();

    uint8_t tx_counter = 0;
    uint8_t pb_buffer[SPI_FLASH_STATUS_SIZE];
    uint8_t cobs_buffer[amber::cobs::MaxEncodedLength(SPI_FLASH_STATUS_SIZE)];

    while (true) {
        int32_t vrefint_mv =
            __HAL_ADC_CALC_VREFANALOG_VOLTAGE(raw_adc[0], ADC_RESOLUTION12b);
        int32_t temperature = __HAL_ADC_CALC_TEMPERATURE(vrefint_mv, raw_adc[1],
                                                         ADC_RESOLUTION12b);
        // VBAT has an internal resistor divider
        constexpr int VBAT_MULTIPLIER = 3;
        int32_t vbat_mv = __HAL_ADC_CALC_DATA_TO_VOLTAGE(vrefint_mv, raw_adc[2],
                                                         ADC_RESOLUTION12b) *
                          VBAT_MULTIPLIER;

        spi_flash_status_t status{
            .has_tx_counter = true,
            .tx_counter = tx_counter++,
            .has_rx_counter = true,
            .rx_counter = rx_counter,
            .has_temperature_degc = true,
            .temperature_degc = temperature,
            .has_vbat_mv = true,
            .vbat_mv = vbat_mv,
            .has_vrefint_mv = true,
            .vrefint_mv = vrefint_mv,
            .has_echo = cmd.has_value,
            .echo = cmd.value,
        };

        pb_ostream_s ostream =
            pb_ostream_from_buffer(pb_buffer, COUNTOF(pb_buffer));

        if (pb_encode(&ostream, &spi_flash_status_t_msg, &status)) {
            int len = amber::cobs::Encode(pb_buffer, ostream.bytes_written,
                                          cobs_buffer);
            CDC_Transmit(cobs_buffer, len);
        }

        HAL_GPIO_TogglePin(LD4_GPIO_Port, LD4_Pin);
        xTaskDelayUntil(&wake_time, pdMS_TO_TICKS(100));
    }
}

void MX_FREERTOS_Init() {
    HAL_GPIO_WritePin(M25_nWRITE_PROTECT_GPIO_Port, M25_nWRITE_PROTECT_Pin,
                      GPIO_PIN_SET);
    HAL_GPIO_WritePin(M25_nRESET_GPIO_Port, M25_nRESET_Pin, GPIO_PIN_SET);
    HAL_GPIO_WritePin(M25_nCHIP_SELECT_GPIO_Port, M25_nCHIP_SELECT_Pin,
                      GPIO_PIN_SET);

    xTaskCreateStatic(task_1000hz, "100Hz", STACK_SIZE_WORDS, NULL,
                      PRIORITY_1000HZ, t1000hz_stack, &t1000hz_ctrl);

    xTaskCreateStatic(task_10hz, "10Hz", STACK_SIZE_WORDS, NULL, PRIORITY_10HZ,
                      t10hz_stack, &t10hz_ctrl);

    HAL_TIM_Base_Start(&htim6);
    HAL_ADC_Start_DMA(&hadc1, raw_adc, COUNTOF(raw_adc));
    HAL_ADCEx_Calibration_Start(&hadc1);

    vTaskStartScheduler();
}