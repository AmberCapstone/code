#pragma once

#include <cstdint>

namespace backscatter {

auto Init() noexcept -> void;
auto Update1000hz() noexcept -> void;
auto GetUartByte() noexcept -> uint8_t;
auto GetReceiveCount() noexcept -> uint8_t;

}  // namespace backscatter