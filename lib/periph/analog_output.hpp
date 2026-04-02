/** 
 * @file analog_output.hpp
 * @author Blake Freer and Ivan Lange
 * @brief Analog output driver wrapper
 * 
 * @date 2026-03-22
 */

#pragma once

#include <algorithm>
#include <cstdint>

#ifdef STM32F7
#include "stm32f7xx_hal.h"
#elif STM32G0
#include "stm32g0xx_hal.h"

#endif

#ifdef HAL_DAC_MODULE_ENABLED

namespace amber::periph {

struct AnalogOutput {
    AnalogOutput(DAC_HandleTypeDef& hdac, uint32_t channel, float systemVoltage = 3.3f)
        : _hdac(hdac),
          _channel(channel),
          _systemVoltage(systemVoltage) {};

    ~AnalogOutput() = default;

    auto SetVoltage(float voltage) noexcept -> void {
        HAL_DAC_Start(&_hdac, _channel);

        uint32_t dacValue = std::clamp<float>(
            (voltage / _systemVoltage) * GetResolution(), 0, GetResolution()
        );

        HAL_DAC_SetValue(&_hdac, _channel, DAC_ALIGN_12B_R, dacValue);
    }

private:
    DAC_HandleTypeDef& _hdac;
    uint32_t _channel;
    const float _systemVoltage;

    auto GetResolution() const noexcept -> uint32_t {
        switch (DAC_GET_RESOLUTION(&_hdac)) {
            case DAC_RESOLUTION_12B:
                return 0xFFF;
            case DAC_RESOLUTION_8B:
                return 0xFF;
            default:
                return 0xFFF; // Default to 12-bit resolution
        }
    }
};

}  // namespace amber::periph

#endif  // HAL_DAC_MODULE_ENABLED
