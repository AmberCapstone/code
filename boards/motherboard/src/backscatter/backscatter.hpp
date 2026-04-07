#pragma once

#include <cstdint>

#include "backscatter.pb.h"

namespace backscatter {

auto Init() noexcept -> void;
auto Update1000hz() noexcept -> void;
auto Receive() noexcept -> void;
auto GetReceiveCount() noexcept -> uint8_t;
auto GetBackscatterBadMessages() noexcept -> uint16_t;
auto GetStatus(backscatter_status_t&) noexcept -> void;
auto GetDacThreshold() noexcept -> float;

}  // namespace backscatter
