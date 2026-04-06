#pragma once

namespace thermal {

auto Init() noexcept -> void;
auto Update10Hz() noexcept -> void;
// auto GetCarrierTemp() noexcept -> float;
auto GetCurrentFanDuty() noexcept -> float;

}  // namespace thermal