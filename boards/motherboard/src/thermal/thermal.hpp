#pragma once

namespace thermal {

auto Init() noexcept -> void;
auto Update10Hz() noexcept -> void;

}  // namespace thermal