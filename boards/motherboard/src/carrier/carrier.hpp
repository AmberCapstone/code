#pragma once

namespace carrier {

auto Init() noexcept -> void;
auto Update_100hz() noexcept -> void;
auto GetLpaPowerDetect() noexcept -> float;
auto GetVcoLocked() noexcept -> bool;
auto GetPowerDown() noexcept -> bool;

}  // namespace carrier