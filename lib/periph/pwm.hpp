/**
 * @file pwm.hpp
 * @author Blake Freer and Ivan Lange
 * @brief PWM driver wrapper
 *
 * @date 2026-03-21
 */

#pragma once

#include <algorithm>
#include <cstdint>

#ifdef STM32F7
#include "stm32f7xx_hal.h"
#elif STM32G0
#include "stm32g0xx_hal.h"

#endif

#ifdef HAL_TIM_MODULE_ENABLED

namespace amber::periph {

struct Pwm {
    Pwm(TIM_HandleTypeDef& htim, uint32_t channel)
        : _htim(htim), _channel(channel) {}

    ~Pwm() = default;

    auto Start() noexcept -> void {
        HAL_TIM_PWM_Start(&_htim, _channel);
    }

    auto Stop() noexcept -> void {
        HAL_TIM_PWM_Stop(&_htim, _channel);
    }

    auto SetDutyCycle(const float dutyCycle) noexcept -> void {
        _dutyCycle = std::clamp<float>(dutyCycle, 0, 100);

        uint32_t pulse = static_cast<uint32_t>((_dutyCycle / 100.0f) *
                                               (_htim.Init.Period + 1));
        __HAL_TIM_SET_COMPARE(&_htim, _channel, pulse);
    }

    auto GetDutyCycle() const noexcept -> float {
        uint32_t pulse = __HAL_TIM_GET_COMPARE(&_htim, _channel);
        uint32_t period = __HAL_TIM_GET_AUTORELOAD(&_htim);

        return (static_cast<float>(pulse) / static_cast<float>(period)) *
               100.0f;
    }

    auto SetFrequency(const float frequency) noexcept -> void {
        float freq = std::max(kMinFrequency, frequency);
        uint32_t autoReload = static_cast<uint32_t>(
            static_cast<float>(GetTimerFrequency()) / freq);

        __HAL_TIM_SET_AUTORELOAD(&_htim, autoReload);
    }

    auto GetFrequency() const noexcept -> float {
        float freq = static_cast<float>(GetTimerFrequency()) /
                     (static_cast<float>(__HAL_TIM_GET_AUTORELOAD(&_htim)));

        return freq;
    }

private:
    static constexpr float kMinFrequency = 0.000015259022f;

    TIM_HandleTypeDef& _htim;
    uint32_t _channel;

    float _dutyCycle = 0.0;

    auto GetTimerFrequency() const noexcept -> float {
        auto tickFreq = HAL_GetTickFreq();
        return tickFreq;
    }
};

}  // namespace amber::periph

#endif  // HAL_TIM_MODULE_ENABLED
