/** 
 * @file digital.hpp
 * @author Ivan Lange
 * @brief Digital input/output driver wrapper
 * 
 * @date 2026-03-22
 */

#pragma once

#include <cstdint>

namespace amber::periph {

struct DigitalInput {
    DigitalInput(GPIO_TypeDef& port, uint16_t pin)
        : _port(port), _pin(pin) {}

    ~DigitalInput() = default;

    auto Read() const noexcept -> bool {
        return HAL_GPIO_ReadPin(&_port, _pin);
    }

private:
    GPIO_TypeDef& _port;
    uint16_t _pin;
};

struct DigitalOutput {
    DigitalOutput(GPIO_TypeDef& port, uint16_t pin)
        : _port(port), _pin(pin) {}

    ~DigitalOutput() = default;

    auto SetHigh() noexcept -> void {
        HAL_GPIO_WritePin(&_port, _pin, GPIO_PIN_SET);
    }
    auto SetLow() noexcept -> void {
        HAL_GPIO_WritePin(&_port, _pin, GPIO_PIN_RESET);
    }
    auto Toggle() noexcept -> void {
        HAL_GPIO_TogglePin(&_port, _pin);
    }

private:
    GPIO_TypeDef& _port;
    uint16_t _pin;
};

}  // namespace amber::periph
