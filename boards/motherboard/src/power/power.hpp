#pragma once

#include <array>
#include <cstdint>

#include "lib/tps1h100/tps1h100.hpp"
#include "lib/tps274160b/tps274160b.hpp"

namespace power {

enum class PowerMuxState : uint8_t {
    USB_POWER = 0,
    BARREL_JACK = 1,
};

auto Init() noexcept -> void;
auto Update_100hz() noexcept -> void;

auto GetPowerMuxState() noexcept -> PowerMuxState;
auto GetP6VHsd1Currents() noexcept
    -> const std::array<float, amber::tps274160b::kNumChannels>&;
auto GetP6VHsd2Currents() noexcept
    -> const std::array<float, amber::tps274160b::kNumChannels>&;
auto GetP6VScatterCurrent() noexcept -> float;
auto GetP12VCurrent() noexcept -> float;

}  // namespace power
