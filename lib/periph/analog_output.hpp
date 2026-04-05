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
#elif defined(STM32G0)
#include "stm32g0xx_hal.h"
#endif

#ifdef HAL_DAC_MODULE_ENABLED

namespace amber::periph {

enum class DacResolution : uint8_t {
    Bits8  = 8,
    Bits12 = 12,
};

struct AnalogOutput {
    AnalogOutput(DAC_HandleTypeDef& hdac,
                 uint32_t channel,
                 DacResolution resolution  = DacResolution::Bits12,
                 float systemVoltage       = 3.3f)
        : _hdac(hdac),
          _channel(channel),
          _resolution(resolution),
          _systemVoltage(systemVoltage) {}

    ~AnalogOutput() = default;

    auto SetVoltage(float voltage) noexcept -> void {
        HAL_DAC_Start(&_hdac, _channel);

        const uint32_t maxCount = GetMaxCount();
        const uint32_t dacValue = static_cast<uint32_t>(
            std::clamp(voltage / _systemVoltage, 0.0f, 1.0f) * maxCount
        );

        const uint32_t align = (_resolution == DacResolution::Bits8)
                                   ? DAC_ALIGN_8B_R
                                   : DAC_ALIGN_12B_R;

        HAL_DAC_SetValue(&_hdac, _channel, align, dacValue);
    }

private:
    DAC_HandleTypeDef& _hdac;
    uint32_t           _channel;
    DacResolution      _resolution;
    const float        _systemVoltage;

    auto GetMaxCount() const noexcept -> uint32_t {
        return (_resolution == DacResolution::Bits8) ? 0xFFu : 0xFFFu;
    }
};

}  // namespace amber::periph

#endif  // HAL_DAC_MODULE_ENABLED
