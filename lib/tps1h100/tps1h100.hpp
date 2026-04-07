/**
 * @file tps1h100.hpp
 * @author Ivan Lange
 * @brief Driver for TPS1H100 4-Channel HSD
 * 
 * @date 2026-04-04
 */

#pragma once

#include "periph/analog_input.hpp"
#include "periph/digital.hpp"

namespace amber::tps1h100 {

static constexpr uint16_t kCurrentSenseRatio = 500;

struct Config {
    uint16_t currentSenseResistor;

    amber::periph::DigitalOutput* enablePin;
    amber::periph::DigitalOutput& diagEn;
    amber::periph::AnalogInput& currentSense;
};

struct Driver {
    Driver(const Config& cfg);
    ~Driver() = default;

    // channel control
    auto enable() noexcept -> void;
    auto disable() noexcept -> void;
    auto set(bool enable) noexcept -> void;

    // diagnostics
    auto diagEnable(bool en) noexcept -> void;
    auto getCurrent() const noexcept -> float;

private:
    const Config& _cfg;
};

}  // namespace amber::tps1h100
