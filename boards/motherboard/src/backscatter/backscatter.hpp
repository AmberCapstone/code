#pragma once

#include <cstdint>

namespace backscatter {

auto Init() noexcept -> void;
auto Update1000hz() noexcept -> void;
auto GetReceiveCount() noexcept -> uint8_t;
auto GetXCoord() noexcept -> uint16_t;
auto GetYCoord() noexcept -> uint16_t;

}  // namespace backscatter