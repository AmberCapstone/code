/**
 * @file tps274160.hpp
 * @author Ivan Lange
 * @brief Driver for TPS274160B 4-Channel HSD
 * 
 * @date 2026-04-04
 */

#pragma once

#include <array>

#include "periph/analog_input.hpp"
#include "periph/digital.hpp"

namespace amber::tps274160b {

static constexpr uint8_t kNumChannels = 4;
static constexpr uint8_t kNumDiagPins = 2;
static constexpr uint16_t kCurrentSenseRatio = 300;

struct Config {
    uint16_t currentSenseResistor;

    std::array<amber::periph::DigitalOutput, kNumChannels> enablePins;
    std::array<amber::periph::DigitalOutput, kNumDiagPins> diagSelect;
    amber::periph::DigitalInput& fault;
    amber::periph::DigitalOutput& diagEn;
    amber::periph::AnalogInput& currentSense;
};

struct Driver {
    Driver(Config& cfg);
    ~Driver() = default;

    // channel control
    auto enable(const uint8_t channel) noexcept -> void;
    auto disable(const uint8_t channel) noexcept -> void;
    auto set(uint8_t channel, bool enable) noexcept -> void;
    auto enableAll() noexcept -> void;
    auto disableAll() noexcept -> void;

    // diagnostics
    auto getFault() const noexcept -> bool;
    auto diagEnable(bool en) noexcept -> void;
    auto selectDiagPin(uint8_t pin) noexcept -> void;
    auto getCurrent() const noexcept -> float;

private:
    Config& _cfg;
};

}  // namespace amber::tps274160b
