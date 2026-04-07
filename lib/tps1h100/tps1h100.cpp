#include "tps1h100.hpp"

namespace amber::tps1h100 {

Driver::Driver(const Config& cfg) : _cfg(cfg) {}

auto Driver::enable() noexcept -> void {
    if (_cfg.enablePin == nullptr) { return; }
    _cfg.enablePin->SetHigh();
};

auto Driver::disable() noexcept -> void {
    if (_cfg.enablePin == nullptr) { return; }
    _cfg.enablePin->SetLow();
};

auto Driver::set(bool enable) noexcept -> void {
    if (_cfg.enablePin == nullptr) { return; }
    _cfg.enablePin->Set(enable);
};

auto Driver::diagEnable(bool en) noexcept -> void {
    _cfg.diagEn.Set(en);
};

auto Driver::getCurrent() const noexcept -> float {
    const float voltage = _cfg.currentSense.ReadVoltage();
    return (voltage * kCurrentSenseRatio) / _cfg.currentSenseResistor;
};

}  // namespace amber::tps1h100
