#include "sensors.hpp"

#include <cstdint>
#include <cstdlib>

// CubeMX
#include "adc.h"
#include "common_macros.hpp"
#include "stm32u0xx_hal.h"
#include "tim.h"

namespace sensors {

static uint32_t RESOLUTION = ADC_RESOLUTION12b;  // updated in Init()

static uint32_t raw_adc[3] = {0};
static int32_t temperature_degc = 0;
static int32_t vbat_mv = 0;
static int32_t vrefint_mv = 0;

void Init(void) {
    RESOLUTION = hadc1.Init.Resolution;

    HAL_TIM_Base_Start(&htim6);
    HAL_ADC_Start_DMA(&hadc1, raw_adc, COUNTOF(raw_adc));
    HAL_ADCEx_Calibration_Start(&hadc1);
}

void Update_10hz(void) {
    vrefint_mv = __HAL_ADC_CALC_VREFANALOG_VOLTAGE(raw_adc[0], RESOLUTION);
    temperature_degc =
        __HAL_ADC_CALC_TEMPERATURE(vrefint_mv, raw_adc[1], RESOLUTION);

    // VBAT has an internal resistor divider
    constexpr int VBAT_MULTIPLIER = 3;
    vbat_mv =
        __HAL_ADC_CALC_DATA_TO_VOLTAGE(vrefint_mv, raw_adc[2], RESOLUTION) *
        VBAT_MULTIPLIER;
}

int32_t GetTemperatureC(void) {
    return temperature_degc;
}

int32_t GetVrefintMv(void) {
    return vrefint_mv;
}

int32_t GetVbatMv(void) {
    return vbat_mv;
}

void PopulateStatus(spi_flash_status_t* msg) {
    msg->has_temperature_degc = true;
    msg->temperature_degc = sensors::GetTemperatureC();
    msg->has_vbat_mv = true;
    msg->vbat_mv = sensors::GetVbatMv();
    msg->has_vrefint_mv = true;
    msg->vrefint_mv = sensors::GetVrefintMv();
}

}  // namespace sensors