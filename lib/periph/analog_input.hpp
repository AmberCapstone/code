/** 
 * @file analog_input.hpp
 * @author Ivan Lange
 * @brief Analog input driver wrapper
 * 
 * @date 2026-03-22
 */

#pragma once

#include <cstdint>

#ifdef STM32F7
#include "stm32f7xx_hal.h"
#elif STM32G0
#include "stm32g0xx_hal.h"

#endif

#ifdef HAL_ADC_MODULE_ENABLED

namespace amber::periph {

struct AnalogInput {
    AnalogInput(ADC_HandleTypeDef& hadc, uint32_t channel, float systemVoltage = 3.3f)
        : _hadc(hadc),
          _adcChannel(channel),
          _systemVoltage(systemVoltage) {};

    ~AnalogInput() = default;

    auto ReadVoltage() noexcept -> float {
        Start();
        HAL_ADC_PollForConversion(&_hadc, HAL_MAX_DELAY);
        auto rawValue = HAL_ADC_GetValue(&_hadc);
        HAL_ADC_Stop(&_hadc);

        return (rawValue / GetDivisor()) * _systemVoltage;
    }

private:
    ADC_HandleTypeDef& _hadc;
    uint32_t _adcChannel;
    const float _systemVoltage;

    auto Start() noexcept -> void {
        ADC_ChannelConfTypeDef adcConfig = {
            .Channel = _adcChannel,
            .Rank = ADC_REGULAR_RANK_1,
            .SamplingTime = ADC_SAMPLETIME_28CYCLES
            .Offset = 0
        };

        HAL_ADC_ConfigChannel(&_hadc, &adcConfig);
        HAL_ADC_Start(&_hadc);
    }

    auto GetDivisor() const noexcept -> float {
        switch (ADC_GET_RESOLUTION(&_hadc)) {
            case ADC_RESOLUTION_12B:
                return 4095.0f;
            case ADC_RESOLUTION_10B:
                return 1023.0f;
            case ADC_RESOLUTION_8B:
                return 255.0f;
            case ADC_RESOLUTION_6B:
                return 63.0f;
            default:
                return 4095.0f;
        }
    }
};

}  // namespace amber::periph

#endif  // HAL_ADC_MODULE_ENABLED
