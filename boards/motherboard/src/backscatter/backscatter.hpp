#pragma once

#include <cstdint>

#include "backscatter.pb.h"

namespace backscatter {

auto Init() noexcept -> void;
auto Update1000hz() noexcept -> void;
auto Receive() noexcept -> void;
auto GetReceiveCount() noexcept -> uint8_t;
auto HandleCommand(base_station_command_t* cmd) noexcept -> void;

}  // namespace backscatter
